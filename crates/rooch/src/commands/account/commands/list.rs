// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::cli_types::{CommandAction, WalletContextOptions};
use async_trait::async_trait;
use clap::Parser;
use rooch_key::keystore::account_keystore::AccountKeystore;
use rooch_key::keystore::types::LocalAccount;
use rooch_types::{
    crypto::EncodeDecodeBase64,
    error::RoochResult,
    rooch_network::{BuiltinChainID, RoochNetwork},
};
use rpassword::prompt_password;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
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

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct LocalAccountView {
    pub address: String,
    pub hex_address: String,
    pub bitcoin_address: String,
    pub public_key: String,
    pub has_session_key: bool,
}

impl LocalAccountView {
    pub fn from_account(account: LocalAccount, btc_network: u8) -> Self {
        LocalAccountView {
            address: account.address.to_bech32(),
            hex_address: account.address.to_hex_literal(),
            bitcoin_address: account
                .bitcoin_address
                .format(btc_network)
                .expect("Failed to format bitcoin address"),
            public_key: account.public_key.encode_base64(),
            has_session_key: account.has_session_key,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct AccountView {
    #[serde(flatten)]
    pub local_account: LocalAccountView,
    pub active: bool,
}

pub type AccountsView = HashMap<String, AccountView>;

#[async_trait]
impl CommandAction<Option<AccountsView>> for ListCommand {
    async fn execute(self) -> RoochResult<Option<AccountsView>> {
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
        let rooch_network: RoochNetwork = context
            .client_config
            .get_active_env()
            .map(|env| env.guess_network())
            .unwrap_or(RoochNetwork::from(BuiltinChainID::Local));
        let account_views: Vec<AccountView> = accounts
            .into_iter()
            .map(|account: LocalAccount| {
                let active = Some(account.address) == active_address;
                AccountView {
                    local_account: LocalAccountView::from_account(
                        account,
                        rooch_network.genesis_config.bitcoin_network,
                    ),
                    active,
                }
            })
            .collect();

        if self.json {
            let mut accounts_view: AccountsView = HashMap::new();
            let mut i = 0;
            for account in account_views {
                if account.active {
                    accounts_view.insert(String::from("default"), account.clone());
                } else {
                    accounts_view.insert(format!("account{}", i), account);
                    i += 1;
                }
            }
            Ok(Some(accounts_view))
        } else {
            let mut output = String::new();

            output.push_str(&format!(
                "{:^66} | {:^66} | {:^48} | {:^10}\n",
                "Address", "Hex Address", "Bitcoin Address", "Active"
            ));
            output.push_str(&format!("{}\n", ["-"; 190].join("")));

            for account in account_views {
                output.push_str(&format!(
                    "{:^66} | {:^66} | {:^48} | {:^10}\n",
                    account.local_account.address,
                    account.local_account.hex_address,
                    account.local_account.bitcoin_address,
                    account.active
                ));
            }
            println!("{}", output);
            Ok(None)
        }
    }
}
