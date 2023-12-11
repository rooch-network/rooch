// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

module rooch_framework::bitcoin_light_client{

    use std::option::{Self, Option};
    use std::vector;
    use moveos_std::context::{Self, Context};
    use moveos_std::table::{Self, Table};
    use moveos_std::bcs;
    use moveos_std::object::{Self, Object};
    use moveos_std::table_vec::{Self, TableVec};
    use rooch_framework::timestamp;
    use rooch_framework::bitcoin_address::{Self, BTCAddress};
    use rooch_framework::bitcoin_types::{Self, Block, Header, Transaction, OutPoint, TxOut};    
    

    friend rooch_framework::genesis;

    const ErrorBlockNotFound:u64 = 1;
    const ErrorBlockAlreadyProcessed:u64 = 2;

    
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
        utxo: Table<OutPoint, TxOut>,
        /// BTC balance
        balance: Table<BTCAddress, u64>,
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
            balance: context::new_table(ctx),
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
        let block_header = bitcoin_types::header(&block);

        if(table::contains(&btc_block_store.height_to_hash, block_height)){
            //TODO handle reorg
        };
        let time = bitcoin_types::time(block_header);
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
        let txdata = bitcoin_types::txdata(block);
        let idx = 0;
        while(idx < vector::length(txdata)){
            let tx = vector::borrow(txdata, idx);
            progress_tx(btc_block_store, tx);
            idx = idx + 1;
        }
    }

    fun progress_tx(btc_block_store: &mut BitcoinBlockStore, tx: &Transaction){
        let txid = bitcoin_types::tx_id(tx);
        table::add(&mut btc_block_store.txs, txid, *tx);
        table_vec::push_back(&mut btc_block_store.tx_ids, txid);
    }

    fun progress_utxo(btc_utxo_store: &mut BitcoinUTXOStore, tx: &Transaction){
        let txinput = bitcoin_types::tx_input(tx);
        let idx = 0;
        while(idx < vector::length(txinput)){
            let txin = vector::borrow(txinput, idx);
            let outpoint = *bitcoin_types::txin_previous_output(txin);
            remove_utxo(btc_utxo_store, outpoint); 
            idx = idx + 1;
        };
        let txoutput = bitcoin_types::tx_output(tx);
        let idx = 0;
        let txid = bitcoin_types::tx_id(tx);
        while(idx < vector::length(txoutput)){
            let txout = *vector::borrow(txoutput, idx);
            let outpoint = bitcoin_types::new_outpoint(txid, (idx as u32));
            add_utxo(btc_utxo_store, outpoint, txout);
            idx = idx + 1;
        }
    }

    fun remove_utxo(btc_utxo_store: &mut BitcoinUTXOStore, outpoint: OutPoint){
        if(table::contains(&btc_utxo_store.utxo, outpoint)){
            let txout = table::remove(&mut btc_utxo_store.utxo, outpoint);
            let script_pubkey = bitcoin_types::txout_script_pubkey(&txout);
            //TODO should we record the address of the utxo?
            let address_opt = bitcoin_address::from_script(script_pubkey);
            if(option::is_some(&address_opt)){
                let address = option::extract(&mut address_opt);
                let balance = table::borrow_mut_with_default(&mut btc_utxo_store.balance, address, 0);
                if (*balance >= bitcoin_types::txout_value(&txout)){
                    *balance = *balance - bitcoin_types::txout_value(&txout);
                }else{
                    //if we sync the block from genesis, this should not happen
                    //TODO emit a event as a warning
                }
            }
        }else{
            //TODO handle double spend
        };
        
    }

    fun add_utxo(btc_utxo_store: &mut BitcoinUTXOStore, outpoint: OutPoint, txout: TxOut){
        table::add(&mut btc_utxo_store.utxo, outpoint, txout);
        let script_pubkey = bitcoin_types::txout_script_pubkey(&txout);
        let address_opt = bitcoin_address::from_script(script_pubkey);
        if(option::is_some(&address_opt)){
            let address = option::extract(&mut address_opt);
            let balance = table::borrow_mut_with_default(&mut btc_utxo_store.balance, address, 0);
            *balance = *balance + bitcoin_types::txout_value(&txout);
        }
    }


    /// The relay server submit a new Bitcoin block to the light client.
    entry fun submit_new_block(ctx: &mut Context, btc_block_store_obj: &mut Object<BitcoinBlockStore>, block_height: u64, block_hash: address, block_bytes: vector<u8>){
        let time = process_block(btc_block_store_obj, block_height, block_hash, block_bytes);

        let timestamp_seconds = (time as u64);
        timestamp::try_update_global_time(ctx, timestamp::seconds_to_milliseconds(timestamp_seconds));      
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
    
    entry fun progress_utxos(btc_block_store_obj: &Object<BitcoinBlockStore>, btc_utxo_store_obj: &mut Object<BitcoinUTXOStore>, batch_size: u64){
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
            progress_utxo(btc_utxo_store, tx);
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

    /// Get tx out via txid and vout
    public fun get_tx_out(btc_utxo_store_obj: &Object<BitcoinUTXOStore>, txid: address, vout: u32): Option<TxOut>{
        let outpoint = bitcoin_types::new_outpoint(txid, vout);
        let btc_utxo_store = object::borrow(btc_utxo_store_obj);
        if(table::contains(&btc_utxo_store.utxo, outpoint)){
            option::some(*table::borrow(&btc_utxo_store.utxo, outpoint))
        }else{
            option::none()
        }
    }
    
}