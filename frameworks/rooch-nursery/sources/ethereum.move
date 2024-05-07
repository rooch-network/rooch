// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

module rooch_nursery::ethereum{

    use moveos_std::account;
    use moveos_std::table::{Self, Table};
    use moveos_std::bcs;
    use moveos_std::signer;
    use moveos_std::features;
    use rooch_framework::ethereum_address::ETHAddress;
    use rooch_framework::timestamp; 

    friend rooch_nursery::genesis;

    const ErrorBlockNotFound:u64 = 1;

    #[data_struct]
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

    struct BlockStore has key {
        blocks: Table<u64, BlockHeader>,
    }

    public(friend) fun genesis_init(genesis_account: &signer){
        let block_store = BlockStore{
            blocks: table::new(),
        };
        account::move_resource_to(genesis_account, block_store);
    }

    fun process_block(block_header_bytes: vector<u8>){
        let block_header = bcs::from_bytes<BlockHeader>(block_header_bytes);
        //TODO validate the block hash
        //TODO validate the block via ethereum consensus(pos validators)
        let block_store = account::borrow_mut_resource<BlockStore>(@rooch_framework);
        if(table::contains(&block_store.blocks, block_header.number)){
            //repeat block number
            //TODO check if it is a soft fork.
            return
        };
        table::add(&mut block_store.blocks, block_header.number, block_header);

        let timestamp_seconds = (block_header.timestamp as u64);
        let module_signer = signer::module_signer<BlockStore>();
        timestamp::try_update_global_time(&module_signer, timestamp::seconds_to_milliseconds(timestamp_seconds));        
    }

    /// The relay server submit a new Ethereum block to the light client.
    public entry fun submit_new_block(block_header_bytes: vector<u8>){
        features::ensure_localnet_enabled();
        process_block(block_header_bytes);
    }

    /// Get block via block_number
    public fun get_block(block_number: u64): &BlockHeader{
        let block_store = account::borrow_resource<BlockStore>(@rooch_framework);
        assert!(table::contains(&block_store.blocks, block_number), ErrorBlockNotFound);
        table::borrow(&block_store.blocks, block_number)
    }
}
