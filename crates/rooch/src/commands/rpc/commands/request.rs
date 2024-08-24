// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::cli_types::{CommandAction, WalletContextOptions};
use async_trait::async_trait;
use clap::Parser;
use rooch_types::error::RoochResult;

use rooch_rpc_api::jsonrpc_types::{
    HumanReadableDisplay, IndexerObjectStatePageView, ObjectStateView,
};

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
    /// Return command outputs in json format
    #[clap(long, default_value = "false")]
    json: bool,
}

#[async_trait]
impl CommandAction<serde_json::Value> for RequestCommand {
    async fn execute(self) -> RoochResult<serde_json::Value> {
        let context = self.context_options.build()?;
        let client = context.get_client().await?;
        let params = match self.params {
            Some(serde_json::Value::Array(array)) => array,
            Some(value) => {
                let s = value.as_str().unwrap();
                let ret = serde_json::from_str(s);
                match ret {
                    Ok(value) => value,
                    Err(_) => {
                        vec![serde_json::value::Value::String(s.to_string())]
                    }
                }
            }
            None => vec![],
        };
        let active_env = context.client_config.get_active_env()?;

        Ok(client
            .request_by_proxy(&active_env.rpc, self.method.as_str(), params)
            .await?)
    }

    /// Executes the command, and serializes it to the common JSON output type
    async fn execute_serialized(self) -> RoochResult<String> {
        let method = self.method.clone();
        let json = self.json;
        let result = self.execute().await?;

        if json {
            let output = serde_json::to_string_pretty(&result).unwrap();
            if output == "null" {
                return Ok("".to_string());
            }
            Ok(output)
        } else if method == "rooch_getObjectStates" {
            let view = serde_json::from_value::<Vec<Option<ObjectStateView>>>(result.clone())?
                .into_iter()
                .flatten()
                .collect::<Vec<_>>();
            Ok(view.to_human_readable_string(true, 0))
        } else if method == "rooch_queryObjectStates" {
            Ok(
                serde_json::from_value::<IndexerObjectStatePageView>(result.clone())?
                    .to_human_readable_string(true, 0),
            )
        } else {
            // TODO: handle other rpc methods.
            let output = serde_json::to_string_pretty(&result).unwrap();
            if output == "null" {
                return Ok("".to_string());
            }
            Ok(output)
        }
    }
}
