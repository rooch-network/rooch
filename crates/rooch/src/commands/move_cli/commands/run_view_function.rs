// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use anyhow::{anyhow, Result};
use clap::Parser;
use move_core_types::{
    account_address::AccountAddress,
    identifier::Identifier,
    language_storage::{ModuleId, TypeTag},
    parser::{parse_transaction_argument, parse_type_tag},
    transaction_argument::TransactionArgument,
    value::MoveValue,
};
use moveos_types::transaction::{Function, ViewPayload};
use rooch_client::Client;
use std::str::FromStr;

/// Identifier of a module function
#[derive(Debug, Clone)]
pub struct FunctionId {
    pub module_id: ModuleId,
    pub function_name: Identifier,
}

fn parse_function_id(function_id: &str) -> Result<FunctionId> {
    let ids: Vec<&str> = function_id.split_terminator("::").collect();
    if ids.len() != 3 {
        return Err(anyhow!(
            "FunctionId is not well formed.  Must be of the form <address>::<module>::<function>"
        ));
    }
    let address = AccountAddress::from_str(ids.first().unwrap())
        .map_err(|err| anyhow!("Module address error: {:?}", err.to_string()))?;
    let module = Identifier::from_str(ids.get(1).unwrap())
        .map_err(|err| anyhow!("Module name error: {:?}", err.to_string()))?;
    let function_name = Identifier::from_str(ids.get(2).unwrap())
        .map_err(|err| anyhow!("Function name error: {:?}", err.to_string()))?;
    let module_id = ModuleId::new(address, module);
    Ok(FunctionId {
        module_id,
        function_name,
    })
}

impl FromStr for FunctionId {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        parse_function_id(s)
    }
}

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

impl RunViewFunction {
    pub async fn execute(self) -> anyhow::Result<()> {
        let args = self
            .args
            .iter()
            .map(|arg| {
                MoveValue::from(arg.clone())
                    .simple_serialize()
                    .expect("transaction arguments must serialize")
            })
            .collect();
        let payload = ViewPayload {
            function: Function {
                module: self.function.module_id.clone(),
                function: self.function.function_name.clone(),
                ty_args: self.type_args,
                args,
            },
        };
        self.client.view(payload).await?;
        Ok(())
    }
}
