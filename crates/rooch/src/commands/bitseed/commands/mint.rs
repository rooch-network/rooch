// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::commands::bitseed::inscribe::InscribeOptions;
use crate::commands::bitseed::inscribe::Inscriber;
use clap::Parser;
use rooch_types::bitcoin::ord::InscriptionID;

#[derive(Debug, Parser)]
pub struct MintCommand {
    #[arg(long, help = "The deploy inscription id.")]
    deploy_inscription_id: InscriptionID,

    #[arg(long, help = "The user input argument to the generator.")]
    user_input: Option<String>,

    #[clap(flatten)]
    inscribe_options: InscribeOptions,
}

// impl MintCommand {
//     pub fn run(self, wallet: Wallet) -> SubcommandResult {
//         let output = Inscriber::new(wallet, self.inscribe_options)?
//             .with_mint(self.deploy_inscription_id, self.user_input)?
//             .inscribe()?;
//         Ok(Box::new(output))
//     }
// }
