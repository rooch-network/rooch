module rooch_framework::bitcoin_light_client{

    use std::error;
    use std::option::{Self, Option};
    use moveos_std::context::{Self, Context};
    use moveos_std::table::{Self, Table};
    use moveos_std::bcs;
    use moveos_std::object::{Self, Object};
    use rooch_framework::timestamp;    
    

    friend rooch_framework::genesis;

    const ErrorBlockNotFound:u64 = 1;
    const ErrorBlockAlreadyProcessed:u64 = 2;

    #[data_struct]
    struct BlockHeader has store, copy, drop {
        /// Block version, now repurposed for soft fork signalling.
        version: u32,
        /// Reference to the previous block in the chain.
        prev_blockhash: vector<u8>,
        /// The root hash of the merkle tree of transactions in the block.
        merkle_root: vector<u8>,
        /// The timestamp of the block, as claimed by the miner.
        time: u32,
        /// The target value below which the blockhash must lie.
        bits: u32,
        /// The nonce, selected to obtain a low enough blockhash.
        nonce: u32,
    }

    struct BitcoinStore has key{
        latest_block_height: Option<u64>,
        /// block hash -> block header
        blocks: Table<vector<u8>, BlockHeader>,
        /// block height -> block hash
        height_to_hash: Table<u64, vector<u8>>,
        /// block hash -> block height
        hash_to_height: Table<vector<u8>, u64>,
        //TODO add utxo store
    }

    public(friend) fun genesis_init(ctx: &mut Context, _genesis_account: &signer){
        let btc_store = BitcoinStore{
            latest_block_height: option::none(),
            blocks: context::new_table(ctx),
            height_to_hash: context::new_table(ctx),
            hash_to_height: context::new_table(ctx),
        };
        let obj = context::new_named_object(ctx, btc_store);
        object::to_shared(obj);
    }

    fun process_block(btc_store_obj: &mut Object<BitcoinStore>, block_height: u64, block_hash: vector<u8>, block_header_bytes: vector<u8>):u32{
        
        let btc_store = object::borrow_mut(btc_store_obj);
        //already processed
        assert!(!table::contains(&btc_store.hash_to_height, block_hash), error::invalid_argument(ErrorBlockAlreadyProcessed));

        let block_header = bcs::from_bytes<BlockHeader>(block_header_bytes);
        validate_block(btc_store, block_height, &block_hash, &block_header);

        if(table::contains(&btc_store.height_to_hash, block_height)){
            //TODO handle reorg
        };
        let time = block_header.time;
        table::add(&mut btc_store.height_to_hash, block_height, block_hash);
        table::add(&mut btc_store.hash_to_height, block_hash, block_height);
        table::add(&mut btc_store.blocks, block_hash, block_header);
        btc_store.latest_block_height = option::some(block_height);
        time 
    }

    fun validate_block(_btc_store: &BitcoinStore, _block_height: u64, _block_hash: &vector<u8>, _block_header: &BlockHeader){
        //TODO validate the block via bitcoin consensus
        // validate prev block hash
        // validate block hash
        // validate block nonce
    }

    /// The relay server submit a new Bitcoin block to the light client.
    entry fun submit_new_block(ctx: &mut Context, btc_store_obj: &mut Object<BitcoinStore>, block_height: u64, block_hash: vector<u8>, block_header_bytes: vector<u8>){
        let time = process_block(btc_store_obj, block_height, block_hash, block_header_bytes);

        let timestamp_seconds = (time as u64);
        timestamp::try_update_global_time(ctx, timestamp::seconds_to_milliseconds(timestamp_seconds));      
    }

    /// Get block via block_hash
    public fun get_block(btc_store_obj: &Object<BitcoinStore>, block_hash: vector<u8>): Option<BlockHeader>{
        let btc_store = object::borrow(btc_store_obj);
        if(table::contains(&btc_store.blocks, block_hash)){
            option::some(*table::borrow(&btc_store.blocks, block_hash))
        }else{
            option::none()
        }
    }

    public fun get_block_height(btc_store_obj: &Object<BitcoinStore>, block_hash: vector<u8>): Option<u64>{
        let btc_store = object::borrow(btc_store_obj);
        if(table::contains(&btc_store.hash_to_height, block_hash)){
            option::some(*table::borrow(&btc_store.hash_to_height, block_hash))
        }else{
            option::none()
        }
    }

    /// Get block via block_height
    public fun get_block_by_height(btc_store_obj: &Object<BitcoinStore>, block_height: u64): Option<BlockHeader>{
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

    
}