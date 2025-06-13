// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::cli_types::{CommandAction, WalletContextOptions};
use async_trait::async_trait;
use bitcoin::hex::DisplayHex;
use clap::Parser;
use moveos_types::transaction::MoveAction;
use rooch_rpc_api::jsonrpc_types::transaction_view::LedgerTxDataView;
use rooch_rpc_api::jsonrpc_types::{
    account_view::BalanceInfoView,
    transaction_view::{TransactionFilterView, TransactionWithInfoView},
    H256View, IndexerObjectStateView, ObjectStateFilterView, RoochAddressView, UnitedAddressView,
};
use rooch_rpc_api::jsonrpc_types::{KeptVMStatusView, MoveActionTypeView, ObjectIDView};
use rooch_types::{address::ParsedAddress, error::RoochResult};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tabled::settings::peaker::PriorityRight;
use tabled::settings::{Height, Width};
use tabled::{
    builder::Builder,
    settings::{Panel, Style},
};
use terminal_size::{terminal_size, Height as TerminalHeight, Width as TerminalWidth};

/// Show account info, including account address, coins, objects and transactions on Rooch Network. Requires internet connection and works without rooch init.
#[derive(Debug, Parser)]
pub struct ShowCommand {
    #[clap(short = 'a', long = "address", value_parser=ParsedAddress::parse, default_value = "default")]
    address: ParsedAddress,

    #[clap(flatten)]
    pub context_options: WalletContextOptions,

    #[clap(short = 'l', long = "limit", default_value = "50")]
    limit: u64,

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
                balance.coin_info.coin_type.0.to_canonical_string()
            } else {
                balance.coin_info.symbol.to_owned()
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
            let key = if transaction.clone().execution_info.is_some() {
                transaction.clone().execution_info.unwrap().tx_hash
            } else {
                transaction
                    .clone()
                    .transaction
                    .sequence_info
                    .tx_accumulator_root
            };
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
            .get_balances(rooch_address_view, None, Some(self.limit))
            .await?
            .data;

        // objects
        let united_address_view = UnitedAddressView::from(rooch_address_view);
        let object_state_filter_view = ObjectStateFilterView::Owner(united_address_view.clone());
        let object_states = client
            .rooch
            .query_object_states(object_state_filter_view, None, Some(self.limit), None)
            .await?
            .data;

        // transactions
        let transaction_filter_view = TransactionFilterView::Sender(united_address_view);
        let transactions = client
            .rooch
            .query_transactions(transaction_filter_view, None, Some(self.limit), None)
            .await?
            .data;

        // rooch network account info from input address
        let rooch_network_account_view = RoochNetworkAccountView::from_account(
            rooch_address_view,
            balances.clone(),
            object_states.clone(),
            transactions.clone(),
        );

        if self.json {
            Ok(Some(rooch_network_account_view))
        } else {
            let mut formatted_account_info_header = vec![];
            let mut formatted_account_info = vec![];
            let mut formatted_balance_info_header = vec![];
            let mut formatted_balance_info = vec![];
            let mut formatted_indexer_object_state_header = vec![];
            let mut formatted_indexer_object_state = vec![];
            let mut formatted_transaction_with_info_header = vec![];
            let mut formatted_transaction_with_info = vec![];

            // terminal
            let (width, height) = get_terminal_size();

            // account
            let mut account_info_builder = Builder::default();
            formatted_account_info_header.push("Rooch Address".to_owned());
            formatted_account_info.push(rooch_address.to_bech32());
            account_info_builder.push_record(formatted_account_info_header);
            account_info_builder.push_record(formatted_account_info);
            let mut account_info_table = account_info_builder.build();
            account_info_table
                .with(Panel::header("Account Info"))
                .with(Style::rounded())
                .with(Width::wrap(width).priority(PriorityRight::new()))
                .with(Width::increase(width))
                .with(Height::limit(height))
                .with(Height::increase(height))
                .to_string();

            // coins
            let mut balance_info_builder = Builder::default();
            formatted_balance_info_header.push("Name".to_owned());
            formatted_balance_info_header.push("Symbol".to_owned());
            formatted_balance_info_header.push("Icon Url".to_owned());
            formatted_balance_info_header.push("Decimals".to_owned());
            formatted_balance_info_header.push("Supply".to_owned());
            formatted_balance_info_header.push("Balance".to_owned());
            for balance_info in balances {
                formatted_balance_info
                    .push(balance_info.coin_info.coin_type.0.to_canonical_string());
                formatted_balance_info.push(balance_info.coin_info.name);
                formatted_balance_info.push(balance_info.coin_info.symbol);
                formatted_balance_info.push(balance_info.coin_info.icon_url.unwrap_or_default());
                formatted_balance_info.push(balance_info.coin_info.decimals.to_string());
                formatted_balance_info.push(balance_info.coin_info.supply.0.to_string());
                formatted_balance_info.push(balance_info.balance.0.to_string());
            }
            balance_info_builder.push_record(formatted_balance_info_header);
            balance_info_builder.push_record(formatted_balance_info);
            let mut balance_info_table = balance_info_builder.build();
            balance_info_table
                .with(Panel::header("Coins"))
                .with(Style::rounded())
                .with(Width::wrap(width).priority(PriorityRight::new()))
                .with(Width::increase(width))
                .with(Height::limit(height))
                .with(Height::increase(height))
                .to_string();

            // objects
            let mut indexer_object_state_builder = Builder::default();
            formatted_indexer_object_state_header.push("Id".to_owned());
            formatted_indexer_object_state_header.push("Owner".to_owned());
            formatted_indexer_object_state_header.push("Owner Bitcoin Address".to_owned());
            formatted_indexer_object_state_header.push("Flag".to_owned());
            formatted_indexer_object_state_header.push("State Root".to_owned());
            formatted_indexer_object_state_header.push("Size".to_owned());
            formatted_indexer_object_state_header.push("Created At".to_owned());
            formatted_indexer_object_state_header.push("Updated At".to_owned());
            formatted_indexer_object_state_header.push("Object Type".to_owned());
            formatted_indexer_object_state_header.push("Value".to_owned());
            formatted_indexer_object_state_header.push("Decoded Value".to_owned());
            formatted_indexer_object_state_header.push("Tx Order".to_owned());
            formatted_indexer_object_state_header.push("State Index".to_owned());
            for object_state in object_states {
                formatted_indexer_object_state.push(object_state.metadata.id.to_hex());
                formatted_indexer_object_state.push(object_state.metadata.owner.0.to_bech32());
                formatted_indexer_object_state.push(
                    object_state
                        .metadata
                        .owner_bitcoin_address
                        .unwrap_or_default(),
                );
                formatted_indexer_object_state.push(object_state.metadata.flag.to_string());
                formatted_indexer_object_state.push(
                    object_state
                        .metadata
                        .state_root
                        .unwrap_or_default()
                        .0
                        .to_string(),
                );
                formatted_indexer_object_state.push(object_state.metadata.size.0.to_string());
                formatted_indexer_object_state.push(object_state.metadata.created_at.0.to_string());
                formatted_indexer_object_state.push(object_state.metadata.updated_at.0.to_string());
                formatted_indexer_object_state
                    .push(object_state.metadata.object_type.0.to_canonical_string());
                formatted_indexer_object_state.push(object_state.value.0.as_hex().to_string());
                formatted_indexer_object_state.push(
                    object_state
                        .decoded_value
                        .unwrap_or_default()
                        .as_str()
                        .unwrap_or_default()
                        .to_string(),
                );
                formatted_indexer_object_state.push(object_state.indexer_id.tx_order.0.to_string());
                formatted_indexer_object_state
                    .push(object_state.indexer_id.state_index.0.to_string());
                if object_state.display_fields.is_none() {
                    formatted_indexer_object_state_header.push("Display Fields".to_owned());
                    formatted_indexer_object_state.push("".to_owned());
                } else {
                    formatted_indexer_object_state_header.push("Display Fields Key".to_owned());
                    formatted_indexer_object_state_header.push("Display Fields Value".to_owned());
                    let display_fields = object_state.display_fields.unwrap();
                    for field in display_fields.fields {
                        formatted_indexer_object_state.push(field.0);
                        formatted_indexer_object_state.push(field.1);
                    }
                }
            }
            indexer_object_state_builder.push_record(formatted_indexer_object_state_header);
            indexer_object_state_builder.push_record(formatted_indexer_object_state);
            let mut indexer_object_state_table = indexer_object_state_builder.build();
            indexer_object_state_table
                .with(Panel::header("Objects"))
                .with(Style::rounded())
                .with(Width::wrap(width).priority(PriorityRight::new()))
                .with(Width::increase(width))
                .with(Height::limit(height))
                .with(Height::increase(height))
                .to_string();

            // transactions
            let mut transaction_with_info_builder = Builder::default();
            for transaction in transactions {
                match transaction.transaction.data {
                    LedgerTxDataView::L1Block(block) => {
                        formatted_transaction_with_info_header.push("Chain Id".to_owned());
                        formatted_transaction_with_info_header.push("Block Height".to_owned());
                        formatted_transaction_with_info_header.push("Block Hash".to_owned());
                        formatted_transaction_with_info_header
                            .push("Bitcoin Block Hash".to_owned());
                        formatted_transaction_with_info.push(block.chain_id.0.to_string());
                        formatted_transaction_with_info.push(block.block_height.0.to_string());
                        formatted_transaction_with_info
                            .push(block.block_hash.0.as_hex().to_string());
                        formatted_transaction_with_info
                            .push(block.bitcoin_block_hash.unwrap_or_default());
                    }
                    LedgerTxDataView::L1Tx(tx) => {
                        formatted_transaction_with_info_header.push("Chain Id".to_owned());
                        formatted_transaction_with_info_header.push("Block Hash".to_owned());
                        formatted_transaction_with_info_header
                            .push("Bitcoin Block Hash".to_owned());
                        formatted_transaction_with_info_header.push("Txid".to_owned());
                        formatted_transaction_with_info_header.push("Bitcoin Txid".to_owned());
                        formatted_transaction_with_info.push(tx.chain_id.0.to_string());
                        formatted_transaction_with_info.push(tx.block_hash.0.as_hex().to_string());
                        formatted_transaction_with_info
                            .push(tx.bitcoin_block_hash.unwrap_or_default());
                        formatted_transaction_with_info.push(tx.txid.0.as_hex().to_string());
                        formatted_transaction_with_info.push(tx.bitcoin_txid.unwrap_or_default());
                    }
                    LedgerTxDataView::L2Tx(tx) => {
                        formatted_transaction_with_info_header.push("Sequence Number".to_owned());
                        formatted_transaction_with_info_header.push("Sender".to_owned());
                        formatted_transaction_with_info_header
                            .push("Sender Bitcoin Address".to_owned());
                        formatted_transaction_with_info_header.push("Action Type".to_owned());
                        formatted_transaction_with_info_header.push("Action".to_owned());
                        formatted_transaction_with_info_header.push("Raw".to_owned());
                        formatted_transaction_with_info_header.push("Chain Id".to_owned());
                        formatted_transaction_with_info_header.push("Max Gas Amount".to_owned());
                        formatted_transaction_with_info.push(tx.sequence_number.0.to_string());
                        formatted_transaction_with_info.push(tx.sender);
                        formatted_transaction_with_info
                            .push(tx.sender_bitcoin_address.unwrap_or_default());
                        match tx.action_type {
                            MoveActionTypeView::ScriptCall => {
                                formatted_transaction_with_info.push("scriptcall".to_owned());
                            }
                            MoveActionTypeView::FunctionCall => {
                                formatted_transaction_with_info.push("functioncall".to_owned());
                            }
                            MoveActionTypeView::ModuleBundle => {
                                formatted_transaction_with_info.push("modulebundle".to_owned());
                            }
                        };
                        let move_action = MoveAction::from(tx.action);
                        let encoded_move_action_bytes = move_action.encode()?;
                        formatted_transaction_with_info
                            .push(encoded_move_action_bytes.as_hex().to_string()); // with hex string of encoded bytes, not the original object
                        formatted_transaction_with_info.push(tx.raw.0.as_hex().to_string());
                        formatted_transaction_with_info.push(tx.chain_id.0.to_string());
                        formatted_transaction_with_info.push(tx.max_gas_amount.0.to_string());
                    }
                }

                formatted_transaction_with_info_header.push("Tx Order".to_owned());
                formatted_transaction_with_info_header.push("Tx Order Signature".to_owned());
                formatted_transaction_with_info_header.push("Tx Accumulator Root".to_owned());
                formatted_transaction_with_info_header.push("Tx Timestamp".to_owned());
                formatted_transaction_with_info
                    .push(transaction.transaction.sequence_info.tx_order.0.to_string());
                formatted_transaction_with_info.push(
                    transaction
                        .transaction
                        .sequence_info
                        .tx_order_signature
                        .0
                        .as_hex()
                        .to_string(),
                );
                formatted_transaction_with_info.push(
                    transaction
                        .transaction
                        .sequence_info
                        .tx_accumulator_root
                        .0
                        .to_string(),
                );
                formatted_transaction_with_info.push(
                    transaction
                        .transaction
                        .sequence_info
                        .tx_timestamp
                        .0
                        .to_string(),
                );

                if transaction.execution_info.is_none() {
                    formatted_transaction_with_info_header.push("Execution Info".to_owned());
                    formatted_transaction_with_info.push("".to_owned());
                } else {
                    formatted_transaction_with_info_header.push("Tx Hash".to_owned());
                    formatted_transaction_with_info_header.push("State Root".to_owned());
                    formatted_transaction_with_info_header.push("Event Root".to_owned());
                    formatted_transaction_with_info_header.push("Gas Used".to_owned());
                    let execution_info = transaction.execution_info.unwrap();
                    formatted_transaction_with_info.push(execution_info.tx_hash.0.to_string());
                    formatted_transaction_with_info.push(execution_info.state_root.0.to_string());
                    formatted_transaction_with_info.push(execution_info.event_root.0.to_string());
                    formatted_transaction_with_info.push(execution_info.gas_used.0.to_string());

                    formatted_transaction_with_info_header.push("Status".to_owned());
                    match execution_info.status {
                        KeptVMStatusView::Executed => {
                            formatted_transaction_with_info.push("executed".to_owned());
                        }
                        KeptVMStatusView::OutOfGas => {
                            formatted_transaction_with_info.push("outofgas".to_owned());
                        }
                        KeptVMStatusView::MoveAbort {
                            location,
                            abort_code,
                        } => {
                            formatted_transaction_with_info.push("moveabort".to_owned());
                        }
                        KeptVMStatusView::ExecutionFailure {
                            location,
                            function,
                            code_offset,
                        } => {
                            formatted_transaction_with_info.push("executionfailure".to_owned());
                        }
                        KeptVMStatusView::MiscellaneousError => {
                            formatted_transaction_with_info.push("miscellaneouserror".to_owned());
                        }
                    }
                }
            }
            transaction_with_info_builder.push_record(formatted_transaction_with_info_header);
            transaction_with_info_builder.push_record(formatted_transaction_with_info);
            let mut transaction_with_info_table = transaction_with_info_builder.build();
            transaction_with_info_table
                .with(Panel::header("Transactions"))
                .with(Style::rounded())
                .with(Width::wrap(width).priority(PriorityRight::new()))
                .with(Width::increase(width))
                .with(Height::limit(height))
                .with(Height::increase(height))
                .to_string();

            println!("{}", account_info_table);
            println!("{}", balance_info_table);
            println!("{}", indexer_object_state_table);
            println!("{}", transaction_with_info_table);

            Ok(None)
        }
    }
}

fn get_terminal_size() -> (usize, usize) {
    let (TerminalWidth(width), TerminalHeight(height)) =
        terminal_size().expect("failed to obtain a terminal size");

    (width as usize, height as usize)
}
