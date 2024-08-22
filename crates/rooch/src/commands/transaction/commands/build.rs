// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use std::{fs::File, io::Write};

use crate::cli_types::{CommandAction, TransactionOptions, WalletContextOptions};
use anyhow::Result;
use async_trait::async_trait;
use move_command_line_common::types::ParsedStructType;
use move_core_types::language_storage::TypeTag;
use moveos_types::transaction::MoveAction;
use rooch_types::{
    error::RoochResult,
    function_arg::{parse_function_arg, FunctionArg, ParsedFunctionId},
};

/// Get transactions by order
#[derive(Debug, clap::Parser)]
pub struct BuildCommand {
    /// Function name as `<ADDRESS>::<MODULE_ID>::<FUNCTION_NAME>`
    /// Example: `0x42::message::set_message`, `rooch_framework::empty::empty`
    #[clap(long, required = true)]
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
    #[clap(long = "args", value_parser=parse_function_arg)]
    pub args: Vec<FunctionArg>,

    #[clap(flatten)]
    tx_options: TransactionOptions,

    #[clap(flatten)]
    context: WalletContextOptions,

    /// File destination for the file being written
    #[clap(long)]
    file_destination: Option<String>,

    /// Return command outputs in json format
    #[clap(long, default_value = "false")]
    json: bool,
}

#[async_trait]
impl CommandAction<Option<String>> for BuildCommand {
    async fn execute(self) -> RoochResult<Option<String>> {
        let context = self.context.build()?;
        let address_mapping = context.address_mapping();
        let sender = context.resolve_address(self.tx_options.sender)?.into();
        let max_gas_amount = self.tx_options.max_gas_amount;

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
        let action = MoveAction::new_function_call(function_id, type_args, args);

        let tx_data = context
            .build_tx_data(sender, action, max_gas_amount)
            .await?;

        if let Some(file_destination) = self.file_destination {
            let mut file = File::create(file_destination)?;
            file.write_all(&tx_data.encode())?;
            println!("Write transaction hex succeeded in the destination");

            Ok(None)
        } else {
            let tx_data_hex = hex::encode(tx_data.encode());
            if self.json {
                Ok(Some(tx_data_hex))
            } else {
                println!(
                    "Build transaction succeeded with the transaction hex [{}]",
                    tx_data_hex
                );

                Ok(None)
            }
        }
    }
}
