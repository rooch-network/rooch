// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

module bitcoin_move::light_client{

    use std::option::{Self, Option};
    use std::vector;
    use std::string::{String};
    use moveos_std::type_info;
    //use moveos_std::event;
    use moveos_std::context::{Self, Context};
    use moveos_std::table::{Self, Table};
    use moveos_std::bcs;
    use moveos_std::object::{Self, Object, ObjectID};
    use moveos_std::table_vec::{Self, TableVec};
    use moveos_std::simple_multimap;
    use moveos_std::signer;
    use rooch_framework::timestamp;
    use bitcoin_move::types::{Self, Block, Header, Transaction, OutPoint};    
    use bitcoin_move::ord::{Self, Inscription};
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

    struct BitcoinUTXOStore has key{
        /// The next tx index to be processed
        next_tx_index: u64,
        /// outpoint -> txout
        utxo: Table<OutPoint, ObjectID>,
    }

    public(friend) fun genesis_init(ctx: &mut Context, _genesis_account: &signer){
        let btc_block_store = BitcoinBlockStore{
            latest_block_height: option::none(),
            blocks: context::new_table(ctx),
            height_to_hash: context::new_table(ctx),
            hash_to_height: context::new_table(ctx),
            txs: context::new_table(ctx),
            tx_ids: context::new_table_vec(ctx),
        };
        let obj = context::new_named_object(ctx, btc_block_store);
        object::to_shared(obj);

        let btc_utxo_store = BitcoinUTXOStore{
            next_tx_index: 0,
            utxo: context::new_table(ctx),
        };
        let obj = context::new_named_object(ctx, btc_utxo_store);
        object::to_shared(obj);
    }

    fun process_block(btc_block_store_obj: &mut Object<BitcoinBlockStore>, block_height: u64, block_hash: address, block_bytes: vector<u8>):u32{
        
        let btc_block_store = object::borrow_mut(btc_block_store_obj);
        //already processed
        assert!(!table::contains(&btc_block_store.hash_to_height, block_hash), ErrorBlockAlreadyProcessed);

        let block = bcs::from_bytes<Block>(block_bytes);
        validate_block(btc_block_store, block_height, block_hash, &block);
        progress_txs(btc_block_store, &block); 
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

    fun progress_txs(btc_block_store: &mut BitcoinBlockStore, block:&Block){
        let txdata = types::txdata(block);
        let idx = 0;
        while(idx < vector::length(txdata)){
            let tx = vector::borrow(txdata, idx);
            progress_tx(btc_block_store, tx);
            idx = idx + 1;
        }
    }

    fun progress_tx(btc_block_store: &mut BitcoinBlockStore, tx: &Transaction){
        let txid = types::tx_id(tx);
        table::add(&mut btc_block_store.txs, txid, *tx);
        table_vec::push_back(&mut btc_block_store.tx_ids, txid);
    }

    fun progress_utxo(ctx: &mut Context, btc_utxo_store: &mut BitcoinUTXOStore, tx: &Transaction){
        let txid = types::tx_id(tx);
        let txinput = types::tx_input(tx);
        let idx = 0;
        let output_seals = simple_multimap::new<u64, UTXOSeal>();
        while(idx < vector::length(txinput)){
            let txin = vector::borrow(txinput, idx);
            let outpoint = *types::txin_previous_output(txin);
            if(table::contains(&btc_utxo_store.utxo, outpoint)){
                let object_id = table::remove(&mut btc_utxo_store.utxo, outpoint);
                let (_owner, utxo_obj) = utxo::take(ctx, object_id);
                let seal_outs = ord::spend_utxo(ctx, &mut utxo_obj, tx);
                if(!vector::is_empty(&seal_outs)){
                    let protocol = type_info::type_name<Inscription>();
                    let j = 0;
                    let seal_outs_len = vector::length(&seal_outs);
                    while(j < seal_outs_len){
                        let seal_out = vector::pop_back(&mut seal_outs);
                        let (output_index, object_id) = utxo::unpack_seal_out(seal_out);
                        let utxo_seal = utxo::new_utxo_seal(protocol, object_id);
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
        //TODO confirm this
        if(simple_multimap::length(&output_seals) == 0){
            let ord_seals = ord::progress_transaction(ctx, tx);
            let idx = 0;
            let protocol = type_info::type_name<Inscription>();
            while(idx < vector::length(&ord_seals)){
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
            let utxo_obj = utxo::new(ctx, txid, vout, value);
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
            idx = idx + 1;
        };
        simple_multimap::drop(output_seals);
    }


    /// The relay server submit a new Bitcoin block to the light client.
    entry fun submit_new_block(ctx: &mut Context, btc_block_store_obj: &mut Object<BitcoinBlockStore>, block_height: u64, block_hash: address, block_bytes: vector<u8>){
        let time = process_block(btc_block_store_obj, block_height, block_hash, block_bytes);

        let timestamp_seconds = (time as u64);
        let module_signer = signer::module_signer<BitcoinBlockStore>();
        timestamp::try_update_global_time(ctx, &module_signer, timestamp::seconds_to_milliseconds(timestamp_seconds));      
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
    
    entry fun progress_utxos(ctx: &mut Context, btc_block_store_obj: &Object<BitcoinBlockStore>, btc_utxo_store_obj: &mut Object<BitcoinUTXOStore>, batch_size: u64){
        let btc_block_store = object::borrow(btc_block_store_obj);
        let btc_utxo_store = object::borrow_mut(btc_utxo_store_obj);
        let start_tx_index = btc_utxo_store.next_tx_index;
        let max_tx_count = table_vec::length(&btc_block_store.tx_ids);
        if (start_tx_index >= max_tx_count){
            return
        };
        let progressed_tx_count = 0;
        let progress_tx_index = start_tx_index;
        while(progressed_tx_count < batch_size && progress_tx_index < max_tx_count){
            let txid = *table_vec::borrow(&btc_block_store.tx_ids, progress_tx_index);
            let tx = table::borrow(&btc_block_store.txs, txid);
            progress_utxo(ctx, btc_utxo_store, tx);
            progressed_tx_count = progressed_tx_count + 1;
            progress_tx_index = progress_tx_index + 1;
        };
        btc_utxo_store.next_tx_index = progress_tx_index;
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

    /// Get block via block_height
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
    }
    
}