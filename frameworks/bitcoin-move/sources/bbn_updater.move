// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

module bitcoin_move::bbn_updater {

    use std::option::{Self, is_none, is_some};

    use bitcoin_move::types;
    use bitcoin_move::bbn::{Self, BBNStakeSeal};
    use bitcoin_move::bitcoin;

    const ErrorTransactionNotFound: u64 = 1;

    /// Check if the transaction is a possible Babylon transaction
    /// If the transaction contains an OP_RETURN output with the correct tag, it is considered a possible Babylon transaction
    public fun is_possible_bbn_tx(txid: address): bool {
        let block_height_opt = bitcoin::get_tx_height(txid);
        if (is_none(&block_height_opt)) {
            return false
        };
        let block_height = option::destroy_some(block_height_opt); 
        let tx_opt = bitcoin::get_tx(txid);
        if (is_none(&tx_opt)) {
            return false
        };
        let tx = option::destroy_some(tx_opt);
        bbn::is_possible_bbn_transaction(block_height, &tx)
    }

    public entry fun process_bbn_tx_entry(txid: address){
        process_bbn_tx(txid)
    }

    fun process_bbn_tx(txid: address) {
        let block_height_opt = bitcoin::get_tx_height(txid);
        assert!(is_some(&block_height_opt), ErrorTransactionNotFound);
        let block_height = option::destroy_some(block_height_opt);

        let tx_opt = bitcoin::get_tx(txid);
        assert!(is_some(&tx_opt), ErrorTransactionNotFound);
        
        let tx = option::destroy_some(tx_opt);
        bbn::process_bbn_transaction(block_height, &tx)
    }

    public fun is_expired(stake: &BBNStakeSeal): bool {
        let latest_block_opt = bitcoin::get_latest_block();
        if (is_none(&latest_block_opt)) {
            return false
        };
        let latest_block = option::destroy_some(latest_block_opt);
        let (current_block_height, _hash) = types::unpack_block_height_hash(latest_block);
        bbn::is_expired_at(stake, current_block_height)
    }
}