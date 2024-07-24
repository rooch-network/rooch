// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::cli_types::{CommandAction, TransactionOptions, WalletContextOptions};
use async_trait::async_trait;
use clap::Parser;
use move_cli::Move;
use move_core_types::effects::Op;
use move_core_types::{identifier::Identifier, language_storage::ModuleId};
use moveos_compiler::dependency_order::sort_by_dependency_order;
use moveos_types::move_std::string::MoveString;
use moveos_types::moveos_std::module_store::ModuleStore;
use moveos_types::moveos_std::move_module::MoveModule;
use moveos_types::moveos_std::object::ObjectMeta;
use moveos_types::{
    addresses::MOVEOS_STD_ADDRESS, move_types::FunctionId, state::ObjectState,
    transaction::MoveAction,
};
use moveos_verifier::build::run_verifier;
use moveos_verifier::verifier;
use rooch_key::key_derive::verify_password;
use rooch_key::keystore::account_keystore::AccountKeystore;
use rooch_rpc_api::jsonrpc_types::ExecuteTransactionResponseView;
use rooch_types::address::RoochAddress;
use rooch_types::error::{RoochError, RoochResult};
use rooch_types::transaction::rooch::RoochTransaction;
use rpassword::prompt_password;
use std::collections::BTreeMap;
use std::io::stderr;

#[derive(Parser)]
pub struct Publish {
    #[clap(flatten)]
    context_options: WalletContextOptions,

    #[clap(flatten)]
    move_args: Move,

    #[clap(flatten)]
    tx_options: TransactionOptions,

    /// Named addresses for the move binary
    ///
    /// Example: alice=0x1234, bob=default, alice2=alice
    ///
    /// Note: This will fail if there are duplicates in the Move.toml file remove those first.
    #[clap(long, value_parser=crate::utils::parse_map::<String, String>, default_value = "")]
    pub(crate) named_addresses: BTreeMap<String, String>,

    /// Whether publish modules by `MoveAction::ModuleBundle`?
    /// If not set, publish moduels through Move entry function
    /// `moveos_std::module_store::publish_modules_entry`.
    /// **Deprecated**! Publish modules by `MoveAction::ModuleBundle` is no longer used anymore.
    /// So you should never add this option.
    /// For now, the option is kept for test only.
    #[clap(long)]
    pub by_move_action: bool,

    /// Return command outputs in json format
    #[clap(long, default_value = "false")]
    json: bool,
}

#[async_trait]
impl CommandAction<ExecuteTransactionResponseView> for Publish {
    async fn execute(self) -> RoochResult<ExecuteTransactionResponseView> {
        // Build context and handle errors
        let context = self.context_options.build()?;

        // Clone variables for later use
        let package_path = self
            .move_args
            .package_path
            .unwrap_or_else(|| std::env::current_dir().unwrap());
        let config = self.move_args.build_config.clone();
        let mut config = config.clone();

        // Parse named addresses from context and update config
        config.additional_named_addresses =
            context.parse_and_resolve_addresses(self.named_addresses)?;
        let config_cloned = config.clone();

        // Compile the package and run the verifier
        let mut package = config.compile_package_no_exit(&package_path, &mut stderr())?;
        run_verifier(package_path, config_cloned, &mut package)?;

        // Get the modules from the package
        let modules = package.root_modules_map();
        let empty_modules = modules.iter_modules_owned().is_empty();
        let pkg_address = if !empty_modules {
            let first_module = &modules.iter_modules_owned()[0];
            first_module.self_id().address().to_owned()
        } else {
            return Err(RoochError::MoveCompilationError(format!(
                "compiling move modules error! Is the project or module empty: {:?}",
                empty_modules,
            )));
        };

        // Initialize bundles vector and sort modules by dependency order
        let mut bundles: Vec<Vec<u8>> = vec![];
        let sorted_modules = sort_by_dependency_order(modules.iter_modules())?;
        let resolver = context.get_client().await?;
        // Serialize and collect module binaries into bundles
        verifier::verify_modules(&sorted_modules, &resolver)?;
        for module in sorted_modules {
            let module_address = module.self_id().address().to_owned();
            if module_address != pkg_address {
                return Err(RoochError::MoveCompilationError(format!(
                    "module's address ({:?}) not same as package module address {:?}",
                    module_address,
                    pkg_address.clone(),
                )));
            };
            let mut binary: Vec<u8> = vec![];
            module.serialize(&mut binary)?;
            bundles.push(binary);
        }

        // Validate sender account if provided
        if pkg_address != context.resolve_address(self.tx_options.sender)? {
            return Err(RoochError::CommandArgumentError(
                "--sender-account required and the sender account must be the same as the package address"
                    .to_string(),
            ));
        }

        // Create a sender RoochAddress
        let sender: RoochAddress = pkg_address.into();
        eprintln!("Publish modules to address: {:?}", sender);

        let max_gas_amount: Option<u64> = self.tx_options.max_gas_amount;

        // Prepare and execute the transaction based on the action type
        let tx_result = if !self.by_move_action {
            let args = bcs::to_bytes(&bundles).unwrap();
            let action = MoveAction::new_function_call(
                FunctionId::new(
                    ModuleId::new(
                        MOVEOS_STD_ADDRESS,
                        Identifier::new("module_store".to_owned()).unwrap(),
                    ),
                    Identifier::new("publish_modules_entry".to_owned()).unwrap(),
                ),
                vec![],
                vec![args],
            );

            // Handle transaction with or without authenticator
            match self.tx_options.authenticator {
                Some(authenticator) => {
                    let tx_data = context
                        .build_tx_data(sender, action, max_gas_amount)
                        .await?;
                    let tx = RoochTransaction::new(tx_data, authenticator.into());
                    context.execute(tx).await?
                }
                None => {
                    if context.keystore.get_if_password_is_empty() {
                        context
                            .sign_and_execute(sender, action, None, max_gas_amount)
                            .await?
                    } else {
                        let password =
                            prompt_password("Enter the password to publish:").unwrap_or_default();
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
                            .sign_and_execute(sender, action, Some(password), max_gas_amount)
                            .await?
                    }
                }
            }
        } else {
            // Handle MoveAction.ModuleBundle case
            let action = MoveAction::ModuleBundle(bundles);

            if context.keystore.get_if_password_is_empty() {
                context
                    .sign_and_execute(sender, action, None, max_gas_amount)
                    .await?
            } else {
                let password =
                    prompt_password("Enter the password to publish:").unwrap_or_default();
                let is_verified =
                    verify_password(Some(password.clone()), context.keystore.get_password_hash())?;

                if !is_verified {
                    return Err(RoochError::InvalidPasswordError(
                        "Password is invalid".to_owned(),
                    ));
                }

                context
                    .sign_and_execute(sender, action, Some(password), max_gas_amount)
                    .await?
            }
        };
        //Directly return the result, the publish transaction may be failed.
        //Caller need to check the `execution_info.status` field.
        Ok(tx_result)
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
            Self::pretty_transaction_response(&result)
        }
    }
}

impl Publish {
    fn pretty_transaction_response(
        txn_response: &ExecuteTransactionResponseView,
    ) -> RoochResult<String> {
        let mut output = String::new();

        // print execution info
        let exe_info = &txn_response.execution_info;
        output.push_str(&format!(
            r#"Execution info:
    status: {:?}
    gas used: {}
    tx hash: {}
    state root: {}
    event root: {}"#,
            exe_info.status,
            exe_info.gas_used,
            exe_info.tx_hash,
            exe_info.state_root,
            exe_info.event_root
        ));

        if let Some(txn_output) = &txn_response.output {
            // print modules
            let changes = &txn_output.changeset.changes;
            let module_store_id = ModuleStore::object_id();
            let mut new_modules = vec![];
            let mut updated_modules = vec![];
            for change in changes {
                if change.metadata.id != module_store_id {
                    continue;
                };

                for package_change in &change.fields {
                    let package_owner = package_change.metadata.owner.0;
                    for module_change in &package_change.fields {
                        let metadata = ObjectMeta::from(module_change.metadata.clone());

                        let value = module_change.value.clone().map(Op::<Vec<u8>>::from).ok_or(
                            RoochError::TransactionError(
                                "Module change value is missing".to_owned(),
                            ),
                        )?;
                        let (flag, bytes) = match value {
                            Op::New(bytes) => (0, bytes),
                            Op::Modify(bytes) => (1, bytes),
                            Op::Delete => unreachable!("Module will never be deleted"),
                        };
                        let object_state = ObjectState::new(metadata, bytes);
                        let module = object_state.value_as_df::<MoveString, MoveModule>()?;
                        let module_id = format!("{}::{}", package_owner, module.name);
                        if flag == 0 {
                            new_modules.push(module_id);
                        } else {
                            updated_modules.push(module_id);
                        }
                    }
                }
            }

            output.push_str("\n\nNew modules:");
            if new_modules.is_empty() {
                output.push_str("\n    None");
            } else {
                for module in new_modules {
                    output.push_str(&format!("\n    {}", module));
                }
            };
            output.push_str("\n\nUpdated modules:");
            if updated_modules.is_empty() {
                output.push_str("\n    None");
            } else {
                for module in updated_modules {
                    output.push_str(&format!("\n    {}", module));
                }
            };
        }

        Ok(output)
    }
}
