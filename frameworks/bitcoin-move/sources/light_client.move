// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

module bitcoin_move::light_client{
    use std::option::{Self, Option};
    use std::vector;
    use std::string::{String};
    use moveos_std::type_info;
    use moveos_std::table::{Self, Table};
    use moveos_std::bcs;
    use moveos_std::object::{Self, Object};
    use moveos_std::table_vec::{Self, TableVec};
    use moveos_std::simple_multimap;
    use moveos_std::signer;
    use rooch_framework::timestamp;
    use bitcoin_move::types::{Self, Block, Header, Transaction};
    use bitcoin_move::ord::{Self, Inscription, bind_multichain_address};
    use bitcoin_move::utxo::{Self, UTXOSeal};
    

    friend bitcoin_move::genesis;

    const ErrorBlockNotFound:u64 = 1;
    const ErrorBlockAlreadyProcessed:u64 = 2;

    struct TxProgressErrorLogEvent has copy, drop{
        txid: address,
        message: String,
    }

    struct BitcoinBlockStore has key{
        latest_block_height: Option<u64>,
        /// block hash -> block header
        blocks: Table<address, Header>,
        /// block height -> block hash
        height_to_hash: Table<u64, address>,
        /// block hash -> block height
        hash_to_height: Table<address, u64>,
        /// tx id -> tx
        txs: Table<address, Transaction>,
        /// tx id list, we can use this to scan txs
        tx_ids: TableVec<address>,
    }

    public(friend) fun genesis_init(_genesis_account: &signer){
        let btc_block_store = BitcoinBlockStore{
            latest_block_height: option::none(),
            blocks: table::new(),
            height_to_hash: table::new(),
            hash_to_height: table::new(),
            txs: table::new(),
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
        process_txs(btc_block_store, &block); 
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
 
    fun process_txs(btc_block_store: &mut BitcoinBlockStore, block:&Block){
        let txdata = types::txdata(block);
        let idx = 0;
        while(idx < vector::length(txdata)){
            let tx = vector::borrow(txdata, idx);
            process_tx(btc_block_store, tx);
            idx = idx + 1;
        }

        // handle coinbase tx
    }

    fun process_tx(btc_block_store: &mut BitcoinBlockStore, tx: &Transaction){
        process_utxo(tx);
        let txid = types::tx_id(tx);
        table::add(&mut btc_block_store.txs, txid, *tx);
        table_vec::push_back(&mut btc_block_store.tx_ids, txid);
    }

    fun process_utxo(tx: &Transaction){
        let txid = types::tx_id(tx);
        let txinput = types::tx_input(tx);

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
        while(idx < vector::length(txinput)){
            let txin = vector::borrow(txinput, idx);
            let outpoint = *types::txin_previous_output(txin);
            if(utxo::exists_utxo(outpoint)){
                let object_id = utxo::derive_utxo_id(outpoint);
                let (_owner, utxo_obj) = utxo::take(object_id);
                let seal_points = ord::spend_utxo(&mut utxo_obj, tx, input_utxo_values, idx);

                if(!vector::is_empty(&seal_points)){
                    let protocol = type_info::type_name<Inscription>();
                    let j = 0;
                    let seal_points_len = vector::length(&seal_points);
                    while(j < seal_points_len){
                        let seal_point = vector::pop_back(&mut seal_points);
                        let (output_index, _offset, _object_id) = utxo::unpack_seal_point(seal_point);
                        let utxo_seal = utxo::new_utxo_seal(protocol, seal_point);
                        simple_multimap::add(&mut output_seals, output_index, utxo_seal);
                        j = j + 1;
                    };
                };

                let seals = utxo::remove(utxo_obj);
                //The seals should be empty after utxo is spent
                simple_multimap::destroy_empty(seals);
            }else{
                //We allow the utxo not exists in the utxo store, because we may not sync the block from genesis
            };

            idx = idx + 1;
        };

        //If a utxo is spend seal assets, it should not seal new assets
        if(simple_multimap::length(&output_seals) == 0){
            let seal_points = ord::process_transaction(tx, input_utxo_values);
            let idx = 0;
            let protocol = type_info::type_name<Inscription>();
            let seal_points_len = vector::length(&seal_points);
            while(idx < seal_points_len){
                let seal_point = vector::pop_back(&mut seal_points);
                let output_index = utxo::seal_point_output_index(&seal_point);
                let utxo_seal = utxo::new_utxo_seal(protocol, seal_point);
                simple_multimap::add(&mut output_seals, output_index, utxo_seal);
                idx = idx + 1;
            };
        };

        // create new utxo
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
            if(simple_multimap::contains_key(&output_seals, &seal_index)){
                let utxo_seals = simple_multimap::borrow_mut(&mut output_seals, &seal_index);
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

            //Auto create address mapping if not exist
            let bitcoin_address_opt = types::txout_address(txout);
            bind_multichain_address(owner_address, bitcoin_address_opt);

            idx = idx + 1;
        };
        simple_multimap::drop(output_seals);
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

    /// Get latest block height
    public fun get_latest_block_height(): Option<u64> {
        let btc_block_store_obj = borrow_block_store();
        let btc_block_store = object::borrow(btc_block_store_obj);
        btc_block_store.latest_block_height
    } 
    
}