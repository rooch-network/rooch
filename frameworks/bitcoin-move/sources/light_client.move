// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

module bitcoin_move::light_client{

    use std::option::{Self, Option};
    use std::vector;
    use std::string::{String};
    use moveos_std::simple_map;
    use moveos_std::object::ObjectID;
    use moveos_std::type_info;
    use moveos_std::table::{Self, Table};
    use moveos_std::bcs;
    use moveos_std::object::{Self, Object};
    use moveos_std::table_vec::{Self, TableVec};
    use moveos_std::simple_multimap;
    use moveos_std::signer;
    use rooch_framework::timestamp;
    use bitcoin_move::types::{Self, Block, Header, Transaction, OutPoint, outpoint_txid, outpoint_vout, new_outpoint};
    use bitcoin_move::ord::{Self, Inscription, bind_multichain_address, SatPoint, inscriptions_on_output,
        satpoint_offset, new_satpoint, update_inscription_index, new_satpoint_mapping, satpoint_mapping,
        remove_inscription_index
    };
    use bitcoin_move::utxo::{Self, UTXOSeal, utxo_range_vout, utxo_range_txid, utxo_range_range};
    

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

    struct BitcoinUTXOStore has key{
        /// The next tx index to be processed
        next_tx_index: u64,
        /// outpoint -> txout
        utxo: Table<OutPoint, ObjectID>,
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

        let btc_utxo_store = BitcoinUTXOStore{
            next_tx_index: 0,
            utxo: table::new(),
        };
        let obj = object::new_named_object(btc_utxo_store);
        object::to_shared(obj);
    }

    fun process_block(btc_block_store_obj: &mut Object<BitcoinBlockStore>, block_height: u64, block_hash: address, block_bytes: vector<u8>):u32{
        
        let btc_block_store = object::borrow_mut(btc_block_store_obj);
        //already processed
        assert!(!table::contains(&btc_block_store.hash_to_height, block_hash), ErrorBlockAlreadyProcessed);

        let block = bcs::from_bytes<Block>(block_bytes);
        validate_block(btc_block_store, block_height, block_hash, &block);
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

    fun validate_block(_btc_block_store: &BitcoinBlockStore, _block_height: u64, _block_hash: address, _block: &Block){
        //TODO validate the block via bitcoin consensus
        // validate prev block hash
        // validate block hash
        // validate block nonce
        //TODO validate txid
    }

    fun process_txs(btc_block_store: &mut BitcoinBlockStore, block:&Block){
        let txdata = types::txdata(block);
        let idx = 0;
        while(idx < vector::length(txdata)){
            let tx = vector::borrow(txdata, idx);
            process_tx(btc_block_store, tx);
            idx = idx + 1;
        }
    }

    fun process_tx(btc_block_store: &mut BitcoinBlockStore, tx: &Transaction){
        let txid = types::tx_id(tx);
        table::add(&mut btc_block_store.txs, txid, *tx);
        table_vec::push_back(&mut btc_block_store.tx_ids, txid);
    }

    fun process_utxo(btc_utxo_store: &mut BitcoinUTXOStore, tx: &Transaction){
        let txid = types::tx_id(tx);
        let txinput = types::tx_input(tx);
        let txoutput = types::tx_output(tx);
        let output_seals = simple_multimap::new<u64, UTXOSeal>();
        let previous_outputs = vector[];
        // let output_satpoint_mapping = vector[];
        vector::for_each(*txinput, |txin| {
            let outpoint = *types::txin_previous_output(&txin);
            vector::push_back(&mut previous_outputs, outpoint);
        });


        // match utxo
        let previous_output_idx = 0;
        let previous_output_next_offset :u64 = 0;
        let previous_output_len = vector::length(&previous_outputs);
        let output_idx = 0;
        let output_len = vector::length(txoutput);
        let output_utxo_mapping = vector[];
        while(output_idx < output_len){
            let input_to_output_mapping = vector[];

            let txout = vector::borrow(txoutput, output_idx);
            // let vout = (output_idx as u32);
            // let outpoint = types::new_outpoint(txid, vout);
            let output_value = types::txout_value(txout);

            let utxo_value_accumulator: u64 = 0;
            while(previous_output_idx < previous_output_len){
                let previous_out = vector::borrow(&previous_outputs, previous_output_idx);
                let previous_txid = outpoint_txid(previous_out);
                let previous_vout = outpoint_vout(previous_out);

                let previous_utxo_value = 0;
                let previous_utxo_avaliable_value = 0;
                if(table::contains(&btc_utxo_store.utxo, *previous_out)){
                    let previous_utxo = utxo::borrow_utxo(previous_txid, previous_vout);
                    previous_utxo_value = utxo::value(object::borrow(previous_utxo));
                    previous_utxo_avaliable_value = if (previous_utxo_value >= previous_output_next_offset) {
                        previous_utxo_value - previous_output_next_offset
                    } else {
                        0u64
                    };

                    utxo_value_accumulator = utxo_value_accumulator + previous_utxo_avaliable_value;
                } else {
                    //We allow the utxo not exists in the utxo store, because we may not sync the block from genesis
                };

                if (utxo_value_accumulator >= output_value){
                    let previous_output_next_offset_new = if(output_value >= previous_utxo_value) {
                        output_value - (utxo_value_accumulator - previous_utxo_avaliable_value)
                    } else {
                        previous_output_next_offset + output_value
                    };
                    let utxo_range = utxo::new_utxo_range(previous_txid, previous_vout, previous_output_next_offset, previous_output_next_offset_new);
                    vector::push_back(&mut input_to_output_mapping, utxo_range);
                    previous_output_next_offset = previous_output_next_offset_new;

                    break;
                } else {
                    if (previous_utxo_avaliable_value > 0){
                        let utxo_range = utxo::new_utxo_range(previous_txid, previous_vout, previous_output_next_offset, previous_utxo_value);
                        vector::push_back(&mut input_to_output_mapping, utxo_range);
                    };
                    // reset previous_output_next_offset
                    previous_output_next_offset = 0;
                };

                previous_output_idx = previous_output_idx + 1;
            };

            vector::push_back(&mut output_utxo_mapping, input_to_output_mapping);
            output_idx = output_idx + 1;
        };

        // match satpoint mapping
        let output_utxo_mapping_idx = 0;
        let output_utxo_mapping_len = vector::length(&output_utxo_mapping);
        let output_satpoint_mapping = vector[];

        let previous_output_to_satpoint_mapping = simple_map::new<OutPoint, vector<SatPoint>>();
        vector::for_each(previous_outputs, |output| {
            let satpoints = inscriptions_on_output(&output);
            simple_map::add(&mut previous_output_to_satpoint_mapping, output, satpoints);
        });
        while(output_utxo_mapping_idx < output_utxo_mapping_len) {
            let satpoint_mapping = vector[];
            let vout = (output_utxo_mapping_idx as u32);

            let input_to_output_mapping = *vector::borrow(&output_utxo_mapping, output_utxo_mapping_idx);
            let offset_accumulator: u64 = 0;
            vector::for_each(input_to_output_mapping, |utxo_range| {
                let utxo_outpint = new_outpoint(utxo_range_txid(&utxo_range), utxo_range_vout(&utxo_range));
                let previous_output_to_satpoint = simple_map::borrow(&previous_output_to_satpoint_mapping, &utxo_outpint);

                let previous_output_to_satpoint_idx = 0;
                let previous_output_to_satpoint_len = vector::length(previous_output_to_satpoint);
                let (range_start_offset, range_end_offset) = utxo_range_range(&utxo_range);
                while(previous_output_to_satpoint_idx < previous_output_to_satpoint_len) {
                    let previous_satpoint = vector::borrow(previous_output_to_satpoint, previous_output_to_satpoint_idx);
                    let previous_satpoint_offset = satpoint_offset(previous_satpoint);
                    if(previous_satpoint_offset >= range_start_offset && previous_satpoint_offset < range_end_offset) {
                        let new_offset = offset_accumulator + (previous_satpoint_offset - range_start_offset);
                        let new_satpoint = new_satpoint(txid, vout, new_offset);
                        let satpoint_mapping_item = new_satpoint_mapping(*previous_satpoint, new_satpoint);
                        vector::push_back(&mut satpoint_mapping, satpoint_mapping_item);
                    } else {
                    };

                    previous_output_to_satpoint_idx = previous_output_to_satpoint_idx + 1;
                };
                offset_accumulator = offset_accumulator + (range_end_offset - range_start_offset);
            });

            vector::push_back(&mut output_satpoint_mapping, satpoint_mapping);
            output_utxo_mapping_idx = output_utxo_mapping_idx + 1;
        };

        // spend utxo and update satpoint mapping
        let output_satpoint_mapping_idx = 0;
        let output_satpoint_mapping_len = vector::length(&output_satpoint_mapping);
        while(output_satpoint_mapping_idx < output_satpoint_mapping_len) {
            let output_satpoint_mapping_item = vector::borrow(&output_satpoint_mapping, output_satpoint_mapping_idx);
            let txout = vector::borrow(txoutput, output_satpoint_mapping_idx);
            let outpoint = types::new_outpoint(txid, (output_satpoint_mapping_idx as u32));
            vector::for_each(*output_satpoint_mapping_item, |satpoint_mapping_item| {
                let (old_satpoint, new_satpoint) = satpoint_mapping(&satpoint_mapping_item);
                update_inscription_index(txout, outpoint, old_satpoint, new_satpoint, tx);
            });
        };
        vector::for_each(previous_outputs, |outpoint| {
            // spent utxo
            if(table::contains(&btc_utxo_store.utxo, outpoint)) {
                let object_id = table::remove(&mut btc_utxo_store.utxo, outpoint);
                let (_owner, utxo_obj) = utxo::take(object_id);
                let seals = utxo::remove(utxo_obj);
                //The seals should be empty after utxo is spent
                simple_multimap::destroy_empty(seals);
            };

            remove_inscription_index(outpoint);
        });



        // while(idx < vector::length(txinput)){
        //     let txin = vector::borrow(txinput, idx);
        //     let outpoint = *types::txin_previous_output(txin);
        //
        //     vector::push_back(&mut previous_outputs, outpoint);

            // if(table::contains(&btc_utxo_store.utxo, outpoint)){
            //     let object_id = table::remove(&mut btc_utxo_store.utxo, outpoint);
            //     let (_owner, utxo_obj) = utxo::take(object_id);

                // // vector::push_back(&mut previous_outputs, utxo_obj);
                // let obj = object::new_named_object(btc_utxo_store);
                // object::to_shared(obj);
                // let inscription_store_obj_id = object::named_object_id<InscriptionStore>();
                // let inscription_store_obj = object::borrow_object<InscriptionStore>(inscription_store_obj_id);
                // let inscription_store = object::borrow(inscription_store_obj);

                // let seal_outs = ord::spend_utxo(&mut utxo_obj, tx);
                // if(!vector::is_empty(&seal_outs)){
                //     let protocol = type_info::type_name<Inscription>();
                //     let j = 0;
                //     let seal_outs_len = vector::length(&seal_outs);
                //     while(j < seal_outs_len){
                //         let seal_out = vector::pop_back(&mut seal_outs);
                //         let (output_index, object_id) = utxo::unpack_seal_out(seal_out);
                //         let utxo_seal = utxo::new_utxo_seal(protocol, object_id);
                //         simple_multimap::add(&mut output_seals, output_index, utxo_seal);
                //         j = j + 1;
                //     };
                // };
                // let seals = utxo::remove(utxo_obj);
                // //The seals should be empty after utxo is spent
                // simple_multimap::destroy_empty(seals);
            // }else{
            //     //We allow the utxo not exists in the utxo store, because we may not sync the block from genesis
            // };

        //     idx = idx + 1;
        // };

        // let txoutput = types::tx_output(tx);
        let txoutput_len = vector::length(txoutput);

        let output_index = 0;
        let idx = 0;
        let txoutput_len = vector::length(txoutput);
        while(idx < txoutput_len){
            let txout = vector::borrow(txoutput, idx);
            let vout = (idx as u32);
            let outpoint = types::new_outpoint(txid, vout);
            let value = types::txout_value(txout);

        };





        //If a utxo is spend seal assets, it should not seal new assets
        //TODO confirm this
        if(simple_multimap::length(&output_seals) == 0){
            let ord_seals = ord::process_transaction(tx);

            let ord_seals_len = vector::length(&ord_seals);
            let idx = 0;
            let protocol = type_info::type_name<Inscription>();
            while(idx < ord_seals_len){
                let seal_out = vector::pop_back(&mut ord_seals);
                let (output_index, object_id) = utxo::unpack_seal_out(seal_out);
                let utxo_seal = utxo::new_utxo_seal(protocol, object_id);
                simple_multimap::add(&mut output_seals, output_index, utxo_seal);
                idx = idx + 1;
            };
        };
        let txoutput = types::tx_output(tx);
        let idx = 0;
        let txoutput_len = vector::length(txoutput); 
        while(idx < txoutput_len){
            let txout = vector::borrow(txoutput, idx);
            let vout = (idx as u32);
            let outpoint = types::new_outpoint(txid, vout);
            let value = types::txout_value(txout);
            let utxo_obj = utxo::new(txid, vout, value);
            let utxo = object::borrow_mut(&mut utxo_obj);
            if(simple_multimap::contains_key(&output_seals, &idx)){
                let utxo_seals = simple_multimap::borrow_mut(&mut output_seals, &idx);
                let j = 0;
                let utxo_seals_len = vector::length(utxo_seals);
                while(j < utxo_seals_len){
                    let utxo_seal = vector::pop_back(utxo_seals);
                    utxo::add_seal(utxo, utxo_seal);
                    j = j + 1;
                };
            };
            let object_id = object::id(&utxo_obj);
            table::add(&mut btc_utxo_store.utxo, outpoint, object_id);
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
    entry fun submit_new_block(btc_block_store_obj: &mut Object<BitcoinBlockStore>, block_height: u64, block_hash: address, block_bytes: vector<u8>){
        let time = process_block(btc_block_store_obj, block_height, block_hash, block_bytes);

        let timestamp_seconds = (time as u64);
        let module_signer = signer::module_signer<BitcoinBlockStore>();
        timestamp::try_update_global_time(&module_signer, timestamp::seconds_to_milliseconds(timestamp_seconds));      
    }

    public fun remaining_tx_count(btc_block_store_obj: &Object<BitcoinBlockStore>, btc_utxo_store_obj: &Object<BitcoinUTXOStore>): u64{
        let btc_block_store = object::borrow(btc_block_store_obj);
        let btc_utxo_store = object::borrow(btc_utxo_store_obj);
        let start_tx_index = btc_utxo_store.next_tx_index;
        let max_tx_count = table_vec::length(&btc_block_store.tx_ids);
        if(start_tx_index < max_tx_count){
            max_tx_count - start_tx_index
        }else{
            0
        }
    }
    
    entry fun process_utxos(btc_block_store_obj: &Object<BitcoinBlockStore>, btc_utxo_store_obj: &mut Object<BitcoinUTXOStore>, batch_size: u64){
        let btc_block_store = object::borrow(btc_block_store_obj);
        let btc_utxo_store = object::borrow_mut(btc_utxo_store_obj);
        let start_tx_index = btc_utxo_store.next_tx_index;
        let max_tx_count = table_vec::length(&btc_block_store.tx_ids);
        if (start_tx_index >= max_tx_count){
            return
        };
        let processed_tx_count = 0;
        let process_tx_index = start_tx_index;
        while(processed_tx_count < batch_size && process_tx_index < max_tx_count){
            let txid = *table_vec::borrow(&btc_block_store.tx_ids, process_tx_index);
            let tx = table::borrow(&btc_block_store.txs, txid);
            process_utxo(btc_utxo_store, tx);
            processed_tx_count = processed_tx_count + 1;
            process_tx_index = process_tx_index + 1;
        };
        btc_utxo_store.next_tx_index = process_tx_index;
    }

    public fun txs(btc_block_store_obj: &Object<BitcoinBlockStore>): &Table<address, Transaction>{
        let btc_block_store = object::borrow(btc_block_store_obj);
        &btc_block_store.txs
    }

    public fun tx_ids(btc_block_store_obj: &Object<BitcoinBlockStore>): &TableVec<address>{
        let btc_block_store = object::borrow(btc_block_store_obj);
        &btc_block_store.tx_ids
    }

    public fun get_tx(btc_block_store_obj: &Object<BitcoinBlockStore>, txid: address): Option<Transaction>{
        let btc_block_store = object::borrow(btc_block_store_obj);
        if(table::contains(&btc_block_store.txs, txid)){
            option::some(*table::borrow(&btc_block_store.txs, txid))
        }else{
            option::none()
        }
    }

    /// Get block via block_hash
    public fun get_block(btc_block_store_obj: &Object<BitcoinBlockStore>, block_hash: address): Option<Header>{
        let btc_block_store = object::borrow(btc_block_store_obj);
        if(table::contains(&btc_block_store.blocks, block_hash)){
            option::some(*table::borrow(&btc_block_store.blocks, block_hash))
        }else{
            option::none()
        }
    }

    public fun get_block_height(btc_block_store_obj: &Object<BitcoinBlockStore>, block_hash: address): Option<u64>{
        let btc_block_store = object::borrow(btc_block_store_obj);
        if(table::contains(&btc_block_store.hash_to_height, block_hash)){
            option::some(*table::borrow(&btc_block_store.hash_to_height, block_hash))
        }else{
            option::none()
        }
    }

    /// Get block via block_height
    public fun get_block_by_height(btc_block_store_obj: &Object<BitcoinBlockStore>, block_height: u64): Option<Header>{
        let btc_block_store = object::borrow(btc_block_store_obj);
        if(table::contains(&btc_block_store.height_to_hash, block_height)){
            let block_hash = *table::borrow(&btc_block_store.height_to_hash, block_height);
            option::some(*table::borrow(&btc_block_store.blocks, block_hash))
        }else{
            option::none()
        }
    }

    /// Get latest block height
    public fun get_latest_block_height(btc_block_store_obj: &Object<BitcoinBlockStore>): Option<u64> {
        let btc_block_store = object::borrow(btc_block_store_obj);
        btc_block_store.latest_block_height
    }

    /// Get UTXO via txid and vout
    public fun get_utxo(btc_utxo_store_obj: &Object<BitcoinUTXOStore>, txid: address, vout: u32): Option<ObjectID>{
        let outpoint = types::new_outpoint(txid, vout);
        let btc_utxo_store = object::borrow(btc_utxo_store_obj);
        if(table::contains(&btc_utxo_store.utxo, outpoint)){
            option::some(*table::borrow(&btc_utxo_store.utxo, outpoint))
        }else{
            option::none()
        }

        // get_utxo_by_outpoint(btc_utxo_store_obj, outpoint)
    }

    /// Get UTXO via outpoint
    fun get_utxo_by_outpoint(btc_utxo_store: &BitcoinUTXOStore, outpoint: OutPoint): Option<ObjectID>{
        // let btc_utxo_store = object::borrow(btc_utxo_store_obj);
        if(table::contains(&btc_utxo_store.utxo, outpoint)){
            option::some(*table::borrow(&btc_utxo_store.utxo, outpoint))
        }else{
            option::none()
        }
    }
    
}