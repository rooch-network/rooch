// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::move_cli::types::TransactionOptions;
use anyhow::{anyhow, Result};
use async_trait::async_trait;
use clap::Parser;
use move_core_types::{
    account_address::AccountAddress,
    identifier::Identifier,
    language_storage::{ModuleId, TypeTag},
    parser::{parse_transaction_argument, parse_type_tag},
    transaction_argument::TransactionArgument,
    value::MoveValue,
};
use moveos::moveos::TransactionOutput;
use moveos_types::transaction::MoveAction;
use rooch_client::Client;
use rooch_server::response::JsonResponse;
use rooch_types::{
    address::RoochAddress,
    cli::{CliError, CliResult, CommandAction},
    transaction::{authenticator::AccountPrivateKey, rooch::RoochTransactionData},
};
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
pub struct RunFunction {
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

    #[clap(flatten)]
    txn_options: TransactionOptions,
}

#[async_trait]
impl CommandAction<JsonResponse<TransactionOutput>> for RunFunction {
    async fn execute(self) -> CliResult<JsonResponse<TransactionOutput>> {
        let args = self
            .args
            .iter()
            .map(|arg| {
                MoveValue::from(arg.clone())
                    .simple_serialize()
                    .expect("transaction arguments must serialize")
            })
            .collect();

        if self.txn_options.sender_account.is_none() {
            return Err(CliError::CommandArgumentError(
                "--sender-account required".to_string(),
            ));
        }

        let sender: RoochAddress = self.txn_options.sender_account.unwrap().into();
        let sequence_number = self
            .client
            .get_sequence_number(sender)
            .await
            .map_err(CliError::from)?;
        let tx_data = RoochTransactionData::new(
            sender,
            sequence_number,
            MoveAction::new_function(
                self.function.module_id.clone(),
                self.function.function_name.clone(),
                self.type_args,
                args,
            ),
        );
        //TODO sign the tx by the account private key
        let private_key = AccountPrivateKey::generate_for_testing();
        let tx = tx_data
            .sign(&private_key)
            .map_err(|e| CliError::SignMessageError(e.to_string()))?;
        self.client
            .execute_tx(tx)
            .await
            .map_err(|e| CliError::TransactionError(e.to_string()))
    }
}
