// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use clap::Parser;
use rooch_types::error::RoochResult;
use tabled::{builder::Builder, settings::Style};

use crate::cli_types::WalletContextOptions;

#[derive(Debug, Parser)]
pub struct ListCommand {
    #[clap(flatten)]
    pub context_options: WalletContextOptions,
}

impl ListCommand {
    pub async fn execute(self) -> RoochResult<()> {
        let context = self.context_options.build()?;
        let mut builder = Builder::default();

        builder.push_record(["Env Alias", "RPC URL", "Websocket URL", "Active Env"]);

        for env in context.client_config.envs.iter() {
            let active = if context.client_config.active_env == Some(env.alias.clone()) {
                "True"
            } else {
                ""
            };
            let ws = env.ws.clone().unwrap_or_else(|| "Null".to_owned());

            builder.push_record([&env.alias, &env.rpc, &ws, active]);
        }

        let mut table = builder.build();
        table.with(Style::rounded());

        println!("{}", table);

        Ok(())
    }
}
