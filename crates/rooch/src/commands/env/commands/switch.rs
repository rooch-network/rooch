// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use clap::Parser;
use rooch_types::error::{RoochError, RoochResult};

use crate::cli_types::WalletContextOptions;

#[derive(Debug, Parser)]
pub struct SwitchCommand {
    #[clap(flatten)]
    pub context_options: WalletContextOptions,
    #[clap(long)]
    env: String,
}

impl SwitchCommand {
    pub async fn execute(self) -> RoochResult<()> {
        let mut context = self.context_options.build().await?;
        let env = Some(self.env.clone());

        if context.config.get_env(&env).is_none() {
            return Err(RoochError::SwitchEnvError(format!(
                "The environment config for `{}` does not exist",
                self.env
            )));
        }

        context.config.active_env = env;
        context.config.save()?;

        println!(
            "The active environment was successfully switched to `{}`",
            self.env
        );

        Ok(())
    }
}
