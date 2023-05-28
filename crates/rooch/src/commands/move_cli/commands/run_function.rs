// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::move_cli::types::TransactionOptions;
use async_trait::async_trait;
use clap::Parser;
use move_core_types::{
    language_storage::TypeTag,
    parser::{parse_transaction_argument, parse_type_tag},
    transaction_argument::TransactionArgument,
    value::MoveValue,
};
use moveos::moveos::TransactionOutput;
use moveos_types::{move_types::FunctionId, transaction::MoveAction};
use rooch_client::Client;
use rooch_types::{
    address::RoochAddress,
    cli::{CliError, CliResult, CommandAction},
    transaction::rooch::RoochTransactionData,
};

use rooch_common::config::{
    rooch_config_dir, rooch_config_path, Config, PersistedConfig, RoochConfig, ROOCH_CONFIG,
};
use rooch_key::keystore::AccountKeystore;

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
impl CommandAction<TransactionOutput> for RunFunction {
    async fn execute(self) -> CliResult<TransactionOutput> {
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
            MoveAction::new_function_call(self.function, self.type_args, args),
        );
        //TODO sign the tx by the account private key

        // TODO: Code refactoring
        let config: RoochConfig = PersistedConfig::read(rooch_config_path()?.as_path())?;
        let config: PersistedConfig<RoochConfig> = config.persisted(
            rooch_config_dir()
                .map_err(CliError::from)?
                .join(ROOCH_CONFIG)
                .as_path(),
        );

        let tx = config.keystore.sign_transaction(&sender, tx_data).unwrap();

        self.client
            .execute_tx(tx)
            .await
            .map_err(|e| CliError::TransactionError(e.to_string()))
    }
}
