// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::commands::da::commands::TxPositionIndexer;
use rooch_types::error::{RoochError, RoochResult};
use std::path::PathBuf;

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

        let indexer = TxPositionIndexer::new_with_updates(
            db_path,
            reset_from,
            self.segment_dir,
            self.max_block_number,
        )
        .await?;
        indexer.close()?;

        Ok(())
    }

    pub async fn execute(self) -> RoochResult<()> {
        self.exec_inner().await.map_err(RoochError::from)
    }
}
