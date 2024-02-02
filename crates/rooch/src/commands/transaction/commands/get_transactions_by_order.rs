// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::cli_types::{CommandAction, WalletContextOptions};
use async_trait::async_trait;
use rooch_rpc_api::jsonrpc_types::TransactionWithInfoPageView;
use rooch_types::error::RoochResult;

/// Get transactions by order
#[derive(Debug, clap::Parser)]
pub struct GetTransactionsByOrderCommand {
    /// Transaction's hash
    #[clap(long)]
    pub cursor: Option<u64>,

    #[clap(long)]
    pub limit: Option<u64>,

    /// descending order
    #[clap(short = 'd', long)]
    descending_order: Option<bool>,

    #[clap(flatten)]
    pub(crate) context_options: WalletContextOptions,
}

#[async_trait]
impl CommandAction<TransactionWithInfoPageView> for GetTransactionsByOrderCommand {
    async fn execute(self) -> RoochResult<TransactionWithInfoPageView> {
        let client = self.context_options.build()?.get_client().await?;

        let resp = client
            .rooch
            .get_transactions_by_order(self.cursor, self.limit, self.descending_order)
            .await?;

        Ok(resp)
    }
}
