// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::commands::db::commands::init;
use clap::Parser;
use moveos_store::transaction_store::TransactionStore;
use moveos_types::h256::H256;
use rooch_config::R_OPT_NET_HELP;
use rooch_types::error::RoochResult;
use rooch_types::rooch_network::RoochChainID;
use rustc_hash::FxHashMap as HashMap;
use std::io::{BufWriter, Write};
use std::path::PathBuf;

/// List tx with no execution info
#[derive(Debug, Parser)]
pub struct ListEmptyCommand {
    #[clap(long)]
    pub start_order: Option<u64>,

    #[clap(long, short = 'o')]
    pub output: PathBuf,

    #[clap(long = "data-dir", short = 'd')]
    pub base_data_dir: Option<PathBuf>,

    #[clap(long, short = 'n', help = R_OPT_NET_HELP)]
    pub chain_id: Option<RoochChainID>,
}

impl ListEmptyCommand {
    pub fn execute(self) -> RoochResult<()> {
        let (_root, rooch_db, _start_time) = init(self.base_data_dir, self.chain_id);
        let moveos_store = rooch_db.moveos_store.clone();

        let mut tx_hash_order_map: HashMap<H256, u64> =
            HashMap::with_capacity_and_hasher(100 * 1024 * 1024, Default::default());
        let mut tx_hash_order_dup_map: HashMap<H256, Vec<u64>> =
            HashMap::with_capacity_and_hasher(1024, Default::default());
        let mut non_execution_info_tx_orders = Vec::with_capacity(1024);

        let mut tx_order = self.start_order.unwrap_or(1);
        let mut done_count = 0;
        loop {
            let tx_hashes = rooch_db
                .rooch_store
                .get_transaction_store()
                .get_tx_hashes(vec![tx_order])?;
            if tx_hashes.is_empty() || tx_hashes[0].is_none() {
                tracing::info!("tx_order: {} not found", tx_order);
                break;
            }
            let tx_hash = tx_hashes[0].unwrap();
            let old_value = tx_hash_order_map.insert(tx_hash, tx_order);
            if let Some(old_value) = old_value {
                tx_hash_order_dup_map
                    .entry(tx_hash)
                    .or_default()
                    .push(old_value);
                tx_hash_order_dup_map
                    .get_mut(&tx_hash)
                    .unwrap()
                    .push(tx_order);
            }

            let execution_info = moveos_store
                .get_transaction_store()
                .get_tx_execution_info(tx_hash)?;
            if execution_info.is_none() {
                non_execution_info_tx_orders.push(tx_order);
            }

            done_count += 1;
            if done_count % (1024 * 1024) == 0 {
                tracing::info!("done: {}", done_count);
            }

            tx_order += 1;
        }

        let file = std::fs::File::create(self.output)?;
        let mut writer = BufWriter::with_capacity(8 * 1024 * 1024, file.try_clone()?);
        writeln!(writer, "--- tx with dup hash ---")?;
        let mut dup_count = 0;
        for (tx_hash, orders) in tx_hash_order_dup_map.iter_mut() {
            orders.sort();
            orders.dedup();
            dup_count += 1;
            writeln!(writer, "{:?}: {:?}", tx_hash, orders)?;
        }
        tracing::info!("dup count: {}", dup_count);

        writeln!(writer, "--- tx with no execution info ---")?;
        for order in non_execution_info_tx_orders.iter() {
            writeln!(writer, "{}", order)?;
        }
        tracing::info!(
            "no execution info count: {}",
            non_execution_info_tx_orders.len()
        );

        writer.flush()?;
        file.sync_data()?;

        Ok(())
    }
}
