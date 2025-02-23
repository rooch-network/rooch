// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::utils::{derive_builtin_genesis_namespace, open_rooch_db};
use clap::Parser;
use moveos_store::transaction_store::TransactionStore;
use moveos_types::h256::H256;
use rooch_config::R_OPT_NET_HELP;
use rooch_pipeline_processor::actor::TxAnomalies;
use rooch_types::error::RoochResult;
use rooch_types::rooch_network::{BuiltinChainID, RoochChainID};
use rustc_hash::FxHashMap;
use std::collections::HashMap;
use std::path::PathBuf;

/// List tx anomalies
#[derive(Debug, Parser)]
pub struct ListAnomaly {
    #[clap(long)]
    pub start_order: Option<u64>,
    #[clap(long, short = 'o')]
    pub output: PathBuf,
    #[clap(long, help = "pretty/binary output")]
    pub pretty: bool,

    #[clap(long = "data-dir", short = 'd')]
    pub base_data_dir: PathBuf,
    #[clap(long, short = 'n', help = R_OPT_NET_HELP)]
    pub chain_id: BuiltinChainID,
}

impl ListAnomaly {
    pub fn execute(self) -> RoochResult<()> {
        let (_root, rooch_db, _start_time) = open_rooch_db(
            Some(self.base_data_dir),
            Some(RoochChainID::Builtin(self.chain_id)),
        );
        let moveos_store = rooch_db.moveos_store.clone();
        let genesis_namespace = derive_builtin_genesis_namespace(self.chain_id)?;

        let mut tx_hash_order_map: FxHashMap<H256, u64> =
            FxHashMap::with_capacity_and_hasher(1024 * 1024 * 100, Default::default());

        let mut dup_hash: HashMap<H256, Vec<u64>> = HashMap::with_capacity(1024);
        let mut no_execution_info: HashMap<H256, u64> = HashMap::with_capacity(1024);

        let mut tx_order = self.start_order.unwrap_or(1);
        let mut done_count: u64 = 0;
        let mut dup_count: u64 = 0;

        let report_interval = 1024 * 1024;
        let mut dup_count_interval: u64 = 0;
        let mut no_execution_count_interval: u64 = 0;

        loop {
            let tx_hashes = rooch_db
                .rooch_store
                .get_transaction_store()
                .get_tx_hashes(vec![tx_order])?;
            if tx_hashes.is_empty() || tx_hashes[0].is_none() {
                tracing::info!("tx_order: {} not found, reach the end", tx_order);
                break;
            }
            let tx_hash = tx_hashes[0].unwrap();
            let previous_order_opt = tx_hash_order_map.insert(tx_hash, tx_order);
            if let Some(previous_order) = previous_order_opt {
                dup_hash.entry(tx_hash).or_default();

                // Ensure no duplicate tx_order
                let orders = dup_hash.get_mut(&tx_hash).unwrap();
                if !orders.contains(&previous_order) {
                    orders.push(previous_order);
                }
                if !orders.contains(&tx_order) {
                    orders.push(tx_order);
                }

                dup_count_interval += 1;
                dup_count += 1;
            }

            let execution_info = moveos_store
                .get_transaction_store()
                .get_tx_execution_info(tx_hash)?;
            if execution_info.is_none() {
                no_execution_info.insert(tx_hash, tx_order);
                no_execution_count_interval += 1;
            }

            done_count += 1;
            if done_count % report_interval == 0 {
                tracing::info!(
                    "done: {}. tx_order range: [{},{}], tx_hash_dup: {}, no_exec_info: {}",
                    done_count,
                    tx_order - 1 - report_interval,
                    tx_order - 1,
                    dup_count_interval,
                    no_execution_count_interval
                );
                dup_count_interval = 0;
                no_execution_count_interval = 0;
            }

            tx_order += 1;
        }

        let no_execution_info_count = no_execution_info.len();

        let tx_anomalies = TxAnomalies {
            genesis_namespace,
            dup_hash,
            no_execution_info,
        };

        if self.pretty {
            tx_anomalies.save_plain_text_to(self.output)?;
        } else {
            tx_anomalies.save_to(self.output)?;
        }

        tracing::info!(
            "done: {}. tx_order range: [{},{}], tx_hash_dup_count: {}, no_exec_info_count: {}",
            done_count,
            self.start_order.unwrap_or(1),
            tx_order - 1,
            dup_count,
            no_execution_info_count
        );

        Ok(())
    }
}
