// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::cli_types::{CommandAction, WalletContextOptions};
use async_trait::async_trait;
use clap::Parser;
use moveos_types::module_binding::MoveFunctionCaller;
use rooch_rpc_api::jsonrpc_types::{StateOptions, StatePageView};
use rooch_types::address::ParsedAddress;
use rooch_types::error::{RoochError, RoochResult};
use rooch_types::framework::session_key::SessionKeyModule;

/// List all session keys by address
#[derive(Debug, Parser)]
pub struct ListCommand {
    #[clap(short = 'a', long = "address", value_parser=ParsedAddress::parse, default_value = "default")]
    /// The account's address to list session keys, if absent, show the default active account.
    address: ParsedAddress,

    #[clap(flatten)]
    pub context_options: WalletContextOptions,

    /// Display output as a table instead of JSON
    #[clap(long)]
    pub json: bool,
}

#[async_trait]
impl CommandAction<Vec<serde_json::Value>> for ListCommand {
    async fn execute(self) -> RoochResult<Vec<serde_json::Value>> {
        let context = self.context_options.build()?;
        let mapping = context.address_mapping();
        let address_addr = self.address.into_account_address(&mapping)?;

        let client = context.get_client().await?;
        let session_key_module = client.as_module_binding::<SessionKeyModule>();
        let obj_id = session_key_module
            .get_session_keys_handle(address_addr)?
            .ok_or_else(|| {
                RoochError::ViewFunctionError("Failed to get session keys object".to_string())
            })?;

        let options = StateOptions::new().decode(true);
        let field_result = client
            .rooch
            .list_field_states(obj_id.into(), None, None, Some(options))
            .await
            .map_err(RoochError::from)?;

        Ok(extract_session_keys(field_result))
    }
}

fn extract_session_keys(field_result: StatePageView) -> Vec<serde_json::Value> {
    let mut result = vec![];
    for data in field_result.data {
        if let Some(decoded_value) = data.state.decoded_value {
            if let Some(value) = decoded_value.get("value") {
                result.push(value.clone());
            }
        }
    }
    result
}
