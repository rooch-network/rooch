// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::cli_types::WalletContextOptions;
use clap::{Parser, ValueHint};
use rooch_rpc_client::client_config::Env;
use rooch_types::error::RoochResult;
use std::time::Duration;

/// Add a new Rooch environment
#[derive(Debug, Parser)]
pub struct AddCommand {
    #[clap(flatten)]
    pub context_options: WalletContextOptions,
    #[clap(long)]
    pub alias: String,
    #[clap(long, value_hint = ValueHint::Url)]
    pub rpc: String,
    #[clap(long, value_hint = ValueHint::Url)]
    pub ws: Option<String>,
}

impl AddCommand {
    pub async fn execute(self) -> RoochResult<()> {
        let mut context = self.context_options.build().await?;
        let AddCommand { alias, rpc, ws, .. } = self;
        let env = Env {
            ws,
            rpc,
            alias: alias.clone(),
        };

        // TODO: is this request timeout okay?
        env.create_rpc_client(Duration::from_secs(5), None).await?;
        context.client_config.add_env(env);
        context.client_config.save()?;

        println!("Environment `{} was successfully added", alias);

        Ok(())
    }
}
