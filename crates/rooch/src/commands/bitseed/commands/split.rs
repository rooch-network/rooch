// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::{
    cli_types::{CommandAction, WalletContextOptions},
    commands::bitseed::inscribe::{InscribeOptions, InscribeOutput, Inscriber},
};
use async_trait::async_trait;
use clap::Parser;
use rooch_types::{bitcoin::ord::InscriptionID, error::RoochResult};

#[derive(Debug, Parser)]
pub struct SplitCommand {
    #[arg(long, help = "The split SFT inscription ID.")]
    sft_inscription_id: InscriptionID,

    #[arg(long, help = "The split amounts.", num_args = 1..)]
    amounts: Vec<u64>,

    #[clap(flatten)]
    inscribe_options: InscribeOptions,

    #[clap(flatten)]
    pub context_options: WalletContextOptions,
}

#[async_trait]
impl CommandAction<InscribeOutput> for SplitCommand {
    async fn execute(self) -> RoochResult<InscribeOutput> {
        let context = self.context_options.build_require_password()?;
        let output = Inscriber::new(context, self.inscribe_options)
            .await?
            .with_split(self.sft_inscription_id, self.amounts)
            .await?
            .inscribe()
            .await?;

        Ok(output)
    }
}
