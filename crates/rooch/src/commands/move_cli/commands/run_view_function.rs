// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::cli_types::{CommandAction, WalletContextOptions};
use anyhow::Result;
use async_trait::async_trait;
use clap::Parser;
use move_command_line_common::types::ParsedStructType;
use move_core_types::language_storage::TypeTag;
use moveos_types::transaction::FunctionCall;
use rooch_rpc_api::jsonrpc_types::AnnotatedFunctionResultView;
use rooch_types::{
    error::{RoochError, RoochResult},
    function_arg::{FunctionArg, ParsedFunctionId},
};

/// Run a Move function
#[derive(Parser)]
pub struct RunViewFunction {
    /// Function name as `<ADDRESS>::<MODULE_ID>::<FUNCTION_NAME>`
    /// Example: `0x42::message::get_message`
    #[clap(long)]
    pub function: ParsedFunctionId,

    /// TypeTag arguments separated by spaces.
    ///
    /// Example: `0x1::M::T1 0x1::M::T2 rooch_framework::empty::Empty`
    #[clap(
        long = "type-args",
        value_parser=ParsedStructType::parse,
    )]
    pub type_args: Vec<ParsedStructType>,

    /// Arguments combined with their type separated by spaces.
    ///
    /// Supported types [u8, u16, u32, u64, u128, u256, bool, object_id, string, address, vector<inner_type>]
    ///
    /// Example: `address:0x1 bool:true u8:0 u256:1234 'vector<u32>:a,b,c,d'`
    ///     address and uint can be written in short form like `@0x1 1u8 4123u256`.
    #[clap(
        long = "args",
    )]
    pub args: Vec<FunctionArg>,

    /// RPC client options.
    #[clap(flatten)]
    context: WalletContextOptions,
}

#[async_trait]
impl CommandAction<AnnotatedFunctionResultView> for RunViewFunction {
    async fn execute(self) -> RoochResult<AnnotatedFunctionResultView> {
        let context = self.context.build()?;
        let address_mapping = context.address_mapping();
        let function_id = self.function.into_function_id(&address_mapping)?;
        let args = self
            .args
            .into_iter()
            .map(|arg| arg.into_bytes(&address_mapping))
            .collect::<Result<Vec<_>>>()?;
        let type_args = self
            .type_args
            .into_iter()
            .map(|tag| {
                Ok(TypeTag::Struct(Box::new(
                    tag.into_struct_tag(&address_mapping)?,
                )))
            })
            .collect::<Result<Vec<_>>>()?;

        let function_call = FunctionCall::new(function_id, type_args, args);

        let client = context.get_client().await?;
        client
            .rooch
            .execute_view_function(function_call)
            .await
            .map_err(|e| RoochError::ViewFunctionError(e.to_string()))
    }
}
