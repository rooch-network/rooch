// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::commands::da::commands::LedgerTxGetter;
use clap::Parser;
use rooch_types::error::RoochResult;
use std::path::PathBuf;

/// List accumulator anomalies
#[derive(Debug, Parser)]
pub struct AccumulatorAnomalyCommand {
    #[clap(long = "segment-dir")]
    pub segment_dir: PathBuf,
    #[clap(long = "start-from", help = "Start from the specified block number")]
    pub start_from: Option<u128>,
}

impl AccumulatorAnomalyCommand {
    pub async fn execute(self) -> RoochResult<()> {
        let segment_dir = self.segment_dir;
        let ledger_tx_loader = LedgerTxGetter::new(segment_dir)?;
        let stop_at = ledger_tx_loader.get_max_chunk_id();
        let mut block_number = self.start_from.unwrap_or(0);

        let mut expected_number_leaves: u64 = 1;
        let mut number_leaves_mismatched_count: u64 = 0;

        println!(
            "exp_number_leaves,act_number_leaves,tx_order,tx_hash,block_number,mismatched_count,timestamp"
        );

        while block_number <= stop_at {
            let tx_list = ledger_tx_loader
                .load_ledger_tx_list(block_number, true, false)
                .await?;
            let tx_list = tx_list.unwrap();

            if expected_number_leaves == 1 {
                expected_number_leaves = tx_list
                    .first()
                    .unwrap()
                    .sequence_info
                    .tx_accumulator_num_leaves;
            }

            for mut ledger_tx in tx_list {
                let number_leaves = ledger_tx.sequence_info.tx_accumulator_num_leaves;
                let tx_hash = ledger_tx.tx_hash();
                let tx_order = ledger_tx.sequence_info.tx_order;
                if number_leaves != expected_number_leaves + number_leaves_mismatched_count {
                    number_leaves_mismatched_count += 1;
                    println!(
                        "{},{},{},{:?},{},{},{}",
                        expected_number_leaves,
                        number_leaves,
                        tx_order,
                        tx_hash,
                        block_number,
                        number_leaves_mismatched_count,
                        ledger_tx.sequence_info.tx_timestamp
                    );
                }
                expected_number_leaves += 1;
            }
            block_number += 1;
        }
        Ok(())
    }
}
