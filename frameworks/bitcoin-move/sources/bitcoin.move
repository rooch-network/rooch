// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

module bitcoin_move::bitcoin{
    use std::option::{Self, Option};
    use std::vector;
 
    use moveos_std::timestamp;
    use moveos_std::simple_multimap::SimpleMultiMap;
    use moveos_std::table::{Self, Table};
    use moveos_std::bcs;
    use moveos_std::object::{Self, Object};
    use moveos_std::table_vec::{Self, TableVec};
    use moveos_std::simple_multimap;
    use moveos_std::signer;
    use moveos_std::event;
    
    use rooch_framework::address_mapping;
    use rooch_framework::bitcoin_address::BitcoinAddress;
    
    use bitcoin_move::network;
    use bitcoin_move::types::{Self, Block, Header, Transaction, BlockHeightHash, OutPoint};
    use bitcoin_move::utxo::{Self, UTXOSeal};
    use bitcoin_move::pending_block::{Self, PendingBlock};
    use bitcoin_move::script_buf;
    use bitcoin_move::bbn;

    friend bitcoin_move::genesis;

    /// If the process block failed, we need to stop the system and fix the issue
    const ErrorBlockProcessError:u64 = 1;
    const ErrorBlockAlreadyProcessed:u64 = 2;
    /// The reorg is too deep, we need to stop the system and fix the issue
    const ErrorReorgTooDeep:u64 = 3;
    const ErrorUTXONotExists:u64 = 4;

    const ORDINAL_GENESIS_HEIGHT:u64 = 767430;
    /// https://github.com/bitcoin/bips/blob/master/bip-0034.mediawiki
    const BIP_34_HEIGHT:u64 = 227835;

    const ORDINALS_PAUSE_HEIGHT:u64 = 859001;

    struct UTXONotExistsEvent has copy, drop{
        outpoint: OutPoint,
    }

    struct RepeatCoinbaseTxEvent has copy, drop{
        txid: address,
        vout: u32,
        block_height: u64,
    }

    struct BitcoinBlockStore has key{
        /// The genesis start block
        genesis_block: BlockHeightHash,
        latest_block: Option<BlockHeightHash>,
        /// block hash -> block header
        blocks: Table<address, Header>,
        /// block height -> block hash
        height_to_hash: Table<u64, address>,
        /// block hash -> block height
        hash_to_height: Table<address, u64>,
        /// tx id -> tx
        txs: Table<address, Transaction>,
        /// tx id -> block height
        tx_to_height: Table<address, u64>,
        /// tx id list, we can use this to scan txs
        tx_ids: TableVec<address>,
    }

    public(friend) fun genesis_init(_genesis_account: &signer, genesis_block_height: u64, genesis_block_hash: address){
        let btc_block_store = BitcoinBlockStore{
            genesis_block: types::new_block_height_hash(genesis_block_height, genesis_block_hash),
            latest_block: option::none(),
            blocks: table::new(),
            height_to_hash: table::new(),
            hash_to_height: table::new(),
            txs: table::new(),
            tx_to_height: table::new(),
            tx_ids: table_vec::new(),
        };
        let obj = object::new_named_object(btc_block_store);
        object::to_shared(obj);
    }

    fun borrow_block_store(): &Object<BitcoinBlockStore>{
        let object_id = object::named_object_id<BitcoinBlockStore>();
        object::borrow_object(object_id)
    }

    fun borrow_block_store_mut(): &mut Object<BitcoinBlockStore>{
        let object_id = object::named_object_id<BitcoinBlockStore>();
        object::borrow_mut_object_shared(object_id)
    }

    fun process_block_header(btc_block_store: &mut BitcoinBlockStore, block_height: u64, block_hash: address, block_header: Header){
        //already processed
        assert!(!table::contains(&btc_block_store.hash_to_height, block_hash), ErrorBlockAlreadyProcessed);

        //We have pending block to handle the reorg, this should not happen. 
        //But if it happens, we need to stop the system and fix the issue
        assert!(!table::contains(&btc_block_store.height_to_hash, block_height), ErrorReorgTooDeep);

        table::add(&mut btc_block_store.height_to_hash, block_height, block_hash);
        table::add(&mut btc_block_store.hash_to_height, block_hash, block_height);
        table::add(&mut btc_block_store.blocks, block_hash, block_header);
        btc_block_store.latest_block = option::some(types::new_block_height_hash(block_height, block_hash)); 
    }

    fun process_tx(btc_block_store: &mut BitcoinBlockStore, pblock: &mut Object<PendingBlock>, tx: &Transaction, is_coinbase: bool){
        let block_height = pending_block::block_height(pblock);
        let txid = types::tx_id(tx);
        let repeat_txid = process_utxo(block_height, pblock, tx, is_coinbase);
        
        if (repeat_txid) {
            table::upsert(&mut btc_block_store.txs, txid, *tx);
            table::upsert(&mut btc_block_store.tx_to_height, txid, block_height);
            //We append the repeat txid, the developer want to scan the txs, need to handle the repeat txid
            table_vec::push_back(&mut btc_block_store.tx_ids, txid);
        }else{
            table::add(&mut btc_block_store.txs, txid, *tx);
            table::add(&mut btc_block_store.tx_to_height, txid, block_height);
            table_vec::push_back(&mut btc_block_store.tx_ids, txid);
        };
        if (bbn::is_possible_bbn_transaction(block_height, tx)) {
            bbn::process_bbn_transaction(block_height, tx);
        };
    }

    fun process_utxo(block_height: u64, pending_block: &mut Object<PendingBlock>, tx: &Transaction, is_coinbase: bool) : bool{
        let txinput = types::tx_input(tx);
        let input_utxos = vector::empty();

        let idx = 0;
        let output_seals = simple_multimap::new<u32, UTXOSeal>();
        let sender: Option<address> = option::none<address>();
        let find_sender: bool = false;
        let input_len = vector::length(txinput);
        while (idx < input_len) {
            let txin = vector::borrow(txinput, idx);
            let outpoint = *types::txin_previous_output(txin);
            if (outpoint == types::null_outpoint()) {
                idx = idx + 1;
                continue
            };
            if (utxo::exists_utxo(outpoint)) {
                let object_id = utxo::derive_utxo_id(outpoint);
                let utxo_obj = utxo::take(object_id);
                let utxo_owner = object::owner(&utxo_obj);
                if (!find_sender && utxo_owner != @bitcoin_move) {
                    sender = option::some(utxo_owner);
                    find_sender = true;
                };
                let utxo = utxo::remove(utxo_obj);
                vector::push_back(&mut input_utxos, utxo);
            }else {
                event::emit(UTXONotExistsEvent{
                        outpoint: outpoint,
                });
                //We allow the utxo not exists in the utxo store, because we may not sync the block from genesis
                //But we should not allow the utxo not exists in the mainnet
                if(utxo::check_utxo_input()){
                    abort ErrorUTXONotExists
                };
                let utxo = utxo::mock_utxo(outpoint, 0);
                vector::push_back(&mut input_utxos, utxo);
            };

            idx = idx + 1;
        };
        //temporary pause the ordinals process for the performance reason
        let skip_ordinals = block_height >= ORDINALS_PAUSE_HEIGHT && network::is_mainnet() && rooch_framework::chain_id::is_main();
        if(!skip_ordinals){
            let seal_outs = bitcoin_move::inscription_updater::process_tx(pending_block, tx, &mut input_utxos);
            let seal_outs_len = vector::length(&seal_outs);
            if (seal_outs_len > 0) {
                let seal_out_idx = 0;
                while (seal_out_idx < seal_outs_len) {
                    let seal_out = vector::pop_back(&mut seal_outs);
                    let (output_index, utxo_seal) = utxo::unpack_seal_out(seal_out);
                    simple_multimap::add(&mut output_seals, output_index, utxo_seal);
                    seal_out_idx = seal_out_idx + 1;
                };
            };
        };
    
        // create new utxo
        let repeat_txid = handle_new_utxo(tx, is_coinbase, &mut output_seals, block_height, sender);

        //We do not remove the value from output_seals, for the preformance reason.
        //So, we can not check the output_seals is empty here. just drop it.
        let _ = output_seals;

        vector::for_each(input_utxos, |utxo| {
            utxo::drop(utxo);
        });
        repeat_txid
    }

    fun handle_new_utxo(tx: &Transaction, is_coinbase: bool, output_seals: &mut SimpleMultiMap<u32, UTXOSeal>, block_height: u64, sender: Option<address>) :bool {
        let txid = types::tx_id(tx);
        let txoutput = types::tx_output(tx);
        let idx = 0;
        let txoutput_len = vector::length(txoutput);
        let repeat_txid = false;
        while(idx < txoutput_len){
            let txout = vector::borrow(txoutput, idx);
            let vout = (idx as u32);
            let value = types::txout_value(txout);
            let output_script_buf = types::txout_script_pubkey(txout);
            let is_op_return = script_buf::is_op_return(output_script_buf);
            if (is_coinbase &&  ((block_height < BIP_34_HEIGHT && network::is_mainnet()) || !network::is_mainnet())) {
                let outpoint = types::new_outpoint(txid, vout);
                let utxo_id = utxo::derive_utxo_id(outpoint);
                //Before BIP34, some coinbase txid may be reused, we need to remove the old utxo
                //https://github.com/rooch-network/rooch/issues/2178
                if (object::exists_object(utxo_id)){
                    let utxo_obj = utxo::take(utxo_id);
                    let utxo = utxo::remove(utxo_obj);
                    utxo::drop(utxo);
                    event::emit(RepeatCoinbaseTxEvent{
                        txid: txid,
                        vout: vout,
                        block_height: block_height,
                    });
                    repeat_txid = true;
                };
            };
            //We should not create UTXO object for OP_RETURN output
            if(is_op_return){
                idx = idx + 1;
                continue
            };
            let utxo_obj = utxo::new(txid, vout, value);
            let utxo = object::borrow_mut(&mut utxo_obj);
            let seal_index = (idx as u32);
            if(simple_multimap::contains_key(output_seals, &seal_index)){
                let utxo_seals = simple_multimap::borrow_mut(output_seals, &seal_index);
                let j = 0;
                let utxo_seals_len = vector::length(utxo_seals);
                while(j < utxo_seals_len){
                    let utxo_seal = vector::pop_back(utxo_seals);
                    utxo::add_seal_internal(utxo, utxo_seal);
                    j = j + 1;
                };
            };
            let receiver = types::txout_object_address(txout);
            utxo::transfer(utxo_obj, sender, receiver);
            //Auto create address mapping, we ensure when UTXO object create, the address mapping is recored
            let bitcoin_address_opt = types::txout_address(txout);
            bind_bitcoin_address(receiver, bitcoin_address_opt);
            idx = idx + 1;
        };
        repeat_txid
    }


    /// The the sequencer submit a new Bitcoin block to execute
    /// This function is a system function, is the execute_l1_block entry point
    fun execute_l1_block(block_height: u64, block_hash: address, block_bytes: vector<u8>){
        let btc_block_store_obj = borrow_block_store();
        let btc_block_store = object::borrow(btc_block_store_obj);
        assert!(!table::contains(&btc_block_store.height_to_hash, block_height), ErrorReorgTooDeep);
        let block = bcs::from_bytes<Block>(block_bytes);
        let block_header = types::header(&block);
        let time = types::time(block_header);
        if(pending_block::add_pending_block(block_height, block_hash, block)){
            //We directly update the global time do not wait the pending block to be confirmed
            //The reorg do not affect the global time
            let timestamp_seconds = (time as u64);
            let module_signer = signer::module_signer<BitcoinBlockStore>();
            timestamp::try_update_global_time(&module_signer, timestamp::seconds_to_milliseconds(timestamp_seconds));
        };
    }

    /// This is the execute_l1_tx entry point
    fun execute_l1_tx(block_hash: address, txid: address){
        let btc_block_store_obj = borrow_block_store_mut();
        let btc_block_store = object::borrow_mut(btc_block_store_obj);
        let inprocess_block = pending_block::process_pending_tx(block_hash, txid);
        let block_height = pending_block::inprocess_block_height(&inprocess_block);
        let tx = *pending_block::inprocess_block_tx(&inprocess_block);
        let pblock = pending_block::inprocess_block_pending_block(&mut inprocess_block);
        let is_coinbase = types::is_coinbase_tx(&tx);
        process_tx(btc_block_store, pblock, &tx, is_coinbase);
        if(is_coinbase){
            let header = pending_block::finish_pending_block(inprocess_block);
            process_block_header(btc_block_store, block_height, block_hash, header);
        }else{
            pending_block::finish_pending_tx(inprocess_block);
        };
    }

    public fun get_tx(txid: address): Option<Transaction>{
        let btc_block_store_obj = borrow_block_store();
        let btc_block_store = object::borrow(btc_block_store_obj);
        if(table::contains(&btc_block_store.txs, txid)){
            option::some(*table::borrow(&btc_block_store.txs, txid))
        }else{
            option::none()
        }
    }

    public fun get_tx_height(txid: address): Option<u64>{
        let btc_block_store_obj = borrow_block_store();
        let btc_block_store = object::borrow(btc_block_store_obj);
        if(table::contains(&btc_block_store.tx_to_height, txid)){
            option::some(*table::borrow(&btc_block_store.tx_to_height, txid))
        }else{
            option::none()
        }
    }

    /// Get block via block_hash
    public fun get_block(block_hash: address): Option<Header>{
        let btc_block_store_obj = borrow_block_store();
        let btc_block_store = object::borrow(btc_block_store_obj);
        if(table::contains(&btc_block_store.blocks, block_hash)){
            option::some(*table::borrow(&btc_block_store.blocks, block_hash))
        }else{
            option::none()
        }
    }

    public fun get_block_height(block_hash: address): Option<u64>{
        let btc_block_store_obj = borrow_block_store();
        let btc_block_store = object::borrow(btc_block_store_obj);
        if(table::contains(&btc_block_store.hash_to_height, block_hash)){
            option::some(*table::borrow(&btc_block_store.hash_to_height, block_hash))
        }else{
            option::none()
        }
    }

    /// Get block hash via block_height
    public fun get_block_hash_by_height(block_height: u64): Option<address>{
        let btc_block_store_obj = borrow_block_store();
        let btc_block_store = object::borrow(btc_block_store_obj);
        if(table::contains(&btc_block_store.height_to_hash, block_height)){
            let block_hash = *table::borrow(&btc_block_store.height_to_hash, block_height);
            option::some(block_hash)
        }else{
            option::none()
        }
    }

    /// Get block via block_height
    public fun get_block_by_height(block_height: u64): Option<Header>{
        let btc_block_store_obj = borrow_block_store();
        let btc_block_store = object::borrow(btc_block_store_obj);
        if(table::contains(&btc_block_store.height_to_hash, block_height)){
            let block_hash = *table::borrow(&btc_block_store.height_to_hash, block_height);
            option::some(*table::borrow(&btc_block_store.blocks, block_hash))
        }else{
            option::none()
        }
    }

    public fun get_genesis_block(): BlockHeightHash {
        let btc_block_store_obj = borrow_block_store();
        let btc_block_store = object::borrow(btc_block_store_obj);
        btc_block_store.genesis_block
    }

    /// Get latest block height
    public fun get_latest_block(): Option<BlockHeightHash> {
        let btc_block_store_obj = borrow_block_store();
        let btc_block_store = object::borrow(btc_block_store_obj);
        btc_block_store.latest_block
    }

    /// Get the bitcoin time in seconds
    public fun get_bitcoin_time(): u32 {
        let btc_block_store_obj = borrow_block_store();
        let btc_block_store = object::borrow(btc_block_store_obj);
        let latest_block = *&btc_block_store.latest_block;
        if(option::is_some(&latest_block)){
            let latest_block = option::destroy_some(latest_block);
            let (_block_height, block_hash) = types::unpack_block_height_hash(latest_block);
            let header = table::borrow(&btc_block_store.blocks, block_hash);
            types::time(header)
        }else{
            // Get the genesis block time
            (timestamp::now_seconds() as u32)
        }
    }

    fun bind_bitcoin_address(rooch_address: address, bitcoin_address_opt: Option<BitcoinAddress>) {
        //Auto create address mapping if not exist
        if(option::is_some(&bitcoin_address_opt)) {
            let bitcoin_address = option::extract(&mut bitcoin_address_opt);
            let bitcoin_move_signer = moveos_std::signer::module_signer<BitcoinBlockStore>();
            address_mapping::bind_bitcoin_address_by_system(&bitcoin_move_signer, rooch_address, bitcoin_address);
        };
    }

    public fun contains_header(block_header : &Header) : bool {
        let block_hash = types::header_to_hash(block_header);
        let block = get_block(block_hash);
        option::is_some(&block)
    }

    #[test_only]
    public fun execute_l1_block_for_test(block_height: u64, block: Block){
        let block_hash = types::header_to_hash(types::header(&block));
        let block_bytes = bcs::to_bytes(&block);
        execute_l1_block(block_height, block_hash, block_bytes);
        // We directly conform the txs for convenience test
        let (_, txs) = types::unpack_block(block);
        let coinbase_tx = vector::remove(&mut txs, 0);
        vector::for_each(txs, |tx| {
            let txid = types::tx_id(&tx);
            execute_l1_tx(block_hash, txid);
        });
        //process coinbase tx last
        execute_l1_tx(block_hash, types::tx_id(&coinbase_tx));
    }


    #[test_only]
    public fun add_latest_block(block_height: u64, block_hash: address){
        let btc_block_store_obj = borrow_block_store_mut();
        let btc_block_store = object::borrow_mut(btc_block_store_obj);
        btc_block_store.latest_block = option::some(types::new_block_height_hash(block_height, block_hash))
    }

}