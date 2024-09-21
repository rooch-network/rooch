// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::cli_types::CommandAction;
use crate::cli_types::WalletContextOptions;
use crate::commands::bitseed::inscribe::InscribeOptions;
use crate::commands::bitseed::inscribe::InscribeOutput;
use crate::commands::bitseed::inscribe::Inscriber;
use crate::commands::bitseed::operation::deploy_args_cbor_encode;
use async_trait::async_trait;
use clap::Parser;
use rooch_types::bitcoin::ord::InscriptionID;
use rooch_types::error::RoochResult;

#[derive(Debug, Parser)]
pub struct DeployCommand {
    #[arg(long, help = "The SFT tick name.")]
    tick: String,

    #[arg(long, help = "The amount of the tick total supply.")]
    amount: u64,

    #[arg(long, help = "The generator Inscription id on Bitcoin.")]
    generator: Option<InscriptionID>,

    #[arg(long, help = "The mint factory name.")]
    factory: Option<String>,

    #[arg(
        long,
        help = "The number of allowed the SFT attributes repeats. 0 means do not limit.",
        default_value = "0"
    )]
    repeat: u64,

    #[arg(long, help = "The deploy arguments to the generator program.")]
    deploy_args: Vec<String>,

    #[clap(flatten)]
    inscribe_options: InscribeOptions,

    #[clap(flatten)]
    pub context_options: WalletContextOptions,
}

#[async_trait]
impl CommandAction<InscribeOutput> for DeployCommand {
    async fn execute(self) -> RoochResult<InscribeOutput> {
        let context = self.context_options.build_require_password()?;
        //TODO how to encode the factory args.
        let deploy_args = deploy_args_cbor_encode(self.deploy_args)?;

        //TODO check the tick name is valid
        let tick = self.tick.to_uppercase();
        let output = Inscriber::new(context, self.inscribe_options)
            .await?
            .with_deploy(
                tick,
                self.amount,
                self.generator,
                self.factory,
                self.repeat,
                deploy_args,
            )
            .await?
            .inscribe()
            .await?;
        Ok(output)
    }
}
