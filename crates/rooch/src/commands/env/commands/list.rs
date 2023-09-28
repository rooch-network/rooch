// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use clap::Parser;
use rooch_types::error::RoochResult;

use crate::cli_types::WalletContextOptions;

#[derive(Debug, Parser)]
pub struct ListCommand {
    #[clap(flatten)]
    pub context_options: WalletContextOptions,
}

impl ListCommand {
    pub async fn execute(self) -> RoochResult<()> {
        let context = self.context_options.build().await?;

        println!(
            "{:^24} | {:^48} | {:^48} | {:^12}",
            "Env Alias", "RPC URL", "Websocket URL", "Active Env"
        );
        println!("{}", ["-"; 153].join(""));

        for env in context.client_config.envs.iter() {
            let mut active = "";
            if context.client_config.active_env == Some(env.alias.clone()) {
                active = "True"
            }

            let ws = env.ws.clone().unwrap_or("Null".to_owned());
            println!(
                "{:^24} | {:^48} | {:^48} | {:^12}",
                env.alias, env.rpc, ws, active
            )
        }

        Ok(())
    }
}
