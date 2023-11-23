module rooch_framework::bitcoin_light_client{

    use std::error;
    use std::option::{Self, Option};
    use std::vector;
    use moveos_std::context::{Self, Context};
    use moveos_std::table::{Self, Table};
    use moveos_std::bcs;
    use moveos_std::object::{Self, Object};
    use rooch_framework::timestamp;
    use rooch_framework::bitcoin_types::{Self, Block, Header, Transaction, OutPoint, TxOut};    
    

    friend rooch_framework::genesis;

    const ErrorBlockNotFound:u64 = 1;
    const ErrorBlockAlreadyProcessed:u64 = 2;

    
    struct BitcoinStore has key{
        latest_block_height: Option<u64>,
        /// block hash -> block header
        blocks: Table<address, Header>,
        /// block height -> block hash
        height_to_hash: Table<u64, address>,
        /// block hash -> block height
        hash_to_height: Table<address, u64>,
        /// outpoint -> txout
        utxo: Table<OutPoint, TxOut>,
    }

    public(friend) fun genesis_init(ctx: &mut Context, _genesis_account: &signer){
        let btc_store = BitcoinStore{
            latest_block_height: option::none(),
            blocks: context::new_table(ctx),
            height_to_hash: context::new_table(ctx),
            hash_to_height: context::new_table(ctx),
            utxo: context::new_table(ctx),
        };
        let obj = context::new_named_object(ctx, btc_store);
        object::to_shared(obj);
    }

    fun process_block(btc_store_obj: &mut Object<BitcoinStore>, block_height: u64, block_hash: address, block_bytes: vector<u8>):u32{
        
        let btc_store = object::borrow_mut(btc_store_obj);
        //already processed
        assert!(!table::contains(&btc_store.hash_to_height, block_hash), error::invalid_argument(ErrorBlockAlreadyProcessed));

        let block = bcs::from_bytes<Block>(block_bytes);
        validate_block(btc_store, block_height, block_hash, &block);
        progress_txs(btc_store, &block); 
        let block_header = bitcoin_types::header(&block);

        if(table::contains(&btc_store.height_to_hash, block_height)){
            //TODO handle reorg
        };
        let time = bitcoin_types::time(block_header);
        table::add(&mut btc_store.height_to_hash, block_height, block_hash);
        table::add(&mut btc_store.hash_to_height, block_hash, block_height);
        table::add(&mut btc_store.blocks, block_hash, *block_header);
        btc_store.latest_block_height = option::some(block_height);
        time 
    }

    fun validate_block(_btc_store: &BitcoinStore, _block_height: u64, _block_hash: address, _block: &Block){
        //TODO validate the block via bitcoin consensus
        // validate prev block hash
        // validate block hash
        // validate block nonce
        //TODO validate txid
    }

    fun progress_txs(btc_store: &mut BitcoinStore, block:&Block){
        let txdata = bitcoin_types::txdata(block);
        let idx = 0;
        while(idx < vector::length(txdata)){
            let tx = vector::borrow(txdata, idx);
            progress_tx(btc_store, tx);
            idx = idx + 1;
        }
    }

    fun progress_tx(btc_store: &mut BitcoinStore, tx: &Transaction){
        let txinput = bitcoin_types::tx_input(tx);
        let idx = 0;
        while(idx < vector::length(txinput)){
            let txin = vector::borrow(txinput, idx);
            let outpoint = *bitcoin_types::txin_previous_output(txin);
            if(table::contains(&btc_store.utxo, outpoint)){
                table::remove(&mut btc_store.utxo, outpoint);
            }else{
                //TODO handle double spend
            };
            idx = idx + 1;
        };
        let txoutput = bitcoin_types::tx_output(tx);
        let idx = 0;
        let txid = bitcoin_types::tx_id(tx);
        while(idx < vector::length(txoutput)){
            let txout = *vector::borrow(txoutput, idx);
            let outpoint = bitcoin_types::new_outpoint(txid, (idx as u32));
            table::add(&mut btc_store.utxo, outpoint, txout);
            idx = idx + 1;
        }
    }

    /// The relay server submit a new Bitcoin block to the light client.
    entry fun submit_new_block(ctx: &mut Context, btc_store_obj: &mut Object<BitcoinStore>, block_height: u64, block_hash: address, block_bytes: vector<u8>){
        let time = process_block(btc_store_obj, block_height, block_hash, block_bytes);

        let timestamp_seconds = (time as u64);
        timestamp::try_update_global_time(ctx, timestamp::seconds_to_milliseconds(timestamp_seconds));      
    }

    /// Get block via block_hash
    public fun get_block(btc_store_obj: &Object<BitcoinStore>, block_hash: address): Option<Header>{
        let btc_store = object::borrow(btc_store_obj);
        if(table::contains(&btc_store.blocks, block_hash)){
            option::some(*table::borrow(&btc_store.blocks, block_hash))
        }else{
            option::none()
        }
    }

    public fun get_block_height(btc_store_obj: &Object<BitcoinStore>, block_hash: address): Option<u64>{
        let btc_store = object::borrow(btc_store_obj);
        if(table::contains(&btc_store.hash_to_height, block_hash)){
            option::some(*table::borrow(&btc_store.hash_to_height, block_hash))
        }else{
            option::none()
        }
    }

    /// Get block via block_height
    public fun get_block_by_height(btc_store_obj: &Object<BitcoinStore>, block_height: u64): Option<Header>{
        let btc_store = object::borrow(btc_store_obj);
        if(table::contains(&btc_store.height_to_hash, block_height)){
            let block_hash = *table::borrow(&btc_store.height_to_hash, block_height);
            option::some(*table::borrow(&btc_store.blocks, block_hash))
        }else{
            option::none()
        }
    }

    /// Get block via block_height
    public fun get_latest_block_height(btc_store_obj: &Object<BitcoinStore>): Option<u64> {
        let btc_store = object::borrow(btc_store_obj);
        btc_store.latest_block_height
    }

    /// Get tx out via txid and vout
    public fun get_tx_out(btc_store_obj: &Object<BitcoinStore>, txid: address, vout: u32): Option<TxOut>{
        let outpoint = bitcoin_types::new_outpoint(txid, vout);
        let btc_store = object::borrow(btc_store_obj);
        if(table::contains(&btc_store.utxo, outpoint)){
            option::some(*table::borrow(&btc_store.utxo, outpoint))
        }else{
            option::none()
        }
    }
    
}