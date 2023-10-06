module rooch_framework::ethereum_light_client{

    use std::error;
    use moveos_std::bcs;
    use moveos_std::storage_context::StorageContext;
    use moveos_std::account_storage;
    use moveos_std::table::{Self, Table};
    use rooch_framework::ethereum_address::ETHAddress;
    use rooch_framework::timestamp;

    friend rooch_framework::genesis;

    const ErrorBlockNotFound:u64 = 1;

    struct BlockHeader has store, copy, drop {
        /// Hash of the block
        hash: vector<u8>,
        /// Hash of the parent
        parent_hash: vector<u8>,
        /// Hash of the uncles
        uncles_hash: vector<u8>,
        /// Miner/author's address.
        author: ETHAddress,
        /// State root hash
        state_root: vector<u8>,
        /// Transactions root hash
        transactions_root: vector<u8>,
        /// Transactions receipts root hash
        receipts_root: vector<u8>,
        /// Logs bloom
        logs_bloom: vector<u8>,
        /// Difficulty
        difficulty: u256,
        /// Block number.
        number: u64,
        /// Gas Limit
        gas_limit: u256,
        /// Gas Used
        gas_used: u256,
        /// Timestamp
        timestamp: u256,
        /// Extra data
        extra_data: vector<u8>,
    }

    struct BlockStore has key{
        blocks: Table<u64, BlockHeader>,
    }

    public(friend) fun genesis_init(ctx: &mut StorageContext, genesis_account: &signer){
        let block_store = BlockStore{
            blocks: table::new(ctx),
        };
        account_storage::global_move_to(ctx, genesis_account, block_store);
    }

    fun process_block(ctx: &mut StorageContext, block_header_bytes: vector<u8>){
        //TODO find a way to deserialize the block header, do not use bcs::from_bytes
        let block_header = bcs::from_bytes<BlockHeader>(block_header_bytes);
        //TODO validate the block hash
        //TODO validate the block via ethereum consensus(pos validators)
        let block_store = account_storage::global_borrow_mut<BlockStore>(ctx, @rooch_framework);
        if(table::contains(&block_store.blocks, block_header.number)){
            //repeat block number
            //TODO check if it is a soft fork.
            return
        };
        table::add(&mut block_store.blocks, block_header.number, block_header);

        let timestamp_seconds = (block_header.timestamp as u64);
        timestamp::try_update_global_time(ctx, timestamp::seconds_to_microseconds(timestamp_seconds));        
    }

    /// The relay server submit a new Ethereum block to the light client.
    public entry fun submit_new_block(ctx: &mut StorageContext, block_header_bytes: vector<u8>){
        process_block(ctx, block_header_bytes);
    }

    #[view]
    /// Get block via block_number
    public fun get_block(ctx: &StorageContext, block_number: u64): &BlockHeader{
        let block_store = account_storage::global_borrow<BlockStore>(ctx, @rooch_framework);
        assert!(table::contains(&block_store.blocks, block_number), error::invalid_argument(ErrorBlockNotFound));
        table::borrow(&block_store.blocks, block_number)
    }
}