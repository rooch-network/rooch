// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::cli_types::{CommandAction, WalletContextOptions};
use async_trait::async_trait;
use clap::Parser;
use rooch_rpc_api::api::MAX_RESULT_LIMIT;
use rooch_rpc_api::jsonrpc_types::ObjectIDView;
use rooch_rpc_api::jsonrpc_types::{
    account_view::BalanceInfoView,
    transaction_view::{TransactionFilterView, TransactionWithInfoView},
    H256View, IndexerObjectStateView, ObjectStateFilterView, RoochAddressView, UnitedAddressView,
};
use rooch_types::{address::ParsedAddress, error::RoochResult};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
// use tabled::{
//     builder::Builder,
//     settings::{Style, Width},
// };

/// Show account info, including account address, coins, objects and transactions on Rooch Network. Works without rooch init.
#[derive(Debug, Parser)]
pub struct ShowCommand {
    #[clap(short = 'a', long = "address", value_parser=ParsedAddress::parse, default_value = "default")]
    address: ParsedAddress,

    #[clap(flatten)]
    pub context_options: WalletContextOptions,

    /// Return command outputs in json format
    #[clap(long, default_value = "false")]
    json: bool,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct AccountInfoView {
    pub address: RoochAddressView,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct RoochNetworkAccountView {
    pub account: AccountInfoView,
    pub coins: HashMap<String, BalanceInfoView>,
    pub objects: HashMap<ObjectIDView, IndexerObjectStateView>,
    pub transactions: HashMap<H256View, TransactionWithInfoView>,
}

impl RoochNetworkAccountView {
    pub fn from_account(
        address: RoochAddressView,
        balances: Vec<BalanceInfoView>,
        objects: Vec<IndexerObjectStateView>,
        transactions: Vec<TransactionWithInfoView>,
    ) -> Self {
        // account info view with rooch address
        let account_info_view = AccountInfoView { address };
        // coin info view with coin type or symbol string and balance info view
        let mut coin_info_view: HashMap<String, BalanceInfoView> = HashMap::new();
        for balance in balances {
            let key = if coin_info_view.contains_key(&balance.coin_info.symbol) {
                balance.coin_info.coin_type.to_string()
            } else {
                balance.coin_info.symbol.to_string()
            };
            coin_info_view.insert(key, balance);
        }
        // object info view with object id and indexer object state view
        let mut object_info_view: HashMap<ObjectIDView, IndexerObjectStateView> = HashMap::new();
        for object in objects {
            let key = object.clone().metadata.id;
            object_info_view.insert(key.into(), object);
        }
        // transaction info view with transaction hash or transaction accumulator root and transaction with info view
        let mut transaction_info_view: HashMap<H256View, TransactionWithInfoView> = HashMap::new();
        for transaction in transactions {
            let key;
            if transaction.clone().execution_info.is_some() {
                key = transaction.clone().execution_info.unwrap().tx_hash;
            } else {
                key = transaction
                    .clone()
                    .transaction
                    .sequence_info
                    .tx_accumulator_root
            }
            transaction_info_view.insert(key, transaction);
        }
        RoochNetworkAccountView {
            account: account_info_view,
            coins: coin_info_view,
            objects: object_info_view,
            transactions: transaction_info_view,
        }
    }
}

#[async_trait]
impl CommandAction<Option<RoochNetworkAccountView>> for ShowCommand {
    async fn execute(self) -> RoochResult<Option<RoochNetworkAccountView>> {
        let context = self.context_options.build()?;
        let client = context.get_client().await?;
        let mapping = context.address_mapping();
        let rooch_address = self.address.into_rooch_address(&mapping)?;
        let rooch_address_view = RoochAddressView::from(rooch_address);

        // balances
        let balances = client
            .rooch
            .get_balances(rooch_address_view, None, Some(MAX_RESULT_LIMIT))
            .await?
            .data;

        // objects
        let united_address_view = UnitedAddressView::from(rooch_address_view);
        let object_state_filter_view = ObjectStateFilterView::Owner(united_address_view.clone());
        let object_states = client
            .rooch
            .query_object_states(object_state_filter_view, None, Some(MAX_RESULT_LIMIT), None)
            .await?
            .data;

        // transactions
        let transaction_filter_view = TransactionFilterView::Sender(united_address_view);
        let transactions = client
            .rooch
            .query_transactions(transaction_filter_view, None, Some(MAX_RESULT_LIMIT), None)
            .await?
            .data;

        // rooch network account info from input address
        let rooch_network_account_view = RoochNetworkAccountView::from_account(
            rooch_address_view,
            balances,
            object_states,
            transactions,
        );

        if self.json {
            Ok(Some(rooch_network_account_view))
        } else {
            // TODO: tabled print
            // let mut formatted_account_info = vec![];
            // // address
            // formatted_account_info.push((rooch_address, balances, object_states, transactions));
            // print_account_info_table(formatted_account_info);
            Ok(None)
        }
    }
}

// fn print_account_info_table(
//     account_info: Vec<(
//         RoochAddress,
//         Vec<BalanceInfoView>,
//         Vec<IndexerObjectStateView>,
//         Vec<TransactionWithInfoView>,
//     )>,
// ) {
//     let mut builder = Builder::default();
//     builder.push_record(["Address", "Coins", "Objects", "Transactions"]);

//     for (address, coins, objects, transactions) in account_info {
//         builder.push_record([&address, &coins, &objects, &transactions]);
//     }

//     let mut table = builder.build();
//     table.with(Style::rounded()).with(Width::increase(20));

//     println!("{}", table);
// }
