// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::cli_types::{CommandAction, WalletContextOptions};
use async_trait::async_trait;
use bitcoin::hex::DisplayHex;
use clap::Parser;
use moveos_types::transaction::MoveAction;
use rooch_rpc_api::jsonrpc_types::{
    transaction_view::{LedgerTxDataView, TransactionFilterView, TransactionWithInfoView},
    H256View, KeptVMStatusView, MoveActionTypeView, RoochAddressView, UnitedAddressView,
};
use rooch_types::{address::ParsedAddress, error::RoochResult};
use std::collections::HashMap;
use tabled::{
    builder::Builder,
    settings::{peaker::PriorityRight, Height, Panel, Style, Width},
};
use terminal_size::{terminal_size, Height as TerminalHeight, Width as TerminalWidth};

/// List transactions of a holding account on Rooch Network. Requires internet connection and works without rooch init.
#[derive(Debug, Parser)]
pub struct TxCommand {
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

pub type TransactionResultView = HashMap<H256View, TransactionWithInfoView>;

#[async_trait]
impl CommandAction<Option<TransactionResultView>> for TxCommand {
    async fn execute(self) -> RoochResult<Option<TransactionResultView>> {
        let context = self.context_options.build()?;
        let client = context.get_client().await?;
        let mapping = context.address_mapping();
        let rooch_address = self.address.into_rooch_address(&mapping)?;
        let rooch_address_view = RoochAddressView::from(rooch_address);
        let united_address_view = UnitedAddressView::from(rooch_address_view);

        // transactions
        let transaction_filter_view = TransactionFilterView::Sender(united_address_view);
        let transactions = client
            .rooch
            .query_transactions(transaction_filter_view, None, Some(self.limit), None)
            .await?
            .data;

        // transaction result view with transaction hash view or transaction accumulator root view and transaction with info view
        let mut transaction_result_view: HashMap<H256View, TransactionWithInfoView> =
            HashMap::new();
        for transaction in &transactions {
            let key = if transaction.execution_info.is_some() {
                transaction.execution_info.as_ref().unwrap().tx_hash
            } else {
                transaction.transaction.sequence_info.tx_accumulator_root
            };
            transaction_result_view.insert(key, transaction.clone());
        }

        if self.json {
            Ok(Some(transaction_result_view))
        } else {
            let mut formatted_transaction_with_info_header = vec![];
            let mut formatted_transaction_with_info = vec![];

            // terminal
            let (width, height) = get_terminal_size();

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
                        #[allow(unused_variables)]
                        KeptVMStatusView::MoveAbort {
                            location,
                            abort_code,
                        } => {
                            formatted_transaction_with_info.push("moveabort".to_owned());
                        }
                        #[allow(unused_variables)]
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
