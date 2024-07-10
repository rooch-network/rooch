// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::cli_types::{CommandAction, WalletContextOptions};
use async_trait::async_trait;
use clap::Parser;
use move_core_types::identifier::Identifier;
use moveos_types::move_types::FunctionId;
use moveos_types::moveos_std::object::ObjectID;
use moveos_types::transaction::FunctionCall;
use rooch_rpc_api::jsonrpc_types::{
    AnnotatedFunctionResultView, AnnotatedMoveValueView, StateOptions, StatePageView,
};
use rooch_types::address::ParsedAddress;
use rooch_types::error::{RoochError, RoochResult};
use std::collections::BTreeMap;

/// List all session keys by address
#[derive(Debug, Parser)]
pub struct ListCommand {
    #[clap(short = 'a', long = "address", value_parser=ParsedAddress::parse, default_value = "default")]
    /// The account's address to list session keys, if absent, show the default active account.
    address: ParsedAddress,

    #[clap(flatten)]
    pub context_options: WalletContextOptions,
}

#[async_trait]
impl CommandAction<Vec<BTreeMap<Identifier, AnnotatedMoveValueView>>> for ListCommand {
    async fn execute(self) -> RoochResult<Vec<BTreeMap<Identifier, AnnotatedMoveValueView>>> {
        let context = self.context_options.build()?;
        let mapping = context.address_mapping();
        let address_addr = self.address.into_account_address(&mapping)?;

        let function_id = "0x3::session_key::get_session_keys_handle".parse::<FunctionId>()?;
        let function_call = FunctionCall::new(function_id, vec![], vec![address_addr.to_vec()]);
        let client = context.get_client().await?;
        let view_result = client
            .rooch
            .execute_view_function(function_call)
            .await
            .map_err(|e| RoochError::ViewFunctionError(e.to_string()))?;
        let obj_id = extract_obj_id(view_result)
            .ok_or(RoochError::from(anyhow::anyhow!("Session key not found")))?;

        let options = StateOptions::new().decode(true);
        let field_result = client
            .rooch
            .list_field_states(obj_id.into(), None, None, Some(options))
            .await
            .map_err(RoochError::from)?;

        Ok(extract_session_keys(field_result))
    }
}

fn extract_obj_id(view_result: AnnotatedFunctionResultView) -> Option<ObjectID> {
    if let Some(return_value) = view_result.return_values {
        if let AnnotatedMoveValueView::Struct(struct_value) =
            return_value.first()?.decoded_value.clone()
        {
            if let Some(AnnotatedMoveValueView::Vector(vec)) =
                struct_value.value.first_key_value().map(|(_, value)| value)
            {
                if let AnnotatedMoveValueView::Address(addr) = vec.first()? {
                    addr.to_string().parse::<ObjectID>().ok()
                } else {
                    None
                }
            } else {
                None
            }
        } else {
            None
        }
    } else {
        None
    }
}

fn extract_session_keys(
    field_result: StatePageView,
) -> Vec<BTreeMap<Identifier, AnnotatedMoveValueView>> {
    let mut value = vec![];
    for data in field_result.data {
        if let Some(decoded_value) = data.state.decoded_value {
            value.push(decoded_value.value)
        }
    }
    value
}
