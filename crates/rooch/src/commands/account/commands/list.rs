// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::cli_types::{CommandAction, WalletContextOptions};
use async_trait::async_trait;
use clap::Parser;
use rooch_key::keystore::account_keystore::AccountKeystore;
use rooch_types::{crypto::EncodeDecodeBase64, error::RoochResult};
use rpassword::prompt_password;
use std::fmt::Debug;

/// List all keys by its Rooch address, Base64 encoded public key
#[derive(Debug, Parser)]
pub struct ListCommand {
    #[clap(flatten)]
    pub context_options: WalletContextOptions,
}

#[async_trait]
impl CommandAction<()> for ListCommand {
    async fn execute(self) -> RoochResult<()> {
        let context = self.context_options.build().await?;
        let active_address = context.client_config.active_address;

        let password = if context.keystore.get_if_password_is_empty() {
            None
        } else {
            Some(
                prompt_password("Enter the password to create a new key pair:").unwrap_or_default(),
            )
        };

        println!(
            "{:^66} | {:^48} | {:^16} | {:^12}",
            "Rooch Address (Ed25519)", "Public Key (Base64)", "Auth Validator ID", "Active Address"
        );
        println!("{}", ["-"; 153].join(""));

        for (address, public_key) in context.keystore.get_address_public_keys(password)? {
            let auth_validator_id = public_key.auth_validator().flag();
            let active = if active_address == Some(address) {
                "True"
            } else {
                ""
            };

            println!(
                "{:^66} | {:^48} | {:^16} | {:^12}",
                address,
                public_key.encode_base64(),
                auth_validator_id.to_string(),
                active
            );
        }

        Ok(())
    }
}
