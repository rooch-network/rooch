// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use anyhow::{bail, Result};
use async_trait::async_trait;
use clap::Parser;
use move_command_line_common::types::ParsedStructType;
use move_core_types::language_storage::TypeTag;
use moveos_types::transaction::FunctionCall;
use rooch_rpc_api::jsonrpc_types::VMStatusView;
use rooch_rpc_client::{wallet_context::WalletContext, Client};
use rooch_types::function_arg::{parse_function_arg, FunctionArg, ParsedFunctionId};
use tracing::info;

#[async_trait]
pub trait Checker {
    async fn check(&self) -> Result<bool>;
}

#[derive(Debug, Clone, Parser)]
pub struct ViewFunctionCheckerArgs {
    #[clap(name = "checker-function", long = "checker-function", required = true)]
    pub function: ParsedFunctionId,

    /// TypeTag arguments separated by spaces.
    ///
    /// Example: `0x1::M::T1 0x1::M::T2 rooch_framework::empty::Empty`
    #[clap(
        name = "checker-type-args",
        long = "checker-type-args",
        value_parser=ParsedStructType::parse,
    )]
    pub type_args: Vec<ParsedStructType>,

    /// Arguments combined with their type separated by spaces.
    ///
    /// Supported types [u8, u16, u32, u64, u128, u256, bool, object_id, string, address, vector<inner_type>]
    ///
    /// Example: `address:0x1 bool:true u8:0 u256:1234 'vector<u32>:a,b,c,d'`
    ///     address and uint can be written in short form like `@0x1 1u8 4123u256`.
    #[clap(name="checker-args", long = "checker-args", value_parser=parse_function_arg)]
    pub args: Vec<FunctionArg>,
}

impl ViewFunctionCheckerArgs {
    pub async fn build(
        self: ViewFunctionCheckerArgs,
        context: &WalletContext,
    ) -> Result<ViewFunctionChecker> {
        let client = context.get_client().await?;
        let address_mapping = context.address_mapping();

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

        let function_call = FunctionCall {
            function_id: self.function.into_function_id(&address_mapping)?,
            ty_args: type_args,
            args,
        };
        Ok(ViewFunctionChecker {
            client,
            function_call,
        })
    }
}

pub struct ViewFunctionChecker {
    client: Client,
    function_call: FunctionCall,
}

impl ViewFunctionChecker {
    pub async fn call_view_function(&self) -> Result<bool> {
        self.client
            .rooch
            .execute_view_function(self.function_call.clone())
            .await
            .and_then(|fn_result| {
                if fn_result.vm_status != VMStatusView::Executed {
                    bail!("Function failed with status: {:?}", fn_result.vm_status);
                } else {
                    match fn_result.return_values {
                        Some(values) => {
                            if values.len() != 1 {
                                bail!(
                                    "The function {} unexpected return values: {:?}",
                                    self.function_call.function_id,
                                    values
                                );
                            }
                            let value_view = values.first().unwrap();
                            match value_view.value.type_tag.0 {
                                TypeTag::Bool => {
                                    let bool_value: bool = bcs::from_bytes(
                                        &value_view.value.value.0,
                                    )
                                    .map_err(|e| {
                                        anyhow::anyhow!(
                                            "The function {} deserialize bool value error: {}",
                                            self.function_call.function_id,
                                            e
                                        )
                                    })?;
                                    Ok(bool_value)
                                }
                                //TODO should we support other types return value as bool?
                                _ => bail!(
                                    "The function {} return value is not a bool",
                                    self.function_call.function_id
                                ),
                            }
                        }
                        None => {
                            bail!(
                                "The function {} no return value",
                                self.function_call.function_id
                            );
                        }
                    }
                }
            })
    }
}

#[async_trait]
impl Checker for ViewFunctionChecker {
    async fn check(&self) -> Result<bool> {
        let result = self.call_view_function().await;
        info!(
            "Check function {} result: {:?}",
            self.function_call.function_id, result
        );
        result
    }
}
