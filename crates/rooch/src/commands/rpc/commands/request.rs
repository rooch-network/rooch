// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::cli_types::{CommandAction, WalletContextOptions};
use async_trait::async_trait;
use clap::Parser;
use rooch_types::error::RoochResult;

/// Send a RPC request
#[derive(Debug, Parser)]
pub struct RequestCommand {
    /// The RPC method name
    /// --method rooch_getStates
    #[clap(long)]
    pub method: String,

    /// The RPC method params, json value.
    /// --params '"/resource/0x3/0x3::account::Account"'
    /// or
    /// --params '["/resource/0x3/0x3::account::Account", {"decode": true}]'
    #[clap(long)]
    pub params: Option<serde_json::Value>,

    #[clap(flatten)]
    pub(crate) context_options: WalletContextOptions,
}

#[async_trait]
impl CommandAction<serde_json::Value> for RequestCommand {
    async fn execute(self) -> RoochResult<serde_json::Value> {
        let client = self.context_options.build().await?.get_client().await?;
        let params = match self.params {
            Some(serde_json::Value::Array(array)) => array,
            Some(value) => {
                vec![value]
            }
            None => vec![],
        };
        Ok(client.request(self.method.as_str(), params).await?)
    }
}
