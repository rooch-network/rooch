// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::commands::bitseed::inscribe::InscribeOptions;
use crate::commands::bitseed::inscribe::Inscriber;
use clap::Parser;
use rooch_types::bitcoin::ord::InscriptionID;

#[derive(Debug, Parser)]
pub struct SplitCommand {
    #[arg(long, help = "The split SFT inscription ID.")]
    sft_inscription_id: InscriptionID,

    #[arg(long, help = "The split amounts.", num_args = 1..)]
    amounts: Vec<u64>,

    #[clap(flatten)]
    inscribe_options: InscribeOptions,
}

// impl SplitCommand {
//     pub fn run(self, wallet: Wallet) -> SubcommandResult {
//         let output = Inscriber::new(wallet, self.inscribe_options)?
//             .with_split(self.sft_inscription_id, self.amounts)?
//             .inscribe()?;
//         Ok(Box::new(output))
//     }
// }
