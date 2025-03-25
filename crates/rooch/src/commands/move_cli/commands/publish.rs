// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::cli_types::{CommandAction, TransactionOptions, WalletContextOptions};
use crate::tx_runner::dry_run_tx_locally;
use async_trait::async_trait;
use bytes::Bytes;
use clap::Parser;
use framework_builder::releaser;
use move_binary_format::errors::PartialVMResult;
use move_binary_format::CompiledModule;
use move_cli::Move;
use move_core_types::account_address::AccountAddress;
use move_core_types::effects::Op;
use move_core_types::{identifier::Identifier, language_storage::ModuleId};
use move_model::metadata::{CompilerVersion, LanguageVersion};
use move_vm_types::resolver::ModuleResolver;
use moveos_compiler::dependency_order::sort_by_dependency_order;
use moveos_types::access_path::AccessPath;
use moveos_types::move_std::string::MoveString;
use moveos_types::moveos_std::module_store::{ModuleStore, PackageData};
use moveos_types::moveos_std::move_module::MoveModule;
use moveos_types::moveos_std::object::ObjectMeta;
use moveos_types::{
    addresses::MOVEOS_STD_ADDRESS, move_types::FunctionId, state::ObjectState,
    transaction::MoveAction,
};
use moveos_verifier::build::run_verifier;
use rooch_rpc_api::jsonrpc_types::{ExecuteTransactionResponseView, HumanReadableDisplay};
use rooch_rpc_client::wallet_context::WalletContext;
use rooch_rpc_client::Client;
use rooch_types::address::RoochAddress;
use rooch_types::error::{RoochError, RoochResult};
use rooch_types::transaction::rooch::RoochTransaction;
use std::collections::BTreeMap;
use std::io::stderr;
use tokio::runtime::Handle;

struct MemoryModuleResolver {
    packages: BTreeMap<AccountAddress, BTreeMap<String, Vec<u8>>>,
    client: Client,
}

impl MemoryModuleResolver {
    fn new(client: Client) -> Self {
        Self {
            packages: BTreeMap::new(),
            client,
        }
    }

    fn download(&mut self, module_ids: Vec<ModuleId>) -> Result<(), anyhow::Error> {
        // group module_ids by ModuleId.address
        let mut package_group = BTreeMap::new();
        module_ids.into_iter().for_each(|mid| {
            package_group
                .entry(*mid.address())
                .or_insert_with(Vec::new)
                .push(Identifier::from(mid.name()))
        });

        // download each package
        for (package_address, module_names) in package_group {
            let access_path = AccessPath::modules(package_address, module_names);
            let mut modules = BTreeMap::new();
            tokio::task::block_in_place(|| {
                Handle::current().block_on(async {
                    let states = self.client.rooch.get_states(access_path, None).await?;

                    states.into_iter().try_for_each(|state_view| {
                        if let Some(sv) = state_view {
                            let state = ObjectState::from(sv);
                            let module = match state.value_as_df::<MoveString, MoveModule>() {
                                Ok(module) => module,
                                Err(e) => return Err(e),
                            };
                            modules.insert(
                                module.name.clone().as_str().to_owned(),
                                module.value.byte_codes,
                            );
                        };
                        Ok(())
                    })
                })
            })?;
            if !modules.is_empty() {
                self.packages.insert(package_address, modules);
            };
        }

        Ok(())
    }

    pub fn get_modules(
        &self,
        package_addr: &AccountAddress,
    ) -> Result<Vec<CompiledModule>, anyhow::Error> {
        let modules = self.packages.get(package_addr);
        if modules.is_none() {
            return Ok(vec![]);
        }
        let modules = modules.unwrap();
        let mut compiled_modules = vec![];
        for module_bytes in modules.values() {
            let compiled_module = CompiledModule::deserialize(module_bytes)?;
            compiled_modules.push(compiled_module);
        }
        Ok(compiled_modules)
    }
}

impl ModuleResolver for MemoryModuleResolver {
    fn get_module(&self, module_id: &ModuleId) -> PartialVMResult<Option<Bytes>> {
        let pkg_addr = module_id.address();
        let module_name = module_id.name();
        match self.packages.get(pkg_addr) {
            Some(modules) => {
                let module = modules.get(module_name.as_str());
                match module {
                    Some(module) => Ok(Some(Bytes::from(module.to_vec()))),
                    None => Ok(None),
                }
            }
            None => Ok(None),
        }
    }

    fn get_module_metadata(
        &self,
        _module_id: &ModuleId,
    ) -> Vec<move_core_types::metadata::Metadata> {
        unimplemented!("get_module_metadata not implemented")
    }
}

#[derive(Parser)]
pub struct Publish {
    /// Path to the package data file to publish
    #[clap(value_name = "PACKAGE_FILE")]
    pub package_file: Option<String>,

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
    /// `moveos_std::module_store::publish_package_entry`.
    /// **Deprecated**! Publish modules by `MoveAction::ModuleBundle` is no longer used anymore.
    /// So you should never add this option.
    /// For now, the option is kept for test only.
    #[clap(long)]
    pub by_move_action: bool,

    /// Return command outputs in json format
    #[clap(long, default_value = "false")]
    json: bool,

    /// Run the DryRun for this transaction
    #[clap(long, default_value = "false")]
    dry_run: bool,

    /// Skip the client side compatibility check
    #[clap(long, default_value = "false")]
    skip_client_compat_check: bool,
}

#[async_trait]
impl CommandAction<ExecuteTransactionResponseView> for Publish {
    async fn execute(self) -> RoochResult<ExecuteTransactionResponseView> {
        // Build context and handle errors
        let context = self.context_options.build_require_password()?;
        let max_gas_amount: Option<u64> = self.tx_options.max_gas_amount;
        let sender = context
            .resolve_address(self.tx_options.sender.clone())?
            .into();

        if let Some(ref package_file) = self.package_file {
            let file = std::fs::File::open(package_file)?;
            let pkg_data: PackageData = bcs::from_reader(file)?;
            eprintln!("Publish modules to address: {:?}", pkg_data.package_id);
            return self
                .publish_package(&context, sender, pkg_data, max_gas_amount)
                .await;
        }

        // Clone variables for later use
        let package_path = self
            .move_args
            .package_path
            .clone()
            .unwrap_or_else(|| std::env::current_dir().unwrap());
        let config = self.move_args.build_config.clone();
        let mut config = config.clone();
        config.compiler_config.language_version = Some(LanguageVersion::V2_1);
        config.compiler_config.compiler_version = Some(CompilerVersion::V2_1);

        // Parse named addresses from context and update config
        config.additional_named_addresses =
            context.parse_and_resolve_addresses(self.named_addresses.clone())?;
        let config_cloned = config.clone();

        // Compile the package and run the verifier
        let (mut package, _) =
            config.compile_package_no_exit(&package_path, vec![], &mut stderr())?;
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

        // Download all modules from remote
        let all_module_ids = package
            .all_modules_map()
            .get_map()
            .iter()
            .map(|(mid, _)| mid.clone())
            .collect::<Vec<_>>();

        for module in &sorted_modules {
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

        //Because the verify modules function will load many modules from the rpc server,
        //We need to download all modules in one rpc request and then verify the modules.
        let mut resolver = MemoryModuleResolver::new(context.get_client().await?);
        resolver.download(all_module_ids)?;
        moveos_verifier::verifier::verify_modules(&sorted_modules, &resolver)?;
        let old_modules = resolver.get_modules(&pkg_address)?;
        if !old_modules.is_empty() && !self.skip_client_compat_check {
            releaser::check_modules_compat(sorted_modules, old_modules, true)?;
        }

        // Create a sender RoochAddress
        eprintln!("Publish modules to address: {}", pkg_address);

        // Prepare and execute the transaction based on the action type
        let tx_result = if !self.by_move_action {
            let pkg_data = PackageData::new(
                MoveString::from(package.compiled_package_info.package_name.as_str()),
                pkg_address,
                bundles,
            );
            self.publish_package(&context, sender, pkg_data, max_gas_amount)
                .await?
        } else {
            // Handle MoveAction.ModuleBundle case
            let action = MoveAction::ModuleBundle(bundles);
            let tx_data = context
                .build_tx_data(sender, action, max_gas_amount)
                .await?;
            context.sign_and_execute(sender, tx_data).await?
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
        output.push_str(&exe_info.to_human_readable_string(false, 0));

        if let Some(txn_output) = &txn_response.output {
            // print error info
            if let Some(error_info) = txn_response.clone().error_info {
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
                        let module_id = ModuleId::new(
                            package_owner.into(),
                            Identifier::new(format!("{}", module.name))?,
                        );
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
                    output.push_str(&format!("\n    {}", module.short_str_lossless()));
                }
            };
            output.push_str("\n\nUpdated modules:");
            if updated_modules.is_empty() {
                output.push_str("\n    None");
            } else {
                for module in updated_modules {
                    output.push_str(&format!("\n    {}", module.short_str_lossless()));
                }
            };

            // print objects changes
            output.push_str("\n\n");
            output.push_str(
                txn_output
                    .changeset
                    .to_human_readable_string(false, 0)
                    .as_str(),
            );
        }

        Ok(output)
    }

    async fn publish_package(
        &self,
        context: &WalletContext,
        sender: RoochAddress,
        pkg_data: PackageData,
        max_gas_amount: Option<u64>,
    ) -> RoochResult<ExecuteTransactionResponseView> {
        let pkg_bytes = bcs::to_bytes(&pkg_data).unwrap();
        let args = bcs::to_bytes(&pkg_bytes).unwrap();
        let action = MoveAction::new_function_call(
            FunctionId::new(
                ModuleId::new(
                    MOVEOS_STD_ADDRESS,
                    Identifier::new("module_store".to_owned()).unwrap(),
                ),
                Identifier::new("publish_package_entry".to_owned()).unwrap(),
            ),
            vec![],
            vec![args],
        );

        if self.dry_run {
            let rooch_tx_data = context
                .build_tx_data(sender, action.clone(), max_gas_amount)
                .await?;
            let dry_run_result =
                dry_run_tx_locally(context.get_client().await?, rooch_tx_data).await?;

            return Ok(dry_run_result.into());
        }

        let tx_data = context
            .build_tx_data(sender, action, max_gas_amount)
            .await?;

        // Handle transaction with or without authenticator
        match &self.tx_options.authenticator {
            Some(authenticator) => {
                let tx = RoochTransaction::new(tx_data, authenticator.clone().into());
                context.execute(tx).await
            }
            None => context.sign_and_execute(sender, tx_data).await,
        }
    }
}
