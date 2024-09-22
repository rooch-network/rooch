// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::cli_types::CommandAction;
use crate::cli_types::WalletContextOptions;
use crate::commands::bitseed::inscribe::InscribeOptions;
use crate::commands::bitseed::inscribe::InscribeOutput;
use crate::commands::bitseed::inscribe::Inscriber;
use async_trait::async_trait;
use clap::Parser;
use rooch_types::error::RoochResult;
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
    #[clap(flatten)]
    pub context_options: WalletContextOptions,
}

#[async_trait]
impl CommandAction<InscribeOutput> for GeneratorCommand {
    async fn execute(self) -> RoochResult<InscribeOutput> {
        let context = self.context_options.build_require_password()?;
        let output = Inscriber::new(context, self.inscribe_options)
            .await?
            .with_generator(self.name, self.generator)
            .await?
            .inscribe()
            .await?;

        Ok(output)
    }
}
