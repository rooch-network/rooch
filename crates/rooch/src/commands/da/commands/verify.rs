// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::commands::da::commands::LedgerTxGetter;
use clap::Parser;
use rooch_types::error::RoochResult;
use std::path::PathBuf;

/// Verify segments from segments directory.
/// Output: {expected_tx_order},{actual_tx_order},{tx_hash},{block_number}
#[derive(Debug, Parser)]
pub struct VerifyCommand {
    #[clap(long = "segment-dir")]
    pub segment_dir: PathBuf,
    #[clap(long = "start-from", help = "Start from the specified block number")]
    pub start_from: Option<u128>,
}

impl VerifyCommand {
    pub async fn execute(self) -> RoochResult<()> {
        let segment_dir = self.segment_dir;
        let ledger_tx_loader = LedgerTxGetter::new(segment_dir)?;
        let stop_at = ledger_tx_loader.get_max_chunk_id();
        let mut block_number = self.start_from.unwrap_or(0);

        let mut expected_tx_order = 0;

        while block_number <= stop_at {
            let tx_list = ledger_tx_loader
                .load_ledger_tx_list(block_number, true, false)
                .await?;
            let tx_list = tx_list.unwrap();

            if expected_tx_order == 0 {
                expected_tx_order = tx_list.first().unwrap().sequence_info.tx_order;
            }

            for mut ledger_tx in tx_list {
                let tx_order = ledger_tx.sequence_info.tx_order;
                let tx_hash = ledger_tx.tx_hash();
                if tx_order != expected_tx_order {
                    println!(
                        "{},{},{:?},{}",
                        expected_tx_order, tx_order, tx_hash, block_number
                    );
                }
                expected_tx_order += 1;
            }
            block_number += 1;
        }
        Ok(())
    }
}
