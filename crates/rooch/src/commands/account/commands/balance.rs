// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::cli_types::{CommandAction, WalletContextOptions};
use async_trait::async_trait;
use clap::Parser;
use move_command_line_common::types::ParsedStructType;
use move_core_types::u256::U256;
use rooch_rpc_api::api::MAX_RESULT_LIMIT;
use rooch_rpc_api::jsonrpc_types::account_view::BalanceInfoView;
use rooch_rpc_api::jsonrpc_types::btc::utxo::UTXOFilterView;
use rooch_rpc_api::jsonrpc_types::{
    IndexerStateIDView, RoochAddressView, StrView, UnitedAddressView,
};
use rooch_rpc_client::Client;
use rooch_types::address::ParsedAddress;
use rooch_types::error::RoochResult;
use schemars::JsonSchema;
use serde::Serialize;
use std::collections::HashMap;
use tabled::{
    builder::Builder,
    settings::{Style, Width},
};

/// Show account balance, only the accounts managed by the current node are supported
#[derive(Debug, Parser)]
pub struct BalanceCommand {
    #[clap(short = 'a', long = "address", value_parser=ParsedAddress::parse, default_value = "default")]
    /// The account's address to show balance, if absent, show the default active account.
    address: ParsedAddress,

    /// Struct name as `<ADDRESS>::<MODULE_ID>::<STRUCT_NAME><TypeParam>`
    #[clap(long, value_parser=ParsedStructType::parse)]
    coin_type: Option<ParsedStructType>,

    #[clap(flatten)]
    pub context_options: WalletContextOptions,

    /// Return command outputs in json format
    #[clap(long, default_value = "false")]
    json: bool,
}

pub type BalancesView = HashMap<String, BalanceInfoViewUnion>;

#[derive(Debug, Clone, Serialize, JsonSchema)]
#[serde(untagged)]
pub enum BalanceInfoViewUnion {
    Bitcoin(BitcoinBalanceInfoView),
    Other(BalanceInfoView),
}

#[derive(Debug, Clone, Serialize, JsonSchema)]
pub struct BitcoinCoinInfoView {
    name: String,
    symbol: String,
    decimals: u8,
}

#[derive(Debug, Clone, Serialize, JsonSchema)]
pub struct BitcoinBalanceInfoView {
    #[serde(flatten)]
    coin_info: BitcoinCoinInfoView,
    balance: StrView<U256>,
}

impl From<BitcoinBalanceInfoView> for BalanceInfoViewUnion {
    fn from(view: BitcoinBalanceInfoView) -> Self {
        BalanceInfoViewUnion::Bitcoin(view)
    }
}

impl From<BalanceInfoView> for BalanceInfoViewUnion {
    fn from(view: BalanceInfoView) -> Self {
        BalanceInfoViewUnion::Other(view)
    }
}

#[async_trait]
impl CommandAction<Option<BalancesView>> for BalanceCommand {
    async fn execute(self) -> RoochResult<Option<BalancesView>> {
        let context = self.context_options.build()?;
        let mapping = context.address_mapping();
        let address_addr = self.address.clone().into_account_address(&mapping)?;
        let rooch_address = self.address.clone().into_rooch_address(&mapping)?;
        let coin_type = self
            .coin_type
            .map(|t| t.into_struct_tag(&mapping))
            .transpose()?;
        let client = context.get_client().await?;

        let mut balances: Vec<BalanceInfoViewUnion> = Vec::new();

        let btc_balance = get_bitcoin_balance(&client, rooch_address.into()).await?;
        balances.push(BalanceInfoViewUnion::Bitcoin(btc_balance));

        let other_balances = match coin_type {
            Some(coin_type) => vec![
                client
                    .rooch
                    .get_balance(address_addr.into(), coin_type.into())
                    .await?,
            ],
            None => {
                client
                    .rooch
                    .get_balances(address_addr.into(), None, Some(MAX_RESULT_LIMIT))
                    .await?
                    .data
            }
        };

        balances.extend(
            other_balances
                .iter()
                .cloned()
                .map(BalanceInfoViewUnion::Other),
        );

        if self.json {
            let mut balances_view: BalancesView = HashMap::new();
            for balance_info in &balances {
                match balance_info {
                    BalanceInfoViewUnion::Bitcoin(bitcoin_balance) => {
                        balances_view.insert(
                            bitcoin_balance.coin_info.symbol.to_string(),
                            BalanceInfoViewUnion::Bitcoin(bitcoin_balance.clone()),
                        );
                    }
                    BalanceInfoViewUnion::Other(other_balance) => {
                        let key = if balances_view.contains_key(&other_balance.coin_info.symbol) {
                            other_balance.coin_info.coin_type.to_string()
                        } else {
                            other_balance.coin_info.symbol.to_string()
                        };
                        balances_view
                            .insert(key, BalanceInfoViewUnion::Other(other_balance.clone()));
                    }
                }
            }
            Ok(Some(balances_view))
        } else {
            let mut formatted_balances = vec![];
            for balance_info in &balances {
                match balance_info {
                    BalanceInfoViewUnion::Bitcoin(bitcoin_balance) => {
                        formatted_balances.push((
                            "Bitcoin".to_string(),
                            bitcoin_balance.coin_info.symbol.clone(),
                            bitcoin_balance.coin_info.decimals,
                            bitcoin_balance.balance.to_string(),
                        ));
                    }
                    BalanceInfoViewUnion::Other(other_balance) => {
                        formatted_balances.push((
                            other_balance.coin_info.coin_type.to_string(),
                            other_balance.coin_info.symbol.clone(),
                            other_balance.coin_info.decimals,
                            other_balance.balance.to_string(),
                        ));
                    }
                }
            }
            print_balance_info_table(formatted_balances);
            Ok(None)
        }
    }
}

async fn get_bitcoin_balance(
    client: &Client,
    address: RoochAddressView,
) -> RoochResult<BitcoinBalanceInfoView> {
    let total_balance: u64 = get_total_utxo_value(client, UnitedAddressView::from(address)).await?;
    let bitcoin_info = BitcoinCoinInfoView {
        name: "Bitcoin".to_string(),
        symbol: "BTC".to_string(),
        decimals: 8,
    };
    Ok(BitcoinBalanceInfoView {
        coin_info: bitcoin_info,
        balance: StrView(U256::from(total_balance)),
    })
}

async fn get_total_utxo_value(
    client: &Client,
    address: UnitedAddressView,
) -> Result<u64, anyhow::Error> {
    let mut total_value: u64 = 0;
    let mut cursor: Option<IndexerStateIDView> = None;

    loop {
        let page = client
            .rooch
            .query_utxos(
                UTXOFilterView::Owner(address.clone()),
                cursor.map(Into::into),
                Some(MAX_RESULT_LIMIT),
                None,
            )
            .await?;

        for utxo in page.data {
            total_value = total_value
                .checked_add(utxo.value.value())
                .ok_or_else(|| anyhow::anyhow!("UTXO value overflow"))?;
        }

        if !page.has_next_page {
            break;
        }

        cursor = page.next_cursor;
    }

    Ok(total_value)
}

fn print_balance_info_table(balances: Vec<(String, String, u8, String)>) {
    let mut builder = Builder::default();
    builder.push_record(["Coin Type", "Symbol", "Decimals", "Balance"]);

    for (coin_type, symbol, decimals, balance) in balances {
        builder.push_record([&coin_type, &symbol, &decimals.to_string(), &balance]);
    }

    let mut table = builder.build();
    table.with(Style::rounded()).with(Width::increase(20));

    println!("{}", table);
}
