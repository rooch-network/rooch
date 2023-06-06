// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::types::{CommandAction, WalletContextOptions};
use async_trait::async_trait;
use rooch_types::{error::RoochResult, transaction::TypedTransaction};

#[derive(Debug, clap::Parser)]
pub struct GetByIndexCommand {
    #[clap(long)]
    pub start: u64,
    #[clap(long)]
    pub limit: u64,

    #[clap(flatten)]
    pub(crate) context_options: WalletContextOptions,
}

#[async_trait]
impl CommandAction<Option<Vec<TypedTransaction>>> for GetByIndexCommand {
    async fn execute(self) -> RoochResult<Option<Vec<TypedTransaction>>> {
        let client = self.context_options.build().await?.get_client().await?;

        let resp = client
            .get_transaction_by_index(self.start, self.limit)
            .await?;

        Ok(resp)
    }
}
