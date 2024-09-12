// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::commands::bitseed::inscribe::InscribeOptions;
use crate::commands::bitseed::inscribe::Inscriber;
use clap::Parser;
use std::path::PathBuf;

/// Inscribe a new generator bytecode to Bitcoin
#[derive(Debug, Parser)]
pub struct GeneratorCommand {
    #[arg(long, help = "Name of the generator.")]
    name: String,
    #[arg(long, help = "Path to the generator bytecode file.")]
    generator: PathBuf,
    #[clap(flatten)]
    inscribe_options: InscribeOptions,
}

// impl GeneratorCommand {
//     pub fn run(self, wallet: Wallet) -> SubcommandResult {
//         let output = Inscriber::new(wallet, self.inscribe_options)?
//             .with_generator(self.name, self.generator)?
//             .inscribe()?;

//         Ok(Box::new(output))
//     }
// }
