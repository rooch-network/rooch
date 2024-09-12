// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use clap::Parser;
use rooch_types::bitcoin::ord::InscriptionID;

#[derive(Debug, Parser)]
pub struct ViewCommand {
    #[arg(long, help = "The SFT inscription ID to view.")]
    sft_inscription_id: InscriptionID,
}

// impl ViewCommand {
//     pub fn run(self, wallet: Wallet) -> SubcommandResult {
//         let operation = wallet.get_operation_by_inscription_id(self.sft_inscription_id)?;
//         let sft = match operation {
//             Operation::Mint(mint_record) => mint_record.as_sft(),
//             Operation::Split(split_record) => split_record.as_sft(),
//             Operation::Merge(merge_record) => merge_record.as_sft(),
//             _ => bail!(
//                 "Inscription {} is not a valid SFT record",
//                 self.sft_inscription_id
//             ),
//         };

//         Ok(Box::new(sft))
//     }
// }
