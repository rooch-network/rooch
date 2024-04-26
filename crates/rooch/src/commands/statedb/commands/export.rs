// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::cli_types::WalletContextOptions;
use clap::Parser;
use rooch_types::error::RoochResult;
use std::path::PathBuf;

/// Export statedb
#[derive(Debug, Parser)]
pub struct ExportCommand {
    #[clap(long, short = 'o')]
    /// export output file. like ~/.rooch/local/utxo.csv or utxo.csv
    pub output: PathBuf,

    #[clap(flatten)]
    pub context_options: WalletContextOptions,
}

impl ExportCommand {
    pub async fn execute(self) -> RoochResult<()> {
        let mut _context = self.context_options.build()?;

        todo!()
    }
}
