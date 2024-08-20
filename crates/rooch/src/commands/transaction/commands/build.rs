// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::cli_types::{CommandAction, TransactionOptions, WalletContextOptions};
use async_trait::async_trait;
use move_core_types::{
    account_address::AccountAddress,
    identifier::Identifier,
    language_storage::{ModuleId, TypeTag},
};
use moveos_types::{move_types::FunctionId, transaction::MoveAction};
use rooch_types::error::RoochResult;

/// Get transactions by order
#[derive(Debug, clap::Parser)]
pub struct BuildCommand {
    #[clap(long, required = true)]
    pub module_address: AccountAddress,

    #[clap(long, required = true)]
    pub module_name: String,

    #[clap(long, required = true)]
    pub function_name: String,

    #[clap(long)]
    pub type_args: Vec<TypeTag>,

    #[clap(long)]
    pub args: Vec<Vec<u8>>,

    #[clap(flatten)]
    tx_options: TransactionOptions,

    #[clap(flatten)]
    context: WalletContextOptions,

    /// Return command outputs in json format
    #[clap(long, default_value = "false")]
    json: bool,
}

#[async_trait]
impl CommandAction<Option<String>> for BuildCommand {
    async fn execute(self) -> RoochResult<Option<String>> {
        let context = self.context.build()?;
        let sender = context.resolve_address(self.tx_options.sender)?.into();
        let max_gas_amount = self.tx_options.max_gas_amount;

        let action = MoveAction::new_function_call(
            FunctionId::new(
                ModuleId::new(
                    self.module_address,
                    Identifier::new(self.module_name.to_owned()).unwrap(),
                ),
                Identifier::new(self.function_name.to_owned()).unwrap(),
            ),
            self.type_args,
            self.args,
        );

        let tx_data = context
            .build_tx_data(sender, action, max_gas_amount)
            .await?;
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
