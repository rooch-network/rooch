// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::cli_types::{CommandAction, WalletContextOptions};
use async_trait::async_trait;
use rooch_rpc_api::jsonrpc_types::TransactionPageViewResult;
use rooch_types::error::RoochResult;

/// Get transactions by order
#[derive(Debug, clap::Parser)]
pub struct GetTransactionsByOrderCommand {
    /// Transaction's hash
    #[clap(long)]
    pub cursor: Option<u128>,

    #[clap(long)]
    pub limit: Option<u64>,

    #[clap(flatten)]
    pub(crate) context_options: WalletContextOptions,
}

#[async_trait]
impl CommandAction<TransactionPageViewResult> for GetTransactionsByOrderCommand {
    async fn execute(self) -> RoochResult<TransactionPageViewResult> {
        let client = self.context_options.build().await?.get_client().await?;

        let resp = client
            .rooch
            .get_transactions_by_order(self.cursor, self.limit)
            .await?;

        Ok(resp)
    }
}
