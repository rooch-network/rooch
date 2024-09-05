// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::cli_types::{CommandAction, TransactionFilterOptions, WalletContextOptions};
use async_trait::async_trait;
use clap::Parser;
use rooch_rpc_api::jsonrpc_types::transaction_view::TransactionFilterView;
use rooch_rpc_api::jsonrpc_types::{
    H256View, QueryOptions, RoochAddressView, TransactionWithInfoPageView,
};
use rooch_rpc_client::wallet_context::WalletContext;
use rooch_types::error::{RoochError, RoochResult};

/// Query transactions
#[derive(Debug, Parser)]
pub struct QueryCommand {
    #[clap(flatten)]
    pub filter: TransactionFilterOptions,

    #[clap(long)]
    pub cursor: Option<u64>,

    #[clap(long)]
    pub limit: Option<u64>,

    /// Descending order
    #[clap(short = 'd', long, default_value = "false")]
    pub descending_order: bool,

    /// Render and return display fields
    #[clap(long, default_value = "false")]
    pub show_display: bool,

    /// If true, filter out all match items
    #[clap(long, default_value = "false")]
    pub filter_out: bool,

    #[clap(flatten)]
    pub(crate) context_options: WalletContextOptions,
}

#[async_trait]
impl CommandAction<TransactionWithInfoPageView> for QueryCommand {
    async fn execute(self) -> RoochResult<TransactionWithInfoPageView> {
        let context = self.context_options.build()?;
        let client = context.get_client().await?;

        let filter_view = convert_to_filter_view(self.filter, &context)?;
        let query_options = QueryOptions {
            decode: true,
            filter_out: self.filter_out,
            descending: self.descending_order,
            show_display: self.show_display,
        };

        let resp = client
            .rooch
            .query_transactions(filter_view, self.cursor, self.limit, Some(query_options))
            .await?;

        Ok(resp)
    }
}

fn convert_to_filter_view(
    options: TransactionFilterOptions,
    context: &WalletContext,
) -> RoochResult<TransactionFilterView> {
    if let Some(sender) = options.sender {
        let rooch_address_view: RoochAddressView = sender
            .into_rooch_address(&context.address_mapping())
            .unwrap()
            .into();

        Ok(TransactionFilterView::Sender(rooch_address_view.into()))
    } else if let Some(tx_hashes) = options.tx_hashes {
        Ok(TransactionFilterView::TxHashes(
            tx_hashes.into_iter().map(H256View::from).collect(),
        ))
    } else if options.start_time.is_some() && options.end_time.is_some() {
        Ok(TransactionFilterView::TimeRange {
            start_time: options.start_time.unwrap().into(),
            end_time: options.end_time.unwrap().into(),
        })
    } else if options.from_order.is_some() && options.to_order.is_some() {
        Ok(TransactionFilterView::TxOrderRange {
            from_order: options.from_order.unwrap().into(),
            to_order: options.to_order.unwrap().into(),
        })
    } else {
        Err(RoochError::CommandArgumentError(
            "Invalid transaction filter options".to_string(),
        ))
    }
}
