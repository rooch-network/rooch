// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::types::{CommandAction, WalletContextOptions};
use async_trait::async_trait;
use hex::FromHex;
use rooch_server::jsonrpc_types::TransactionView;
use rooch_types::{error::RoochResult, H256};

#[derive(Debug, clap::Parser)]
pub struct GetByHashCommand {
    #[clap(long)]
    pub hash: String,

    // filter?
    // pub options:...
    #[clap(flatten)]
    pub(crate) context_options: WalletContextOptions,
}

#[async_trait]
impl CommandAction<Option<TransactionView>> for GetByHashCommand {
    async fn execute(self) -> RoochResult<Option<TransactionView>> {
        let client = self.context_options.build().await?.get_client().await?;

        let hex_string = if self.hash.starts_with("0x") {
            &self.hash[2..]
        } else {
            &self.hash
        };

        let bytes = Vec::from_hex(hex_string).expect("invalid hex string");
        let resp = client
            .get_transaction_by_hash(H256::from_slice(&bytes))
            .await?;

        Ok(resp)
    }
}
