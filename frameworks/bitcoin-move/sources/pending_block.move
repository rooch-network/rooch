// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

/// PendingStore is used to store the pending blocks and txs, and handle the reorg
module bitcoin_move::pending_block{
    
    use std::vector;
    use std::option::{Self, Option};
    use moveos_std::object::{Self, Object};
    use moveos_std::simple_map::{Self, SimpleMap};
    use moveos_std::event;
    use bitcoin_move::types::{Self, Transaction, Header, Block};
    use bitcoin_move::ord::{Flotsam};

    friend bitcoin_move::genesis;
    friend bitcoin_move::bitcoin;

    const ErrorBlockAlreadyProcessed:u64 = 1;
    const ErrorPendingBlockNotFound:u64 = 2;
    const ErrorPendingTxNotFound:u64 = 3;
    const ErrorReorgFailed:u64 = 4;
    const ErrorNeedToWaitMoreBlocks:u64 = 5;
    const ErrorPendingBlockNotFinished:u64 = 6;

    const TX_IDS_KEY: vector<u8> = b"tx_ids";
    const BLOCK_FLOTSAM_KEY: vector<u8> = b"block_flotsam";

    struct PendingBlock has key{
        block_height: u64,
        header: Header,
        processed_tx: u64,
    }

    struct PendingBlockID has copy, store, drop{
        block_hash: address,
    }

    struct PendingStore has key{
        /// block_height -> block_hash
        pending_blocks: SimpleMap<u64, address>,
        /// The latest pending block height
        latest_block_height: Option<u64>,
        /// How many blocks we should pending for reorg
        reorg_pending_block_count: u64,
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

    public(friend) fun genesis_init(reorg_pending_block_count: u64){
        let store_obj = object::new_named_object(PendingStore{
            pending_blocks: simple_map::new(),
            latest_block_height: option::none(),
            reorg_pending_block_count,
        });
        object::transfer_extend(store_obj, @bitcoin_move);
    }

    public(friend) fun new_pending_block_id(block_hash: address): PendingBlockID {
        PendingBlockID{
            block_hash: block_hash,
        }
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

    fun borrow_pending_block(block_hash: address): &Object<PendingBlock>{
        let block_id = new_pending_block_id(block_hash);
        let block_obj_id = object::custom_object_id<PendingBlockID, PendingBlock>(block_id);
        assert!(object::exists_object(block_obj_id), ErrorPendingBlockNotFound);
        object::borrow_object(block_obj_id)
    }

    fun borrow_mut_pending_block(block_hash: address): &mut Object<PendingBlock>{
        let block_id = new_pending_block_id(block_hash);
        let block_obj_id = object::custom_object_id<PendingBlockID, PendingBlock>(block_id);
        assert!(object::exists_object(block_obj_id), ErrorPendingBlockNotFound);
        object::borrow_mut_object_extend(block_obj_id)
    }

    fun take_pending_block(block_hash: address): Object<PendingBlock>{
        let block_id = new_pending_block_id(block_hash);
        let block_obj_id = object::custom_object_id<PendingBlockID, PendingBlock>(block_id);
        assert!(object::exists_object(block_obj_id), ErrorPendingBlockNotFound);
        let (_, obj) = object::take_object_extend(block_obj_id);
        obj
    }

    public(friend) fun add_pending_block(block_height: u64, block_hash: address, block: Block){
        let block_id = new_pending_block_id(block_hash);
        let block_obj_id = object::custom_object_id<PendingBlockID, PendingBlock>(block_id);
        assert!(!object::exists_object(block_obj_id), ErrorBlockAlreadyProcessed);

        let store = borrow_mut_store();
        if(simple_map::contains_key(&store.pending_blocks, &block_height)){
            // block already exists, need to process reorg
            handle_reog(store, block_height);
        };
        let (header, txs) = types::unpack_block(block);
        
        let block_obj = object::new_with_id(block_id, PendingBlock{
            block_height: block_height,
            header: header,
            processed_tx: 0,
        });
        let tx_ids = vector::empty<address>(); 
        vector::for_each(txs, |tx| {
            let txid = types::tx_id(&tx);
            object::add_field(&mut block_obj, txid, tx);
            vector::push_back(&mut tx_ids, txid);
        });
        object::add_field(&mut block_obj, TX_IDS_KEY, tx_ids);
        object::transfer_extend(block_obj, @bitcoin_move);
        simple_map::add(&mut store.pending_blocks, block_height, block_hash);
        if(option::is_none(&store.latest_block_height)){
            store.latest_block_height = option::some(block_height);
        }else{
            let current_height = *option::borrow(&store.latest_block_height);
            if(block_height > current_height){
                store.latest_block_height = option::some(block_height);
            };
        }
    }

    fun handle_reog(store: &mut PendingStore, block_height: u64){
        // if the reorg happen, the latest block height should not be none.
        let current_height = *option::borrow(&store.latest_block_height);
        while(current_height >= block_height){
            let (_, prev_hash) = simple_map::remove(&mut store.pending_blocks, &current_height);
            let obj = take_pending_block(prev_hash);
            // If the block already processed, we can't remove it, and reorg failed
            if(object::borrow(&obj).processed_tx > 0){
                event::emit(ReorgEvent{
                    block_height: current_height,
                    block_hash: prev_hash,
                    success: false,
                });
                abort ErrorReorgFailed
            };
            remove_pending_block(obj, false);
            current_height = current_height - 1;
            event::emit(ReorgEvent{
                block_height: current_height,
                block_hash: prev_hash,
                success: true,
            });
        };
        store.latest_block_height = option::some(block_height);
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
        
        if(object::contains_field(&obj, BLOCK_FLOTSAM_KEY)){
            let _flotsam: vector<Flotsam> = object::remove_field(&mut obj, BLOCK_FLOTSAM_KEY);
        };
        let pending_block = object::remove(obj);
        let PendingBlock{block_height:_, header, processed_tx:_} = pending_block;
        header
    }

    // ============== Pending Tx Processing ==============

    public(friend) fun process_pending_tx(block_hash: address, txid: address): InprocessBlock{
        let store = borrow_mut_store();
        let block_obj = take_pending_block(block_hash);
        let latest_block_height = *option::borrow(&store.latest_block_height);
        assert!(object::borrow(&block_obj).block_height + store.reorg_pending_block_count >= latest_block_height, ErrorNeedToWaitMoreBlocks);
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

    public(friend) fun inprocess_block_flotsams_mut(inprocess_block: &mut InprocessBlock): &mut vector<Flotsam>{
        object::borrow_mut_field_with_default(&mut inprocess_block.block_obj, BLOCK_FLOTSAM_KEY, vector::empty())
    }

    public(friend) fun inprocess_block_flotsams(inprocess_block: &InprocessBlock): vector<Flotsam>{
        let default = vector::empty<Flotsam>();
        *object::borrow_field_with_default(&inprocess_block.block_obj, BLOCK_FLOTSAM_KEY, &default)
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
        if(option::is_none(&store.latest_block_height)){
            return option::none()
        };
        let latest_block_height = *option::borrow(&store.latest_block_height);
        let ready_block_height = latest_block_height - store.reorg_pending_block_count;
        if(!simple_map::contains_key(&store.pending_blocks, &ready_block_height)){
            return option::none()
        };
        let block_hash = *simple_map::borrow(&store.pending_blocks, &ready_block_height);
        let block_obj = borrow_pending_block(block_hash);
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

    public fun get_latest_block_height(): Option<u64>{
        let store = borrow_store();
        store.latest_block_height
    }

    public fun get_reorg_pending_block_count(): u64{
        let store = borrow_store();
        store.reorg_pending_block_count
    } 
}