// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

/// PendingStore is used to store the pending blocks and txs, and handle the reorg
module bitcoin_move::pending_block{
    
    use std::vector;
    use std::option::{Self, Option};
    use moveos_std::signer;
    use moveos_std::signer::module_signer;
    use moveos_std::module_store::{ensure_upgrade_permission};

    use moveos_std::object::{Self, Object, ObjectID};
    use moveos_std::simple_map::{Self, SimpleMap};
    use moveos_std::event;
    use moveos_std::type_info;
    
    use bitcoin_move::types::{Self, Transaction, Header, Block, BlockHeightHash};

    friend bitcoin_move::genesis;
    friend bitcoin_move::bitcoin;
    friend bitcoin_move::inscription_updater;

    const ErrorBlockAlreadyProcessed:u64 = 1;
    const ErrorPendingBlockNotFound:u64 = 2;
    const ErrorPendingTxNotFound:u64 = 3;
    const ErrorReorgFailed:u64 = 4;
    const ErrorNeedToWaitMoreBlocks:u64 = 5;
    const ErrorPendingBlockNotFinished:u64 = 6;
    const ErrorUnsupportedChain:u64 = 7;

    const TX_IDS_KEY: vector<u8> = b"tx_ids";

    struct PendingBlock has key{
        block_height: u64,
        block_hash: address,
        header: Header,
        processed_tx: u64,
        next_block_hash: Option<address>,
    }

    struct PendingStore has key{
        /// block_height -> block_hash
        pending_blocks: SimpleMap<u64, address>,
        /// The best block height and hash
        best_block: Option<BlockHeightHash>,
        /// How many blocks we should pending for reorg
        reorg_block_count: u64,
    }

    /// InprocessBlock is used to store the block and txs that are being processed
    /// This is a hot potato struct, can not be store and drop 
    struct InprocessBlock {
        block_hash: address,
        block_obj: Object<PendingBlock>,
        tx: Transaction,
    }

    struct ReorgEvent has copy, drop{
        block_height: u64,
        block_hash: address,
        success: bool,
    }

    public(friend) fun genesis_init(reorg_block_count: u64){
        let store_obj = object::new_named_object(PendingStore{
            pending_blocks: simple_map::new(),
            best_block: option::none(),
            reorg_block_count,
        });
        object::transfer_extend(store_obj, @bitcoin_move);
    }

    fun pending_block_obj_id(block_hash: address): ObjectID{
        object::custom_object_id<address, PendingBlock>(block_hash)
    }

    fun borrow_store(): &PendingStore {
        let obj_id = object::named_object_id<PendingStore>();
        let store_obj = object::borrow_object(obj_id);
        object::borrow(store_obj)
    }

    fun borrow_mut_store(): &mut PendingStore {
        let obj_id = object::named_object_id<PendingStore>();
        let store_obj = object::borrow_mut_object_extend(obj_id);
        object::borrow_mut(store_obj)
    }
    
    fun exists_pending_block(block_hash: address): bool{
        let block_obj_id = pending_block_obj_id(block_hash);
        object::exists_object(block_obj_id)
    }

    fun borrow_pending_block(block_hash: address): &Object<PendingBlock>{
        let block_obj_id = pending_block_obj_id(block_hash);
        assert!(object::exists_object(block_obj_id), ErrorPendingBlockNotFound);
        object::borrow_object(block_obj_id)
    }

    fun borrow_mut_pending_block(block_hash: address): &mut Object<PendingBlock>{
        let block_obj_id = pending_block_obj_id(block_hash);
        assert!(object::exists_object(block_obj_id), ErrorPendingBlockNotFound);
        object::borrow_mut_object_extend(block_obj_id)
    }

    fun take_pending_block(block_hash: address): Object<PendingBlock>{
        let block_obj_id = object::custom_object_id<address, PendingBlock>(block_hash);
        assert!(object::exists_object(block_obj_id), ErrorPendingBlockNotFound);
        let obj = object::take_object_extend(block_obj_id);
        obj
    }

    public(friend) fun add_pending_block(block_height: u64, block_hash: address, block: Block) : bool{
        let block_obj_id = pending_block_obj_id(block_hash);
        if(object::exists_object(block_obj_id)){
            return false
        };

        let store = borrow_mut_store();
        if(simple_map::contains_key(&store.pending_blocks, &block_height)){
            // block already exists, need to process reorg
            handle_reorg(store, block_height);
        };
        let (header, txs) = types::unpack_block(block);
        let prev_block_hash = types::prev_blockhash(&header);
        let block_obj = object::new_with_id(block_hash, PendingBlock{
            block_height: block_height,
            block_hash: block_hash,
            header: header,
            processed_tx: 0,
            next_block_hash: option::none(),
        });
        let tx_ids = vector::empty<address>(); 
        vector::for_each(txs, |tx| {
            let txid = types::tx_id(&tx);
            object::add_field(&mut block_obj, txid, tx);
            vector::push_back(&mut tx_ids, txid);
        });
        object::add_field(&mut block_obj, TX_IDS_KEY, tx_ids);
        
        object::transfer_extend(block_obj, @bitcoin_move);
        
        if(exists_pending_block(prev_block_hash)){
            let prev_block_obj = borrow_mut_pending_block(prev_block_hash);
            let prev_block = object::borrow_mut(prev_block_obj);
            prev_block.next_block_hash = option::some(block_hash);
        };

        simple_map::add(&mut store.pending_blocks, block_height, block_hash);
        //The relayer should ensure the new block is the best block
        //Maybe we should calculate the difficulty here in the future
        store.best_block = option::some(types::new_block_height_hash(block_height, block_hash));
        true
    }

    fun handle_reorg(store: &mut PendingStore, reorg_block_height: u64){
        let (_, reorg_block_hash) = simple_map::remove(&mut store.pending_blocks, &reorg_block_height);
        let reorg_block = take_pending_block(reorg_block_hash);
        let next_block_hash_option = object::borrow(&reorg_block).next_block_hash;
        handle_reorg_block(reorg_block);
        while(option::is_some(&next_block_hash_option)){
            let next_block_hash = option::destroy_some(next_block_hash_option);
            let next_block = take_pending_block(next_block_hash);
            let next_block_height = object::borrow(&next_block).block_height;
            
            next_block_hash_option = object::borrow(&next_block).next_block_hash;

            handle_reorg_block(next_block);
            simple_map::remove(&mut store.pending_blocks, &next_block_height);
        };
    }

    fun handle_reorg_block(obj: Object<PendingBlock>){
        let block_height = object::borrow(&obj).block_height;
        let block_hash = object::borrow(&obj).block_hash;
        // If the block already processed, we can't remove it, and reorg failed
        if(object::borrow(&obj).processed_tx > 0){
            event::emit(ReorgEvent{
                block_height,
                block_hash,
                success: false,
            });
            abort ErrorReorgFailed
        };
        remove_pending_block(obj, false);
        event::emit(ReorgEvent{
            block_height,
            block_hash,
            success: true,
        });
    }

    fun remove_pending_block(obj: Object<PendingBlock>, processed: bool): Header{
        // We need to remove all txs from the block before removing the block
        let ids:vector<address> = object::remove_field(&mut obj, TX_IDS_KEY);
        // If the block is not processed, we need to remove all txs
        // otherwise, the tx has been removed by the tx processing
        if (processed){
            assert!(object::borrow(&obj).processed_tx == vector::length(&ids), ErrorPendingBlockNotFinished);
        }else{
            vector::for_each(ids, |txid| {
                // Directly drop the tx
                let _tx: Transaction = object::remove_field(&mut obj, txid);  
            });
        };
        
        let pending_block = object::remove(obj);
        let PendingBlock{block_height:_, block_hash:_, header, processed_tx:_, next_block_hash:_} = pending_block;
        header
    }

    public(friend) fun block_height(pending_block: &Object<PendingBlock>): u64{
        let block = object::borrow(pending_block);
        block.block_height
    }

    /// The intermediate is used to store the intermediate state during the tx processing
    public(friend) fun take_intermediate<I: store>(pending_block: &mut Object<PendingBlock>): I{
        let intermediate_name = type_info::type_name<I>();
        let intermediate = object::remove_field(pending_block, intermediate_name);
        intermediate
    }

    public(friend) fun add_intermediate<I: store>(pending_block: &mut Object<PendingBlock>, intermediate: I){
        let intermediate_name = type_info::type_name<I>();
        object::add_field(pending_block, intermediate_name, intermediate);
    }

    public(friend) fun exists_intermediate<T>(pending_block: &Object<PendingBlock>): bool{
        let intermediate_name = type_info::type_name<T>();
        object::contains_field(pending_block, intermediate_name)
    }

    // ============== Pending Tx Processing ==============

    public(friend) fun process_pending_tx(block_hash: address, txid: address): InprocessBlock{
        let store = borrow_mut_store();
        let block_obj = take_pending_block(block_hash);
        let (best_block_height, _best_block_hash) = types::unpack_block_height_hash(*option::borrow(&store.best_block));
        assert!(best_block_height >= store.reorg_block_count && best_block_height - store.reorg_block_count >= object::borrow(&block_obj).block_height, ErrorNeedToWaitMoreBlocks);
        assert!(object::contains_field(&block_obj, txid), ErrorPendingTxNotFound);
        let tx = object::remove_field(&mut block_obj, txid);
        let inprocess_block = InprocessBlock{
            block_hash: block_hash,
            block_obj: block_obj,
            tx: tx,
        };
        inprocess_block
    }

    public(friend) fun finish_pending_tx(inprocess_block: InprocessBlock){
        let InprocessBlock{block_hash:_, block_obj, tx:_} = inprocess_block;
        let pending_block = object::borrow_mut(&mut block_obj);
        pending_block.processed_tx = pending_block.processed_tx + 1;
        object::transfer_extend(block_obj, @bitcoin_move);
    }

    public(friend) fun finish_pending_block(inprocess_block: InprocessBlock): Header{
        let InprocessBlock{block_hash:_, block_obj, tx} = inprocess_block;
         // The coinbase tx should be the last tx in the block
        // If the coinbase tx is processed, we can remove the block
        assert!(types::is_coinbase_tx(&tx), ErrorPendingBlockNotFinished);
        let pending_block = object::borrow_mut(&mut block_obj);
        pending_block.processed_tx = pending_block.processed_tx + 1;
        let block_height = pending_block.block_height;
        let header = remove_pending_block(block_obj, true);
        let store = borrow_mut_store();
        simple_map::remove(&mut store.pending_blocks, &block_height);
        header
    }

    public(friend) fun inprocess_block_pending_block(inprocess_block: &mut InprocessBlock): &mut Object<PendingBlock>{
        &mut inprocess_block.block_obj
    }

    public(friend) fun inprocess_block_tx(inprocess_block: &InprocessBlock): &Transaction{
        &inprocess_block.tx
    }

    public(friend) fun inprocess_block_header(inprocess_block: &InprocessBlock): &Header{
        let block_obj = object::borrow(&inprocess_block.block_obj);
        &block_obj.header
    }

    public(friend) fun inprocess_block_height(inprocess_block: &InprocessBlock): u64{
        let block_obj = object::borrow(&inprocess_block.block_obj);
        block_obj.block_height
    }

    // ============== Pending Block Query ==============

    struct PendingTxs has copy, drop, store{
        block_hash: address,
        txs: vector<address>,
    }

    /// Get the pending txs which are ready to be processed
    public fun get_ready_pending_txs(): Option<PendingTxs>{
        let store = borrow_store();
        if(option::is_none(&store.best_block)){
            return option::none()
        };
        let (best_block_height, _best_block_hash) = types::unpack_block_height_hash(*option::borrow(&store.best_block));
        if(best_block_height < store.reorg_block_count){
            return option::none()
        };
        let ready_block_height = best_block_height - store.reorg_block_count;
        if(!simple_map::contains_key(&store.pending_blocks, &ready_block_height)){
            return option::none()
        };
        let block_hash = *simple_map::borrow(&store.pending_blocks, &ready_block_height);
        let block_obj = borrow_pending_block(block_hash);
        let prev_block_hash = types::prev_blockhash(&object::borrow(block_obj).header);
        while(exists_pending_block(prev_block_hash)){
            let prev_block_obj = borrow_pending_block(prev_block_hash);
            prev_block_hash = types::prev_blockhash(&object::borrow(prev_block_obj).header);
            block_obj = prev_block_obj;
        };

        let tx_ids: vector<address> = *object::borrow_field(block_obj, TX_IDS_KEY);
        let unprocessed_tx_ids : vector<address> = vector::filter(tx_ids, |txid| {
            object::contains_field(block_obj, *txid)
        });
        let pending_txs = PendingTxs{
            block_hash: block_hash,
            txs: unprocessed_tx_ids,
        };
        option::some(pending_txs)
    }

    public fun get_best_block(): Option<BlockHeightHash>{
        let store = borrow_store();
        *&store.best_block
    }

    public fun get_reorg_block_count(): u64{
        let store = borrow_store();
        store.reorg_block_count
    }

    //====== Update functions ======

    /// Update the `reorg_block_count` config
    public entry fun update_reorg_block_count(signer: &signer, count: u64){
        let module_signer = module_signer<PendingStore>();
        let package_id = signer::address_of(&module_signer);
        ensure_upgrade_permission(package_id, signer);

        let store = borrow_mut_store();
        store.reorg_block_count = count;
    }

    /// Update the `reorg_block_count` config for local env to testing
    public entry fun update_reorg_block_count_for_local(count: u64){
        assert!(rooch_framework::chain_id::is_local(), ErrorUnsupportedChain);
        let store = borrow_mut_store();
        store.reorg_block_count = count;
    } 
}