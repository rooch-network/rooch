// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

module bitcoin_move::bitcoin{
    use std::option::{Self, Option};
    use std::vector;
    use std::string::{Self, String};
    use moveos_std::address::to_string;
    use moveos_std::event_queue;

    use moveos_std::timestamp;
    use moveos_std::simple_multimap::SimpleMultiMap;
    use moveos_std::type_info;
    use moveos_std::table::{Self, Table};
    use moveos_std::bcs;
    use moveos_std::object::{Self, Object};
    use moveos_std::table_vec::{Self, TableVec};
    use moveos_std::simple_multimap;
    use moveos_std::signer;
    use moveos_std::event;
    
    use rooch_framework::chain_id;
    use rooch_framework::address_mapping;
    use rooch_framework::bitcoin_address::BitcoinAddress;
    
    use bitcoin_move::network;
    use bitcoin_move::types::{Self, Block, Header, Transaction, BlockHeightHash};
    use bitcoin_move::ord::{Self, Inscription,Flotsam, SatPoint};
    use bitcoin_move::utxo::{Self, UTXOSeal};
    use bitcoin_move::pending_block;

    friend bitcoin_move::genesis;

    /// If the process block failed, we need to stop the system and fix the issue
    const ErrorBlockProcessError:u64 = 1;
    const ErrorBlockAlreadyProcessed:u64 = 2;
    /// The reorg is too deep, we need to stop the system and fix the issue
    const ErrorReorgTooDeep:u64 = 3;

    const ORDINAL_GENESIS_HEIGHT:u64 = 767430;
    /// https://github.com/bitcoin/bips/blob/master/bip-0034.mediawiki
    const BIP_34_HEIGHT:u64 = 227835;

    struct TxProgressErrorLogEvent has copy, drop{
        txid: address,
        message: String,
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


    struct TransferUTXOEvent has drop, store, copy {
        txid: address,
        sender: Option<address>,
        receiver: address,
        value: u64
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

    fun process_tx(btc_block_store: &mut BitcoinBlockStore, tx: &Transaction, block_height: u64): vector<Flotsam>{
        let flotsams = process_utxo(tx, block_height);
        let txid = types::tx_id(tx);
        table::add(&mut btc_block_store.txs, txid, *tx);
        table::add(&mut btc_block_store.tx_to_height, txid, block_height);
        table_vec::push_back(&mut btc_block_store.tx_ids, txid);
        flotsams
    }

    fun process_coinbase_tx(btc_block_store: &mut BitcoinBlockStore, tx: &Transaction, flotsams: vector<Flotsam>, block_height: u64){
        let repeat_txid = process_coinbase_utxo(tx, flotsams, block_height);
        let txid = types::tx_id(tx);
        if (repeat_txid) {
            table::upsert(&mut btc_block_store.txs, txid, *tx);
            table::upsert(&mut btc_block_store.tx_to_height, txid, block_height);
            //We append the repeat txid, the developer want to scan the txs, need to handle the repeat txid
            table_vec::push_back(&mut btc_block_store.tx_ids, txid);
        }else{
            table::add(&mut btc_block_store.txs, txid, *tx);
            table::add(&mut btc_block_store.tx_to_height, txid, block_height);
            table_vec::push_back(&mut btc_block_store.tx_ids, txid);
        }
    }

    fun process_utxo(tx: &Transaction, block_height: u64): vector<Flotsam>{
        let txinput = types::tx_input(tx);
        let flotsams = vector::empty();

        let previous_outputs = vector::empty();
        vector::for_each(*txinput, |txin| {
            let outpoint = *types::txin_previous_output(&txin);
            vector::push_back(&mut previous_outputs, outpoint);
        });
        let input_utxo_values = vector::empty();
        vector::for_each(previous_outputs, |output| {
            let utxo_value = if(utxo::exists_utxo(output)){
                let utxo = utxo::borrow_utxo(output);
                utxo::value(object::borrow(utxo))
            } else {
                0
            };
            vector::push_back(&mut input_utxo_values, utxo_value);
        });

        let idx = 0;
        let output_seals = simple_multimap::new<u32, UTXOSeal>();
        let need_process_oridinal = need_process_oridinals(block_height);
        let sender: Option<address> = option::none<address>();
        let find_sender: bool = false;
        while (idx < vector::length(txinput)) {
            let txin = vector::borrow(txinput, idx);
            let outpoint = *types::txin_previous_output(txin);
            if (utxo::exists_utxo(outpoint)) {
                let object_id = utxo::derive_utxo_id(outpoint);
                let utxo_obj = utxo::take(object_id);
                let utxo_owner = object::owner(&utxo_obj);
                if (!find_sender && utxo_owner != @bitcoin_move) {
                    sender = option::some(utxo_owner);
                    find_sender = true;
                };
                if(need_process_oridinal) {
                    let (sat_points, utxo_flotsams) = ord::spend_utxo(&mut utxo_obj, tx, input_utxo_values, idx);
                    handle_sat_point(sat_points, &mut output_seals);
                    vector::append(&mut flotsams, utxo_flotsams);
                };

                let seals = utxo::remove(utxo_obj);
                //The seals should be empty after utxo is spent
                simple_multimap::destroy_empty(seals);
            }else {
                event::emit(TxProgressErrorLogEvent{
                        txid: types::tx_id(tx),
                        message: string::utf8(b"utxo not exists"),
                });
                std::debug::print(&outpoint);
                //We allow the utxo not exists in the utxo store, because we may not sync the block from genesis
                //But we should not allow the utxo not exists in the mainnet
                if(chain_id::is_main()){
                    abort ErrorBlockProcessError
                };
            };

            idx = idx + 1;
        };

        // Transfer and inscribe may happen at the same transaction
        if(need_process_oridinal) {
            let sat_points = ord::process_transaction(tx, input_utxo_values);
            let idx = 0;
            let protocol = type_info::type_name<Inscription>();
            let sat_points_len = vector::length(&sat_points);
            while (idx < sat_points_len) {
                let sat_point = vector::pop_back(&mut sat_points);
                let output_index = ord::sat_point_output_index(&sat_point);
                let seal_object_id = ord::sat_point_object_id(&sat_point);
                let utxo_seal = utxo::new_utxo_seal(protocol, seal_object_id);
                simple_multimap::add(&mut output_seals, output_index, utxo_seal);
                idx = idx + 1;
            };
        };

        // create new utxo
        handle_new_utxo(tx, &mut output_seals, false, block_height, sender);

        simple_multimap::drop(output_seals);
        flotsams
    }

    fun process_coinbase_utxo(tx: &Transaction, flotsams: vector<Flotsam>, block_height: u64) : bool{
        let output_seals = simple_multimap::new<u32, UTXOSeal>();
        if(need_process_oridinals(block_height)) {
            let sat_points = ord::handle_coinbase_tx(tx, flotsams, block_height);
            handle_sat_point(sat_points, &mut output_seals);
        };

        // create new utxo
        let repeat_txid = handle_new_utxo(tx, &mut output_seals, true, block_height, option::none());
        simple_multimap::drop(output_seals);
        repeat_txid
    }

    fun handle_sat_point(sat_points: vector<SatPoint>, output_seals: &mut SimpleMultiMap<u32, UTXOSeal>) {
        if (!vector::is_empty(&sat_points)) {
            let protocol = type_info::type_name<Inscription>();
            let j = 0;
            let sat_points_len = vector::length(&sat_points);
            while (j < sat_points_len) {
                let sat_point = vector::pop_back(&mut sat_points);
                let (output_index, _offset, object_id) = ord::unpack_sat_point(sat_point);
                let utxo_seal = utxo::new_utxo_seal(protocol, object_id);
                simple_multimap::add(output_seals, output_index, utxo_seal);
                j = j + 1;
            };
        };
        // output_seals
    }

    fun handle_new_utxo(tx: &Transaction, output_seals: &mut SimpleMultiMap<u32, UTXOSeal>, is_coinbase: bool, block_height: u64, sender: Option<address>) :bool {
        let txid = types::tx_id(tx);
        let txoutput = types::tx_output(tx);
        let idx = 0;
        let txoutput_len = vector::length(txoutput);
        let repeat_txid = false;
        while(idx < txoutput_len){
            let txout = vector::borrow(txoutput, idx);
            let vout = (idx as u32);
            let value = types::txout_value(txout);
            if (is_coinbase &&  ((block_height < BIP_34_HEIGHT && network::is_mainnet()) || !network::is_mainnet())) {
                let outpoint = types::new_outpoint(txid, vout);
                let utxo_id = utxo::derive_utxo_id(outpoint);
                //Before BIP34, some coinbase txid may be reused, we need to remove the old utxo
                //https://github.com/rooch-network/rooch/issues/2178
                if (object::exists_object(utxo_id)){
                    let utxo = utxo::take(utxo_id);
                    let seals = utxo::remove(utxo);
                    simple_multimap::destroy_empty(seals);
                    event::emit(RepeatCoinbaseTxEvent{
                        txid: txid,
                        vout: vout,
                        block_height: block_height,
                    });
                    repeat_txid = true;
                };
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
                    utxo::add_seal(utxo, utxo_seal);
                    j = j + 1;
                };
            };
            let owner_address = types::txout_object_address(txout);
            utxo::transfer(utxo_obj, owner_address);
            if (owner_address != @bitcoin_move){
                event_queue::emit(to_string(&owner_address), TransferUTXOEvent{
                    txid,
                    sender,
                    receiver: owner_address,
                    value
                })
            };
            if (option::is_some(&sender)){
                let sender_address = option::extract(&mut sender);
                event_queue::emit(to_string(&sender_address), TransferUTXOEvent{
                    txid,
                    sender,
                    receiver: owner_address,
                    value
                });
            };


            //Auto create address mapping, we ensure when UTXO object create, the address mapping is recored
            let bitcoin_address_opt = types::txout_address(txout);
            bind_bitcoin_address(owner_address, bitcoin_address_opt);
            idx = idx + 1;
        };
        repeat_txid
    }


    /// The the sequencer submit a new Bitcoin block to execute
    /// This function is a system function, is the execute_l1_block entry point
    fun execute_l1_block(block_height: u64, block_hash: address, block_bytes: vector<u8>){
        let block = bcs::from_bytes<Block>(block_bytes);
        let block_header = types::header(&block);
        let time = types::time(block_header);
        pending_block::add_pending_block(block_height, block_hash, block);    
        //We directly update the global time do not wait the pending block to be confirmed
        //The reorg do not affect the global time
        let timestamp_seconds = (time as u64);
        let module_signer = signer::module_signer<BitcoinBlockStore>();
        timestamp::try_update_global_time(&module_signer, timestamp::seconds_to_milliseconds(timestamp_seconds));      
    }

    /// This is the execute_l1_tx entry point
    fun execute_l1_tx(block_hash: address, txid: address){
        let btc_block_store_obj = borrow_block_store_mut();
        let btc_block_store = object::borrow_mut(btc_block_store_obj);
        let inprocess_block = pending_block::process_pending_tx(block_hash, txid);
        let block_height = pending_block::inprocess_block_height(&inprocess_block);
        let tx = pending_block::inprocess_block_tx(&inprocess_block);
        if(types::is_coinbase_tx(tx)){
            let flotsams = pending_block::inprocess_block_flotsams(&inprocess_block);
            process_coinbase_tx(btc_block_store, tx, flotsams, block_height);
            let header = pending_block::finish_pending_block(inprocess_block);
            process_block_header(btc_block_store, block_height, block_hash, header);
        }else{
            let tx_flotsams = process_tx(btc_block_store, tx, block_height);
            let flotsams = pending_block::inprocess_block_flotsams_mut(&mut inprocess_block);
            vector::append(flotsams, tx_flotsams);
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

    fun need_process_oridinals(block_height: u64) : bool {
        if(network::is_mainnet()){
            block_height >= ORDINAL_GENESIS_HEIGHT
        }else{
            true
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

}