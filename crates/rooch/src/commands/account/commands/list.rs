// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::cli_types::{CommandAction, WalletContextOptions};
use async_trait::async_trait;
use clap::Parser;
use serde::{Deserialize, Serialize};
use rooch_key::keystore::types::LocalAccount;
use rooch_key::keystore::account_keystore::AccountKeystore;
use rooch_types::{crypto::EncodeDecodeBase64, error::RoochResult};
use rpassword::prompt_password;
use std::fmt::Debug;

/// List all keys by its Rooch address, Base64 encoded public key
#[derive(Debug, Parser)]
pub struct ListCommand {
    #[clap(flatten)]
    pub context_options: WalletContextOptions,

    /// Return command outputs in json format
    #[clap(long, default_value = "false")]
    json: bool,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct AccountView {
    pub local_account: LocalAccount,
    pub active: bool
}

#[async_trait]
impl CommandAction<()> for ListCommand {
    async fn execute(self) -> RoochResult<()> {
        let context = self.context_options.build()?;
        let active_address = context.client_config.active_address;

        let password = if context.keystore.get_if_password_is_empty() {
            None
        } else {
            Some(
                prompt_password("Enter the password to create a new key pair:").unwrap_or_default(),
            )
        };

        let accounts: Vec<LocalAccount> = context.keystore.get_accounts(password)?;

        if self.json {
            let accont_views: Vec<AccountView> = accounts.into_iter().map(| account : LocalAccount| {
                AccountView {
                    local_account: account.clone(),
                    active: Some(account.address) == active_address
                }
            }).collect();

            println!("{}", serde_json::to_string_pretty(&accont_views).unwrap());
            return Ok(())
        }

        println!(
            "{:^66} | {:^66} | {:^48} | {:^16} | {:^12}",
            "Rooch Address (Ed25519)",
            "Multichain Address",
            "Public Key (Base64)",
            "Has session key",
            "Active Address"
        );
        println!("{}", ["-"; 153].join(""));

        for account in accounts {
            let address = account.address;
            let active = if active_address == Some(address) {
                "True"
            } else {
                ""
            };

            println!(
                "{:^66} | {:^66} | {:^48} | {:^16} | {:^12}",
                address,
                account
                    .multichain_address
                    .map(|multichain_address| multichain_address.to_string())
                    .unwrap_or_default(),
                account
                    .public_key
                    .map(|public_key| public_key.encode_base64())
                    .unwrap_or_default(),
                account.has_session_key.to_string(),
                active
            );
        }

        Ok(())
    }
}
