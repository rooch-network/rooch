// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::cli_types::{CommandAction, WalletContextOptions};
use async_trait::async_trait;
use clap::Parser;
use rooch_key::keystore::account_keystore::AccountKeystore;
use rooch_key::keystore::types::LocalAccount;
use rooch_rpc_api::jsonrpc_types::{account_view::BalanceInfoView, RoochAddressView};
use rooch_types::{
    address::{ParsedAddress, RoochAddress},
    error::{RoochError, RoochResult},
    rooch_key::ROOCH_SECRET_KEY_HRP,
};
use rooch_types::{
    error::RoochResult,
    rooch_network::{BuiltinChainID, RoochNetwork},
};
use rpassword::prompt_password;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tabled::{
    builder::Builder,
    settings::{object::Columns, Modify, Style, Width},
};

/// show account info, including account addresses, coin and transactions on the Rooch Network.
#[derive(Debug, Parser)]
pub struct ShowCommand {
    #[clap(short = 'a', long = "address", value_parser=ParsedAddress::parse, default_value = "")]
    address: ParsedAddress,

    #[clap(flatten)]
    pub context_options: WalletContextOptions,

    /// Return command outputs in json format
    #[clap(long, default_value = "false")]
    json: bool,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct AccountInfoView {
    pub address: String,
    pub bitcoin_address: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct RoochNetworkAccountView {
    pub account: AccountInfoView,
    pub coins: HashMap<String, BalanceInfoView>,
    pub objects: ObjectInfoView,
    pub transactions: AccountTransactionView,
}

impl RoochNetworkAccountView {
    pub fn from_address(
        address: ParsedAddress,
        balances: Vec<BalanceInfoView>,
    ) -> (Self, anyhow::Error) {
        let account_info_view = AccountInfoView {
            address: ParsedAddress::Numerical(address.into()),
            bitcoin_address: ParsedAddress::Bitcoin(address.into()),
        };
        let mut coins_info_view: HashMap<String, BalanceInfoView> = HashMap::new();
        for balance in balances {
            let key = if coins_info_view.contains_key(&balance.coin_info.symbol) {
                balance.coin_info.coin_type.to_string()
            } else {
                balance.coin_info.symbol.to_string()
            };
            coins_info_view.insert(key, balance);
        }
        RoochNetworkAccountView {
            account: account_info_view,
            coins: coins_info_view,
            objects,
            transactions,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct AccountView {
    #[serde(flatten)]
    pub network_account: RoochNetworkAccountView,
    pub account_exists_in_key_store: bool,
}

pub type AccountsView = HashMap<String, AccountView>;

#[async_trait]
impl CommandAction<Option<AccountsView>> for ShowCommand {
    async fn execute(self) -> RoochResult<Option<AccountsView>> {
        let context = self.context_options.build()?;
        let client = context.get_client().await?;

        let balances = client
            .rooch
            .get_balances(self.address.into(), None, Some(MAX_RESULT_LIMIT))
            .await?
            .data;

        // TODO: objects and transactions

        RoochNetworkAccountView::from_address(self.address, balances);

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
            let mut accounts_view: AccountsView = HashMap::with_capacity(account_views.len());
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
            let mut builder = Builder::default();
            builder.push_record(["Field", "Value", "Active"]);

            for account in account_views {
                let fields = [
                    "Address",
                    "Hex Address",
                    "Bitcoin Address",
                    "Public Key",
                    "Nostr Public Key",
                ];
                let values = [
                    &account.local_account.address,
                    &account.local_account.hex_address,
                    &account.local_account.bitcoin_address,
                    &account.local_account.public_key,
                    &account.local_account.nostr_public_key,
                ];

                let active = if account.active { "True" } else { "False" };

                let mut first_row = true;
                for (field, value) in fields.iter().zip(values.iter()) {
                    if first_row {
                        builder.push_record([*field, &**value, active]);
                        first_row = false;
                    } else {
                        builder.push_record([*field, &**value, ""]);
                    }
                }

                builder.push_record([
                    "─────────────────────────────────",
                    "─────────────────────────────────────────────────────────────────────────",
                    "──────────",
                ]);
            }

            let mut table = builder.build();
            table
                .with(Style::rounded())
                .with(Modify::new(Columns::single(0)).with(Width::truncate(16)));

            println!("{}", table);

            Ok(None)
        }
    }
}
