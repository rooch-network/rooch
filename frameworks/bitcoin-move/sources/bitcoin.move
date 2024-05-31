// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

module bitcoin_move::bitcoin{
    use std::option::{Self, Option};
    use std::vector;
    use std::string::{Self, String};

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
    use bitcoin_move::types::{Self, Block, Header, Transaction};
    use bitcoin_move::ord::{Self, Inscription,Flotsam, SatPoint};
    use bitcoin_move::utxo::{Self, UTXOSeal};

    friend bitcoin_move::genesis;

    /// If the process block failed, we need to stop the system and fix the issue
    const ErrorBlockProcessError:u64 = 1;
    const ErrorBlockAlreadyProcessed:u64 = 2;

    const ORDINAL_GENESIS_HEIGHT:u64 = 767430;

    struct TxProgressErrorLogEvent has copy, drop{
        txid: address,
        message: String,
    }

    struct BitcoinBlockStore has key{
        /// The genesis start block height
        genesis_block_height: u64,
        latest_block_height: Option<u64>,
        /// block hash -> block header
        blocks: Table<address, Header>,
        /// block height -> block hash
        height_to_hash: Table<u64, address>,
        /// block hash -> block height
        hash_to_height: Table<address, u64>,
        /// tx id -> tx
        txs: Table<address, Transaction>,
        /// tx id -> tx
        tx_to_height: Table<address, u64>,
        /// tx id list, we can use this to scan txs
        tx_ids: TableVec<address>,
    }

    public(friend) fun genesis_init(_genesis_account: &signer, genesis_block_height: u64){
        let btc_block_store = BitcoinBlockStore{
            genesis_block_height,
            latest_block_height: option::none(),
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

    fun process_block(btc_block_store_obj: &mut Object<BitcoinBlockStore>, block_height: u64, block_hash: address, block_bytes: vector<u8>):u32{
        let btc_block_store = object::borrow_mut(btc_block_store_obj);
        //already processed
        assert!(!table::contains(&btc_block_store.hash_to_height, block_hash), ErrorBlockAlreadyProcessed);

        let block = bcs::from_bytes<Block>(block_bytes);
        process_txs(btc_block_store, &block, block_height);
        let block_header = types::header(&block);

        if(table::contains(&btc_block_store.height_to_hash, block_height)){
            //TODO handle reorg
        };
        let time = types::time(block_header);
        table::add(&mut btc_block_store.height_to_hash, block_height, block_hash);
        table::add(&mut btc_block_store.hash_to_height, block_hash, block_height);
        table::add(&mut btc_block_store.blocks, block_hash, *block_header);
        btc_block_store.latest_block_height = option::some(block_height);
        time 
    }
 
    fun process_txs(btc_block_store: &mut BitcoinBlockStore, block:&Block, block_height: u64){
        let txdata = types::txdata(block);
        let idx = 0;
        let coinbase_tx_idx = 0;
        let flotsams = vector::empty();
        let tx_len = vector::length(txdata);
        while(idx < tx_len){
            let tx = vector::borrow(txdata, idx);

            let is_coinbase = is_coinbase_tx(tx);
            if(is_coinbase) {
                coinbase_tx_idx = idx;
            } else {
                let tx_flotsams = process_tx(btc_block_store, tx, block_height);
                vector::append(&mut flotsams, tx_flotsams);
            };
            idx = idx + 1;
        };

        // handle coinbase tx
        let coinbase_tx = vector::borrow(txdata, coinbase_tx_idx);
        process_coinbase_tx(btc_block_store, coinbase_tx, flotsams, block_height)
    }

    fun is_coinbase_tx(tx: &Transaction): bool {
        let txinput = types::tx_input(tx);
        let is_coinbase = if(vector::length(txinput) > 0) {
            let first_input = vector::borrow(txinput, 0);
            let previous_output = types::txin_previous_output(first_input);
            types::is_null_outpoint(previous_output)
        } else {
            false
        };
        is_coinbase
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
        process_coinbase_utxo(tx, flotsams, block_height);
        let txid = types::tx_id(tx);
        table::add(&mut btc_block_store.txs, txid, *tx);
        table_vec::push_back(&mut btc_block_store.tx_ids, txid);
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
        while (idx < vector::length(txinput)) {
            let txin = vector::borrow(txinput, idx);
            let outpoint = *types::txin_previous_output(txin);
            if (utxo::exists_utxo(outpoint)) {
                let object_id = utxo::derive_utxo_id(outpoint);
                let (_owner, utxo_obj) = utxo::take(object_id);
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
        handle_new_utxo(tx, &mut output_seals);

        simple_multimap::drop(output_seals);
        flotsams
    }

    fun process_coinbase_utxo(tx: &Transaction, flotsams: vector<Flotsam>, block_height: u64){
        let output_seals = simple_multimap::new<u32, UTXOSeal>();
        if(need_process_oridinals(block_height)) {
            let sat_points = ord::handle_coinbase_tx(tx, flotsams, block_height);
            handle_sat_point(sat_points, &mut output_seals);
        };

        // create new utxo
        handle_new_utxo(tx, &mut output_seals);
        simple_multimap::drop(output_seals);
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

    fun handle_new_utxo(tx: &Transaction, output_seals: &mut SimpleMultiMap<u32, UTXOSeal>) {
        let txid = types::tx_id(tx);
        let txoutput = types::tx_output(tx);
        let idx = 0;
        let txoutput_len = vector::length(txoutput);
        while(idx < txoutput_len){
            let txout = vector::borrow(txoutput, idx);
            let vout = (idx as u32);
            let value = types::txout_value(txout);
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

            //Auto create address mapping, we ensure when UTXO object create, the address mapping is recored
            let bitcoin_address_opt = types::txout_address(txout);
            bind_bitcoin_address(owner_address, bitcoin_address_opt);
            idx = idx + 1;
        };
    }


    /// The relay server submit a new Bitcoin block to the light client.
    fun submit_new_block(block_height: u64, block_hash: address, block_bytes: vector<u8>){
        let btc_block_store_obj = borrow_block_store_mut();
        let time = process_block(btc_block_store_obj, block_height, block_hash, block_bytes);

        let timestamp_seconds = (time as u64);
        let module_signer = signer::module_signer<BitcoinBlockStore>();
        timestamp::try_update_global_time(&module_signer, timestamp::seconds_to_milliseconds(timestamp_seconds));      
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
        if(table::contains(&btc_block_store.txs, txid)){
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

    public fun get_genesis_block_height(): u64 {
        let btc_block_store_obj = borrow_block_store();
        let btc_block_store = object::borrow(btc_block_store_obj);
        btc_block_store.genesis_block_height
    }

    /// Get latest block height
    public fun get_latest_block_height(): Option<u64> {
        let btc_block_store_obj = borrow_block_store();
        let btc_block_store = object::borrow(btc_block_store_obj);
        btc_block_store.latest_block_height
    }

    /// Get the bitcoin time, if the latest block is not exist, return 0 
    public fun get_bitcoin_time(): u32 {
        let btc_block_store_obj = borrow_block_store();
        let btc_block_store = object::borrow(btc_block_store_obj);
        let latest_block_height = btc_block_store.latest_block_height;
        if(option::is_some(&latest_block_height)){
            let latest_block_height = option::destroy_some(latest_block_height);
            let block_hash = *table::borrow(&btc_block_store.height_to_hash, latest_block_height);
            let header = table::borrow(&btc_block_store.blocks, block_hash);
            types::time(header)
        }else{
            0u32
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

    #[test_only]
    public fun submit_block_for_test(block_height: u64, block_hash: address, block_header: &Header){
        let btc_block_store_obj = borrow_block_store_mut();
        let btc_block_store = object::borrow_mut(btc_block_store_obj);
        let time = types::time(block_header);
        table::add(&mut btc_block_store.height_to_hash, block_height, block_hash);
        table::add(&mut btc_block_store.hash_to_height, block_hash, block_height);
        table::add(&mut btc_block_store.blocks, block_hash, *block_header);

        let curr_latest_height = option::get_with_default(&btc_block_store.latest_block_height, 0);
        if (block_height > curr_latest_height) {
            btc_block_store.latest_block_height = option::some(block_height);
        };

        let timestamp_seconds = (time as u64);
        let module_signer = signer::module_signer<BitcoinBlockStore>();
        timestamp::try_update_global_time(&module_signer, timestamp::seconds_to_milliseconds(timestamp_seconds));    
    }
}