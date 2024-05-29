// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::cli_types::{CommandAction, WalletContextOptions};
use async_trait::async_trait;
use clap::Parser;
use rooch_key::keystore::account_keystore::AccountKeystore;
use rooch_key::keystore::types::LocalAccount;
use rooch_types::{
    address::MultiChainAddress,
    crypto::{EncodeDecodeBase64, PublicKey},
    error::RoochResult,
};
use rpassword::prompt_password;
use serde::{Deserialize, Serialize};
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
pub struct LocalAccountView {
    pub address: String,
    pub hex_address: String,
    pub multichain_address: Option<MultiChainAddress>,
    pub public_key: Option<PublicKey>,
    pub has_session_key: bool,
}

impl From<LocalAccount> for LocalAccountView {
    fn from(account: LocalAccount) -> Self {
        LocalAccountView {
            address: account.address.to_bech32(),
            hex_address: account.address.to_hex_literal(),
            multichain_address: account.multichain_address,
            public_key: account.public_key,
            has_session_key: account.has_session_key,
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct AccountView {
    pub local_account: LocalAccountView,
    pub active: bool,
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
        let account_views: Vec<AccountView> = accounts
            .into_iter()
            .map(|account: LocalAccount| {
                let active = Some(account.address) == active_address;
                AccountView {
                    local_account: account.into(),
                    active,
                }
            })
            .collect();

        if self.json {
            println!("{}", serde_json::to_string_pretty(&account_views).unwrap());
        } else {
            //TODO optimize the output format
            println!(
                "{:^66} | {:^66} | {:^48} | {:^48} | {:^10} | {:^10}",
                "Address (bech32)",
                "Address (hex)",
                "Multichain Address",
                "Public Key(base64)",
                "Session key",
                "Active"
            );
            println!("{}", ["-"; 68].join(""));

            for account in account_views {
                println!(
                    "{:^66} | {:^66} | {:^48} | {:^48} | {:^10} | {:^10}",
                    account.local_account.address,
                    account.local_account.hex_address,
                    account
                        .local_account
                        .multichain_address
                        .map(|multichain_address| multichain_address.to_string())
                        .unwrap_or_default(),
                    account
                        .local_account
                        .public_key
                        .map(|public_key| public_key.encode_base64())
                        .unwrap_or_default(),
                    account.local_account.has_session_key.to_string(),
                    account.active
                );
            }
        }

        Ok(())
    }
}
