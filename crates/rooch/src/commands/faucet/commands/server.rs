// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::cli_types::{CommandAction, WalletContextOptions};
use async_trait::async_trait;
use clap::Parser;
use rooch_faucet::{DiscordConfig, FaucetConfig, WebConfig};
use rooch_types::error::RoochResult;

#[derive(Parser)]
#[clap(
    name = "Rooch Faucet server",
    about = "Faucet for requesting RGas on Rooch",
    rename_all = "kebab-case"
)]
pub struct ServerCommand {
    #[clap(flatten)]
    pub web_config: WebConfig,

    #[clap(flatten)]
    pub faucet_config: FaucetConfig,

    #[clap(flatten)]
    pub discord_config: DiscordConfig,

    #[clap(flatten)]
    pub context_options: WalletContextOptions,
}

#[async_trait]
impl CommandAction<String> for ServerCommand {
    async fn execute(self) -> RoochResult<String> {
        let ServerCommand {
            web_config,
            faucet_config,
            discord_config,
            context_options,
        } = self;
        let wallet_context = context_options.build_require_password()?;
        Ok(
            rooch_faucet::server::start(wallet_context, web_config, faucet_config, discord_config)
                .await?,
        )
    }
}
