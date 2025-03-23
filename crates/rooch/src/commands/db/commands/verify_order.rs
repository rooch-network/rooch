// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::commands::db::commands::load_accumulator;
use crate::utils::open_rooch_db;
use accumulator::accumulator_info::AccumulatorInfo;
use accumulator::{Accumulator, MerkleAccumulator};
use clap::Parser;
use moveos_types::h256::H256;
use rooch_config::R_OPT_NET_HELP;
use rooch_store::RoochStore;
use rooch_types::error::RoochResult;
use rooch_types::rooch_network::RoochChainID;
use std::path::PathBuf;
use tracing::info;

/// Verify Order by Accumulator
#[derive(Debug, Parser)]
pub struct VerifyOrderCommand {
    #[clap(long)]
    pub tx_hash: Option<H256>,
    #[clap(long)]
    pub bypass_hash_check: bool,
    #[clap(long)]
    pub tx_order: Option<u64>,
    #[clap(long)]
    pub first_tx_order: Option<u64>,
    #[clap(long)]
    pub last_tx_order: Option<u64>,
    #[clap(long = "data-dir", short = 'd')]
    pub base_data_dir: Option<PathBuf>,
    #[clap(long, short = 'n', help = R_OPT_NET_HELP)]
    pub chain_id: Option<RoochChainID>,
}

impl VerifyOrderCommand {
    pub fn execute(self) -> RoochResult<()> {
        let (_root, rooch_db, _start_time) = open_rooch_db(self.base_data_dir, self.chain_id);
        let rooch_store = rooch_db.rooch_store;
        let (tx_accumulator, last_tx_order_in_db) = load_accumulator(rooch_store.clone())?;

        if let Some(tx_hash) = self.tx_hash {
            let tx_order = self
                .tx_order
                .expect("tx_order should be set when tx_hash is set");
            verify_single_tx(
                &tx_accumulator,
                rooch_store,
                tx_order,
                Some(tx_hash),
                self.bypass_hash_check,
            )?;
            return Ok(());
        }

        let first_tx_order = self.first_tx_order.unwrap_or(0);
        let last_tx_order = self.last_tx_order.unwrap_or(last_tx_order_in_db);

        verify_txs(&tx_accumulator, rooch_store, first_tx_order, last_tx_order)?;
        Ok(())
    }
}

fn verify_single_tx(
    accumulator: &MerkleAccumulator,
    rooch_store: RoochStore,
    tx_order: u64,
    tx_hash_opt: Option<H256>,
    bypass_hash_check: bool,
) -> anyhow::Result<()> {
    let tx_hash_in_db = rooch_store
        .transaction_store
        .get_tx_hash(tx_order)?
        .ok_or_else(|| anyhow::anyhow!("tx_order: {} not exist", tx_order))?;
    let tx_hash = if let Some(exp_tx_hash) = tx_hash_opt {
        if !bypass_hash_check && tx_hash_in_db != exp_tx_hash {
            return Err(anyhow::anyhow!(
                "tx_hash: {:?} not match tx_order: {}, act_tx_hash: {:?}",
                exp_tx_hash,
                tx_order,
                tx_hash_in_db
            ));
        };
        exp_tx_hash
    } else {
        tx_hash_in_db
    };

    let tx = rooch_store
        .transaction_store
        .get_transaction_by_hash(tx_hash)?
        .ok_or_else(|| anyhow::anyhow!("tx_hash: {:?} not exist", tx_hash))?;
    let tx_accumulator_root = tx.sequence_info.tx_accumulator_root;
    let tx_accumulator_info = AccumulatorInfo {
        accumulator_root: tx_accumulator_root,
        frozen_subtree_roots: tx.sequence_info.tx_accumulator_frozen_subtree_roots,
        num_leaves: tx.sequence_info.tx_accumulator_num_leaves,
        num_nodes: tx.sequence_info.tx_accumulator_num_nodes,
    };
    verify_proof(
        &accumulator.fork(Some(tx_accumulator_info)),
        tx_accumulator_root,
        tx_hash,
        tx_order,
    )
}

fn verify_txs(
    accumulator: &MerkleAccumulator,
    rooch_store: RoochStore,
    first_tx_order: u64,
    last_tx_order: u64,
) -> anyhow::Result<()> {
    info!(
        "Searching for invalid transaction between orders {} and {}",
        first_tx_order, last_tx_order
    );

    // If the first transaction is valid, use binary search to find the first invalid one
    if verify_single_tx(
        accumulator,
        rooch_store.clone(),
        first_tx_order,
        None,
        false,
    )
    .is_ok()
    {
        let min_invalid_tx_order = find_first_invalid_tx(
            accumulator,
            rooch_store.clone(),
            first_tx_order,
            last_tx_order,
        );

        match min_invalid_tx_order {
            Some(invalid_order) => {
                info!(
                    "Found invalid transaction at order: {}. Valid range: [{}, {}]",
                    invalid_order,
                    first_tx_order,
                    invalid_order - 1
                );

                // Verify the invalid transaction to get the detailed error
                verify_single_tx(accumulator, rooch_store, invalid_order, None, false)
            }
            None => {
                info!(
                    "All transactions verified successfully: [{}, {}]",
                    first_tx_order, last_tx_order
                );
                Ok(())
            }
        }
    } else {
        // First transaction is invalid
        info!("First transaction (order {}) is invalid", first_tx_order);
        verify_single_tx(accumulator, rooch_store, first_tx_order, None, false)
    }
}

/// Binary search to find the first invalid transaction
fn find_first_invalid_tx(
    accumulator: &MerkleAccumulator,
    rooch_store: RoochStore,
    mut low: u64,
    mut high: u64,
) -> Option<u64> {
    // Assuming all transactions in the range [low, high] might contain an invalid tx
    // We know low is valid (checked before calling this function)

    if low > high {
        return None;
    }

    while low <= high {
        // Check if we've narrowed down to adjacent transactions
        if high - low <= 1 {
            // Since we know low is valid, check if high is valid
            return if verify_single_tx(accumulator, rooch_store.clone(), high, None, false).is_ok()
            {
                None // All transactions are valid
            } else {
                Some(high) // high is the first invalid transaction
            };
        }

        let mid = low + (high - low) / 2;

        info!("Binary search: checking tx order {}", mid);

        if verify_single_tx(accumulator, rooch_store.clone(), mid, None, false).is_ok() {
            // If mid is valid, look in the higher half
            low = mid;
        } else {
            // If mid is invalid, look in the lower half
            high = mid;
        }
    }

    // This should not be reached with the adjacent check above
    None
}

fn verify_proof(
    accumulator: &MerkleAccumulator,
    root_hash: H256,
    tx_hash: H256,
    tx_order: u64,
) -> anyhow::Result<()> {
    let leaf_index = tx_order;
    let proof = accumulator
        .get_proof(leaf_index)?
        .ok_or_else(|| anyhow::anyhow!("Proof of tx_order: {} not exist", leaf_index))?;
    proof.verify(root_hash, tx_hash, leaf_index)
}
