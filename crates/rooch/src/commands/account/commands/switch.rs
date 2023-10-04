// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::cli_types::{CommandAction, WalletContextOptions};
use async_trait::async_trait;
use clap::Parser;
use rooch_key::keystore::AccountKeystore;
use rooch_types::{
    address::RoochAddress,
    error::{RoochError, RoochResult},
};
use std::{fmt::Debug, str::FromStr};

/// Switch the active Rooch account
#[derive(Debug, Parser)]
pub struct SwitchCommand {
    #[clap(flatten)]
    pub context_options: WalletContextOptions,
    /// The address of the Rooch account to be set as active
    #[clap(short = 'a', long = "address")]
    address: String,
    /// Whether a password should be provided
    #[clap(long = "password")]
    password_required: Option<bool>,
}

#[async_trait]
impl CommandAction<()> for SwitchCommand {
    async fn execute(self) -> RoochResult<()> {
        let mut context = self.context_options.build().await?;
        let rooch_address = RoochAddress::from_str(self.address.as_str()).map_err(|e| {
            RoochError::CommandArgumentError(format!("Invalid Rooch address String: {}", e))
        })?;

        let password = if self.password_required == Some(false) {
            // Use an empty password if not required
            String::new()
        } else {
            // Prompt for a password if required
            rpassword::prompt_password("Enter a password to encrypt the keys in the rooch keystore. Press return to have an empty value: ").unwrap()
        };

        if !context
            .keystore
            .addresses(Some(password))?
            .contains(&rooch_address)
        {
            return Err(RoochError::SwitchAccountError(format!(
                "Address `{}` does not in the Rooch keystore",
                self.address
            )));
        }

        context.client_config.active_address = Some(rooch_address);
        context.client_config.save()?;

        println!(
            "The active account was successfully switched to `{}`",
            self.address
        );

        Ok(())
    }
}
