// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::cli_types::{CommandAction, WalletContextOptions};
use async_trait::async_trait;
use rooch_rpc_api::jsonrpc_types::transaction_view::TransactionResultView;
use rooch_types::{error::RoochResult, H256};

/// Get transactions by hashes
#[derive(Debug, clap::Parser)]
pub struct GetTransactionsByHashCommand {
    /// Transaction's hashes
    #[clap(long, value_delimiter = ',')]
    pub hashes: Vec<H256>,

    #[clap(flatten)]
    pub(crate) context_options: WalletContextOptions,
}

#[async_trait]
impl CommandAction<Vec<Option<TransactionResultView>>> for GetTransactionsByHashCommand {
    async fn execute(self) -> RoochResult<Vec<Option<TransactionResultView>>> {
        let client = self.context_options.build().await?.get_client().await?;

        let resp = client.get_transactions_by_hash(self.hashes).await?;

        Ok(resp)
    }
}
