// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use anyhow::{bail, Result};
use async_trait::async_trait;
use clap::Parser;
use move_command_line_common::types::ParsedStructType;
use move_core_types::language_storage::TypeTag;
use moveos_types::transaction::{FunctionCall, MoveAction};
use rooch_rpc_api::jsonrpc_types::{ExecuteTransactionResponseView, KeptVMStatusView};
use rooch_rpc_client::wallet_context::WalletContext;
use rooch_types::{
    address::RoochAddress,
    function_arg::{parse_function_arg, FunctionArg, ParsedFunctionId},
};
use tracing::{info, warn};

#[async_trait]
pub trait Runner {
    async fn run(&self) -> Result<()>;
}

#[derive(Debug, Clone, Parser)]
pub struct FunctionRunnerArgs {
    #[clap(name = "runner-function", long = "runner-function", required = true)]
    pub function: ParsedFunctionId,

    /// TypeTag arguments separated by spaces.
    ///
    /// Example: `0x1::M::T1 0x1::M::T2 rooch_framework::empty::Empty`
    #[clap(
        name = "runner-type-args",
        long = "runner-type-args",
        value_parser=ParsedStructType::parse,
    )]
    pub type_args: Vec<ParsedStructType>,

    /// Arguments combined with their type separated by spaces.
    ///
    /// Supported types [u8, u16, u32, u64, u128, u256, bool, object_id, string, address, vector<inner_type>]
    ///
    /// Example: `address:0x1 bool:true u8:0 u256:1234 'vector<u32>:a,b,c,d'`
    ///     address and uint can be written in short form like `@0x1 1u8 4123u256`.
    #[clap(name = "runner-args", long = "runner-args", value_parser=parse_function_arg)]
    pub args: Vec<FunctionArg>,
}

impl FunctionRunnerArgs {
    pub async fn build(
        self: FunctionRunnerArgs,
        sender: RoochAddress,
        max_gas_amount: Option<u64>,
        context: &WalletContext,
    ) -> Result<FunctionRunner> {
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
        Ok(FunctionRunner {
            context,
            function_call,
            sender,
            max_gas_amount,
        })
    }
}

pub struct FunctionRunner<'a> {
    context: &'a WalletContext,
    function_call: FunctionCall,
    sender: RoochAddress,
    max_gas_amount: Option<u64>,
}

impl<'a> FunctionRunner<'a> {
    pub async fn run_function(&self) -> Result<ExecuteTransactionResponseView> {
        let tx_data = self
            .context
            .build_tx_data(
                self.sender,
                MoveAction::Function(self.function_call.clone()),
                self.max_gas_amount,
            )
            .await?;
        let response = self.context.sign_and_execute(self.sender, tx_data).await?;
        if response.execution_info.status != KeptVMStatusView::Executed {
            bail!("{:?}, ", response.execution_info.status);
        }
        Ok(response)
    }
}

#[async_trait]
impl<'a> Runner for FunctionRunner<'a> {
    async fn run(&self) -> Result<()> {
        let result = self.run_function().await;
        match result {
            Ok(response) => {
                info!(
                    "Function {} executed, tx order: {}, gas_used: {}",
                    self.function_call.function_id,
                    response.sequence_info.tx_order,
                    response.execution_info.gas_used
                );
                Ok(())
            }
            Err(e) => {
                warn!(
                    "Function {} failed to execute: {:?}",
                    self.function_call.function_id, e
                );
                Err(e)
            }
        }
    }
}
