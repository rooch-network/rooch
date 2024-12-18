// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::commands::da::commands::LedgerTxLoader;
use async_trait::async_trait;
use bitcoin::io::Write;
use rooch_types::error::RoochResult;
use rooch_types::transaction::{L1BlockWithBody, LedgerTxData};
use std::fs::File;
use std::io::BufWriter;
use std::path::PathBuf;

/// Get transactions by hashes
#[derive(Debug, clap::Parser)]
pub struct GetTxOrderHashCommand {
    #[clap(long = "segment-dir")]
    pub segment_dir: PathBuf,
    #[clap(long = "output")]
    pub output: PathBuf,
}

#[async_trait]
impl GetTxOrderHashCommand {
    pub fn execute(self) -> RoochResult<()> {
        let ledger_tx_loader = LedgerTxLoader::new(self.segment_dir)?;
        let mut block_number = ledger_tx_loader.get_min_chunk_id();
        let mut expected_tx_order = 0;
        let file = File::create(self.output.clone())?;
        let mut writer = BufWriter::with_capacity(8 * 1024 * 1024, file.try_clone().unwrap());

        loop {
            let tx_list = ledger_tx_loader.load_ledger_tx_list(block_number)?;
            if tx_list.is_none() {
                break;
            }
            let tx_list = tx_list.unwrap();
            for mut ledger_tx in tx_list {
                let tx_order = ledger_tx.sequence_info.tx_order;
                let tx_hash = ledger_tx.tx_hash();
                if expected_tx_order == 0 {
                    expected_tx_order = tx_order;
                } else {
                    if tx_order != expected_tx_order {
                        tracing::error!(
                            "Tx order not expected, expected: {}, actual: {}, tx_hash: {}",
                            expected_tx_order,
                            tx_order,
                            tx_hash
                        );
                    }
                    expected_tx_order += 1;
                }
                writeln!(writer, "{}:{:?}", tx_order, tx_hash)?;
            }
            block_number += 1;
        }
        writer.flush()?;
        file.sync_data()?;
        Ok(())
    }
}
