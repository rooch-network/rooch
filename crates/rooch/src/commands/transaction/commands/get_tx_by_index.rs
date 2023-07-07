// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::cli_types::{CommandAction, WalletContextOptions};
use async_trait::async_trait;
use rooch_rpc_api::jsonrpc_types::TransactionView;
use rooch_types::error::RoochResult;

/// Get transaction by index
#[derive(Debug, clap::Parser)]
pub struct GetByIndexCommand {
    /// start position
    #[clap(long)]
    pub cursor: u64,

    /// end position
    #[clap(long)]
    pub limit: u64,

    #[clap(flatten)]
    pub(crate) context_options: WalletContextOptions,
}

#[async_trait]
impl CommandAction<Vec<TransactionView>> for GetByIndexCommand {
    async fn execute(self) -> RoochResult<Vec<TransactionView>> {
        let client = self.context_options.build().await?.get_client().await?;

        let resp = client
            .get_transaction_by_index(self.cursor, self.limit)
            .await?;

        Ok(resp)
    }
}
