// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::commands::da::commands::{LedgerTxGetter, TxDAIndex};
use rooch_types::error::{RoochError, RoochResult};
use std::fs::File;
use std::io::BufWriter;
use std::io::Write;
use std::path::PathBuf;

/// Index tx_order:tx_hash:block_number to a file from segments
#[derive(Debug, clap::Parser)]
pub struct IndexCommand {
    #[clap(long = "segment-dir")]
    pub segment_dir: PathBuf,
    #[clap(long = "output")]
    pub output: PathBuf,
}

impl IndexCommand {
    pub fn execute(self) -> RoochResult<()> {
        let ledger_tx_loader = LedgerTxGetter::new(self.segment_dir)?;
        let mut block_number = ledger_tx_loader.get_min_chunk_id();
        let mut expected_tx_order = 0;
        let file = File::create(self.output.clone())?;
        let mut writer = BufWriter::with_capacity(8 * 1024 * 1024, file.try_clone().unwrap());

        loop {
            if block_number > ledger_tx_loader.get_max_chunk_id() {
                break;
            }
            let tx_list = ledger_tx_loader.load_ledger_tx_list(block_number, true)?;
            let tx_list = tx_list.unwrap();
            for mut ledger_tx in tx_list {
                let tx_order = ledger_tx.sequence_info.tx_order;
                let tx_hash = ledger_tx.tx_hash();
                if expected_tx_order == 0 {
                    expected_tx_order = tx_order;
                } else if tx_order != expected_tx_order {
                    return Err(RoochError::from(anyhow::anyhow!(
                        "tx_order mismatch: expected {}, got {}",
                        expected_tx_order,
                        tx_order
                    )));
                }
                writeln!(
                    writer,
                    "{}",
                    TxDAIndex::new(tx_order, tx_hash, block_number)
                )?;
                expected_tx_order += 1;
            }
            block_number += 1;
        }
        writer.flush()?;
        file.sync_data()?;
        Ok(())
    }
}
