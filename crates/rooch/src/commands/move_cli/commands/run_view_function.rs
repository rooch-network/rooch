// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use async_trait::async_trait;
use clap::Parser;
use move_core_types::{
    language_storage::TypeTag,
    parser::{parse_transaction_argument, parse_type_tag},
    transaction_argument::TransactionArgument,
    value::MoveValue,
};
use moveos_types::{move_types::FunctionId, transaction::FunctionCall};
use rooch_client::Client;
use rooch_types::cli::{CliError, CliResult, CommandAction};

/// Run a Move function
#[derive(Parser)]
pub struct RunViewFunction {
    /// Function name as `<ADDRESS>::<MODULE_ID>::<FUNCTION_NAME>`
    /// Example: `0x842ed41fad9640a2ad08fdd7d3e4f7f505319aac7d67e1c0dd6a7cce8732c7e3::message::set_message`
    #[clap(long)]
    pub function: FunctionId,

    /// TypeTag arguments separated by spaces.
    ///
    /// Example: `u8 u16 u32 u64 u128 u256 bool address`
    #[clap(
        long = "type-args",
        parse(try_from_str = parse_type_tag),
        takes_value(true),
        multiple_values(true),
        multiple_occurrences(true)
    )]
    pub type_args: Vec<TypeTag>,

    /// Arguments combined with their type separated by spaces.
    ///
    /// Supported types [u8, u16, u32, u64, u128, u256, bool, hex, address, raw]
    ///
    /// Example: `0x1 true 0 1234 "hello"`
    #[clap(
        long = "args",
        parse(try_from_str = parse_transaction_argument),
        takes_value(true),
        multiple_values(true),
        multiple_occurrences(true)
    )]
    pub args: Vec<TransactionArgument>,

    /// RPC client options.
    #[clap(flatten)]
    client: Client,
}

#[async_trait]
impl CommandAction<Vec<serde_json::Value>> for RunViewFunction {
    async fn execute(self) -> CliResult<Vec<serde_json::Value>> {
        let args = self
            .args
            .iter()
            .map(|arg| {
                MoveValue::from(arg.clone())
                    .simple_serialize()
                    .expect("transaction arguments must be serializabe")
            })
            .collect();
        let function_call = FunctionCall {
            function_id: self.function,
            ty_args: self.type_args,
            args,
        };
        self.client
            .execute_view_function(function_call)
            .await
            .map_err(|e| CliError::ViewFunctionError(e.to_string()))
    }
}
