// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::cli_types::{CommandAction, FunctionArg, TransactionOptions, WalletContextOptions};
use crate::tx_runner::{dry_run_tx_locally, execute_tx_locally_with_gas_profile};
use anyhow::Result;
use async_trait::async_trait;
use clap::Parser;
use move_command_line_common::types::ParsedStructType;
use move_core_types::language_storage::TypeTag;
use moveos_types::transaction::MoveAction;
use rooch_key::key_derive::verify_password;
use rooch_key::keystore::account_keystore::AccountKeystore;
use rooch_rpc_api::jsonrpc_types::{
    ExecuteTransactionResponseView, HumanReadableDisplay, KeptVMStatusView,
};
use rooch_types::function_arg::parse_function_arg;
use rooch_types::{
    address::RoochAddress,
    error::{RoochError, RoochResult},
    function_arg::ParsedFunctionId,
    transaction::rooch::RoochTransaction,
};
use rpassword::prompt_password;

/// Run a Move function
#[derive(Parser)]
pub struct RunFunction {
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

    /// RPC client options.
    #[clap(flatten)]
    context: WalletContextOptions,

    #[clap(flatten)]
    tx_options: TransactionOptions,

    /// Return command outputs in json format
    #[clap(long, default_value = "false")]
    json: bool,

    /// Run the gas profiler and output html report
    #[clap(long, default_value = "false")]
    gas_profile: bool,

    /// Run the DryRun for this transaction
    #[clap(long, default_value = "false")]
    dry_run: bool,
}

#[async_trait]
impl CommandAction<ExecuteTransactionResponseView> for RunFunction {
    async fn execute(self) -> RoochResult<ExecuteTransactionResponseView> {
        let context = self.context.build()?;
        let address_mapping = context.address_mapping();
        let sender: RoochAddress = context.resolve_address(self.tx_options.sender)?.into();
        let max_gas_amount: Option<u64> = self.tx_options.max_gas_amount;
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

        if self.dry_run {
            let rooch_tx_data = context
                .build_tx_data(sender, action.clone(), max_gas_amount)
                .await?;
            let dry_run_result_opt =
                dry_run_tx_locally(context.get_client().await?, rooch_tx_data)?;

            if let Some(dry_run_result) = dry_run_result_opt {
                if dry_run_result.raw_output.status != KeptVMStatusView::Executed {
                    return Ok(dry_run_result.into());
                };
            }
        }

        let result = match (self.tx_options.authenticator, self.tx_options.session_key) {
            (Some(authenticator), _) => {
                let tx_data = context
                    .build_tx_data(sender, action, max_gas_amount)
                    .await?;
                //TODO the authenticator usually is associated with the RoochTransactinData
                //So we need to find a way to let user generate the authenticator based on the tx_data.
                let tx = RoochTransaction::new(tx_data, authenticator.into());
                context.execute(tx).await?
            }
            (_, Some(session_key)) => {
                let tx_data = context
                    .build_tx_data(sender, action, max_gas_amount)
                    .await?;
                let tx = if context.keystore.get_if_password_is_empty() {
                    context
                        .keystore
                        .sign_transaction_via_session_key(&sender, tx_data, &session_key, None)
                        .map_err(|e| RoochError::SignMessageError(e.to_string()))?
                } else {
                    let password =
                        prompt_password("Enter the password to run functions:").unwrap_or_default();
                    let is_verified = verify_password(
                        Some(password.clone()),
                        context.keystore.get_password_hash(),
                    )?;

                    if !is_verified {
                        return Err(RoochError::InvalidPasswordError(
                            "Password is invalid".to_owned(),
                        ));
                    }

                    context
                        .keystore
                        .sign_transaction_via_session_key(
                            &sender,
                            tx_data,
                            &session_key,
                            Some(password),
                        )
                        .map_err(|e| RoochError::SignMessageError(e.to_string()))?
                };
                context.execute(tx).await?
            }
            (None, None) => {
                if context.keystore.get_if_password_is_empty() {
                    context
                        .sign_and_execute(sender, action, None, max_gas_amount)
                        .await?
                } else {
                    let password =
                        prompt_password("Enter the password to run functions:").unwrap_or_default();
                    let is_verified = verify_password(
                        Some(password.clone()),
                        context.keystore.get_password_hash(),
                    )?;

                    if !is_verified {
                        return Err(RoochError::InvalidPasswordError(
                            "Password is invalid".to_owned(),
                        ));
                    }

                    let tx_execution_result = context
                        .sign_and_execute(
                            sender,
                            action.clone(),
                            Some(password.clone()),
                            max_gas_amount,
                        )
                        .await?;

                    if self.gas_profile {
                        let state_root = tx_execution_result
                            .execution_info
                            .state_root
                            .0
                            .as_bytes()
                            .to_vec();
                        let tx = context
                            .sign(sender, action, Some(password), max_gas_amount)
                            .await?;
                        execute_tx_locally_with_gas_profile(
                            state_root,
                            context.get_client().await?,
                            tx.data,
                        )?;
                    }

                    tx_execution_result
                }
            }
        };

        Ok(result)
    }

    /// Executes the command, and serializes it to the common JSON output type
    async fn execute_serialized(self) -> RoochResult<String> {
        let json = self.json;
        let result = self.execute().await?;

        if json {
            let output = serde_json::to_string_pretty(&result).unwrap();
            if output == "null" {
                return Ok("".to_string());
            }
            Ok(output)
        } else {
            let mut output = String::new();
            // print execution info
            let exe_info = &result.execution_info;
            output.push_str(&exe_info.to_human_readable_string(false, 0));

            if let Some(txn_output) = &result.output {
                // print error info
                if let Some(error_info) = result.clone().error_info {
                    output.push_str(
                        format!(
                            "\n\n\nTransaction dry run failed:\n {:?}",
                            error_info.vm_error_info.error_message
                        )
                        .as_str(),
                    );
                    output.push_str("\nCallStack trace:\n".to_string().as_str());
                    for (idx, item) in error_info.vm_error_info.execution_state.iter().enumerate() {
                        output.push_str(format!("{} {}\n", idx, item).as_str());
                    }
                };

                // print objects changes
                output.push_str("\n\n");
                output.push_str(&txn_output.changeset.to_human_readable_string(false, 0));

                // print events
                output.push_str("\n\n");
                output.push_str("Events:\n");
                output.push_str(&txn_output.events.to_human_readable_string(false, 4));
            };

            Ok(output)
        }
    }
}
