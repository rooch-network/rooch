// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use clap::Parser;
use rooch_types::error::{RoochError, RoochResult};

use crate::cli_types::WalletContextOptions;

#[derive(Debug, Parser)]
pub struct RemoveCommand {
    #[clap(flatten)]
    pub context_options: WalletContextOptions,
    #[clap(long)]
    env: String,
}

impl RemoveCommand {
    pub async fn execute(self) -> RoochResult<()> {
        let mut context = self.context_options.rooch_build().await?;
        if let Some(active_env) = &context.config.active_env {
            if active_env == &self.env {
                return Err(RoochError::RemoveEnvError(
                    "Cannot remove the currently active environment. Please switch to another environment and try again".to_owned()
                ));
            }
        }

        context.config.envs.retain(|env| env.alias != self.env);
        context.config.save()?;

        println!("Environment `{}` was successfully removed", self.env);

        Ok(())
    }
}
