// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::cli_types::CommandAction;
use crate::cli_types::WalletContextOptions;
use crate::commands::bitseed::inscribe::InscribeOptions;
use crate::commands::bitseed::inscribe::InscribeOutput;
use crate::commands::bitseed::inscribe::Inscriber;
use async_trait::async_trait;
use clap::Parser;
use rooch_types::bitcoin::ord::InscriptionID;
use rooch_types::error::RoochResult;

#[derive(Debug, Parser)]
pub struct MergeCommand {
    #[arg(long, help = "The merge SFT inscription IDs.")]
    sft_inscription_ids: Vec<InscriptionID>,

    #[clap(flatten)]
    inscribe_options: InscribeOptions,

    #[clap(flatten)]
    pub context_options: WalletContextOptions,
}

#[async_trait]
impl CommandAction<InscribeOutput> for MergeCommand {
    async fn execute(self) -> RoochResult<InscribeOutput> {
        let context = self.context_options.build_require_password()?;
        let output = Inscriber::new(context, self.inscribe_options)
            .await?
            .with_merge(self.sft_inscription_ids)
            .await?
            .inscribe()
            .await?;

        Ok(output)
    }
}
