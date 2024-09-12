// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::commands::bitseed::inscribe::InscribeOptions;
use crate::commands::bitseed::inscribe::Inscriber;
use clap::Parser;
use rooch_types::bitcoin::ord::InscriptionID;

#[derive(Debug, Parser)]
pub struct MergeCommand {
    #[arg(long, help = "The merge SFT inscription IDs.")]
    sft_inscription_ids: Vec<InscriptionID>,

    #[clap(flatten)]
    inscribe_options: InscribeOptions,
}

// impl MergeCommand {
//     pub fn run(self, wallet: Wallet) -> SubcommandResult {
//         let output = Inscriber::new(wallet, self.inscribe_options)?
//             .with_merge(self.sft_inscription_ids)?
//             .inscribe()?;
//         Ok(Box::new(output))
//     }
// }
