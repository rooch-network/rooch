// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use clap::Parser;
use rooch_types::error::RoochResult;
use std::path::PathBuf;

/// Sync chunks to destination DA backend.
#[derive(Debug, Parser)]
pub struct SyncCommand {
    #[clap(long = "segment-dir")]
    pub segment_dir: PathBuf,
}

impl SyncCommand {
    pub fn execute(self) -> RoochResult<()> {
        todo!()
    }
}
