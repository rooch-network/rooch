// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::cli_types::{CommandAction, TransactionOptions, WalletContextOptions};
use anyhow::Result;
use async_trait::async_trait;
use clap::Parser;
use move_command_line_common::types::ParsedStructType;
use move_core_types::language_storage::TypeTag;
use moveos_types::transaction::MoveAction;
use rooch_key::keystore::account_keystore::AccountKeystore;
use rooch_types::{
    error::RoochResult,
    framework::auth_payload::{AuthPayload, SignData},
    function_arg::{parse_function_arg, FunctionArg, ParsedFunctionId},
};

/// Sign an msg with current account private key (sign_hashed)
///
/// This operation must be specified with -a or
/// --address to export only one address with a private key.
#[derive(Debug, Parser)]
pub struct SignCommand {
    /// Function name as `<ADDRESS>::<MODULE_ID>::<FUNCTION_NAME>`
    /// Example: `0x42::message::set_message`, `rooch_framework::empty::empty`
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
    #[clap(long = "args", value_parser=parse_function_arg)]
    pub args: Vec<FunctionArg>,

    #[clap(flatten)]
    pub tx_options: TransactionOptions,

    #[clap(flatten)]
    pub context_options: WalletContextOptions,

    /// Return command outputs in json format
    #[clap(long, default_value = "false")]
    json: bool,
}

#[async_trait]
impl CommandAction<Option<AuthPayload>> for SignCommand {
    async fn execute(self) -> RoochResult<Option<AuthPayload>> {
        let context = self.context_options.build_require_password()?;
        let password = context.get_password();
        let address_mapping = context.address_mapping();
        let sender = context.resolve_address(self.tx_options.sender)?.into();
        let kp = context.keystore.get_key_pair(&sender, password)?;
        let bitcoin_address = kp.public().bitcoin_address()?;
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
        let sign_data = SignData::new_with_default(&tx_data);
        let data_hash = sign_data.data_hash();
        let signature = kp.sign(data_hash.as_bytes());
        let auth_payload = AuthPayload::new(sign_data, signature, bitcoin_address.to_string());

        if self.json {
            Ok(Some(auth_payload))
        } else {
            println!("Sign succeeded with the auth payload: {:?}", auth_payload);
            Ok(None)
        }
    }
}
