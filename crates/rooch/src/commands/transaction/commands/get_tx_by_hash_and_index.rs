// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::cli_types::{CommandAction, WalletContextOptions};
use async_trait::async_trait;
use clap::Parser;
use moveos_types::h256::H256;
use rooch_rpc_api::jsonrpc_types::eth::Transaction;
use rooch_types::error::RoochResult;

/// Get transaction by hash for Ethereum blockchain
#[derive(Debug, Parser)]
pub struct GetByHashAndIndexCommand {
    /// Transaction's hash
    #[clap(long)]
    pub hash: H256,

    /// Transaction's index
    #[clap(long)]
    pub index: u64,

    #[clap(flatten)]
    pub(crate) context_options: WalletContextOptions,
}

#[async_trait]
impl CommandAction<Transaction> for GetByHashAndIndexCommand {
    async fn execute(self) -> RoochResult<Transaction> {
        let client = self
            .context_options
            .ethereum_build()
            .await?
            .get_client()
            .await?;

        let resp = client
            .transaction_by_hash_and_index(self.hash, self.index)
            .await?;

        Ok(resp)
    }
}
