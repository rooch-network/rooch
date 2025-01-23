// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::commands::da::commands::{LedgerTxGetter, TxPosition, TxPositionIndexer};
use anyhow::anyhow;
use rooch_types::error::{RoochError, RoochResult};
use std::cmp::max;
use std::path::PathBuf;
use tracing::info;

/// Index tx_order:tx_hash:block_number
#[derive(Debug, clap::Parser)]
pub struct IndexCommand {
    #[clap(long = "segment-dir", short = 's')]
    pub segment_dir: Option<PathBuf>,
    #[clap(long = "index", short = 'i')]
    pub index_path: PathBuf,
    #[clap(
        long = "reset-from",
        help = "Reset from tx order(inclusive), all tx orders after this will be re-indexed"
    )]
    pub reset_from: Option<u64>,
    #[clap(long = "max-block-number", help = "Max block number to index")]
    pub max_block_number: Option<u128>,
    #[clap(long = "file", help = "Load/dump file-based index")]
    pub index_file_path: Option<PathBuf>,
    #[clap(long = "dump", help = "Dump index to file")]
    pub dump: bool,
}

impl IndexCommand {
    async fn exec_inner(self) -> anyhow::Result<()> {
        if self.index_file_path.is_some() {
            return TxPositionIndexer::load_or_dump(
                self.index_path,
                self.index_file_path.unwrap(),
                self.dump,
            );
        }

        let db_path = self.index_path.clone();
        let reset_from = self.reset_from;
        let mut indexer = TxPositionIndexer::new(db_path, reset_from)?;
        let stats_before_reset = indexer.get_stats()?;
        info!("indexer stats after reset: {:?}", stats_before_reset);

        let segment_dir = self.segment_dir.ok_or(anyhow!("segment-dir is required"))?;
        let ledger_tx_loader = LedgerTxGetter::new(segment_dir)?;
        let stop_at = if let Some(max_block_number) = self.max_block_number {
            max(max_block_number, ledger_tx_loader.get_max_chunk_id())
        } else {
            ledger_tx_loader.get_max_chunk_id()
        };
        let mut block_number = indexer.last_block_number; // avoiding partial indexing
        let mut expected_tx_order = indexer.last_tx_order + 1;
        let mut done_block = 0;

        while block_number <= stop_at {
            let tx_list = ledger_tx_loader
                .load_ledger_tx_list(block_number, true)
                .await?;
            let tx_list = tx_list.unwrap();
            {
                let db = indexer.db;
                let mut wtxn = indexer.db_env.write_txn()?;
                for mut ledger_tx in tx_list {
                    let tx_order = ledger_tx.sequence_info.tx_order;
                    if tx_order < expected_tx_order {
                        continue;
                    }
                    if tx_order == indexer.last_tx_order + 1 {
                        info!(
                            "begin to index block: {}, tx_order: {}",
                            block_number, tx_order
                        );
                    }
                    if tx_order != expected_tx_order {
                        return Err(anyhow!(
                            "tx_order not continuous, expect: {}, got: {}",
                            expected_tx_order,
                            tx_order
                        ));
                    }
                    let tx_hash = ledger_tx.tx_hash();
                    let tx_position = TxPosition {
                        tx_order,
                        tx_hash,
                        block_number,
                    };
                    db.put(&mut wtxn, &tx_order, &tx_position)?;
                    expected_tx_order += 1;
                }
                wtxn.commit()?;
            }
            block_number += 1;
            done_block += 1;
            if done_block % 1000 == 0 {
                info!(
                    "done: block_cnt: {}; next_block_number: {}",
                    done_block, block_number
                );
            }
        }

        indexer.init_cursor()?;
        info!("indexer stats after job: {:?}", indexer.get_stats()?);
        indexer.close()?;

        Ok(())
    }

    pub async fn execute(self) -> RoochResult<()> {
        self.exec_inner().await.map_err(RoochError::from)
    }
}
