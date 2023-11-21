module rooch_framework::bitcoin_light_client{

    use std::error;
    use moveos_std::context::{Self, Context};
    use moveos_std::table::{Self, Table};
    use rooch_framework::timestamp;    
    use moveos_std::bcs;

    friend rooch_framework::genesis;

    const ErrorBlockNotFound:u64 = 1;

    #[data_struct]
    struct BlockHeader has store, copy, drop {
        /// The hash of the block header.
        hash: vector<u8>,
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

    struct BlockStore has key{
        /// block hash -> block header
        blocks: Table<vector<u8>, BlockHeader>,
    }

    public(friend) fun genesis_init(ctx: &mut Context, genesis_account: &signer){
        let block_store = BlockStore{
            blocks: context::new_table(ctx),
        };
        context::move_resource_to(ctx, genesis_account, block_store);
    }

    fun process_block(ctx: &mut Context, block_header_bytes: vector<u8>){
        let block_header = bcs::from_bytes<BlockHeader>(block_header_bytes);
        validate_block(ctx, &block_header);

        let block_store = context::borrow_mut_resource<BlockStore>(ctx, @rooch_framework);
        if(table::contains(&block_store.blocks, block_header.hash)){
            //TODO handle repeat block hash
            return
        };
        table::add(&mut block_store.blocks, block_header.hash, block_header);

        let timestamp_seconds = (block_header.time as u64);
        timestamp::try_update_global_time(ctx, timestamp::seconds_to_milliseconds(timestamp_seconds));        
    }

    fun validate_block(_ctx: &mut Context, _block_header: &BlockHeader){
        //TODO validate the block via bitcoin consensus
        // validate prev block hash
        // validate block hash
        // validate block nonce
    }

    /// The relay server submit a new Bitcoin block to the light client.
    public entry fun submit_new_block(ctx: &mut Context, block_header_bytes: vector<u8>){
        process_block(ctx, block_header_bytes);
    }

    /// Get block via block_hash
    public fun get_block(ctx: &Context, block_hash: vector<u8>): &BlockHeader{
        let block_store = context::borrow_resource<BlockStore>(ctx, @rooch_framework);
        assert!(table::contains(&block_store.blocks, block_hash), error::invalid_argument(ErrorBlockNotFound));
        table::borrow(&block_store.blocks, block_hash)
    }
}