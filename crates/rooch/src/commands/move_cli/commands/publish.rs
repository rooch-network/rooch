// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use async_trait::async_trait;
use clap::Parser;

use move_binary_format::file_format::CompiledModule;
use move_bytecode_utils::dependency_graph::DependencyGraph;
use move_bytecode_utils::Modules;
use move_cli::Move;
use move_core_types::{identifier::Identifier, language_storage::ModuleId};
use rooch_rpc_api::jsonrpc_types::ExecuteTransactionResponseView;
use rooch_types::multichain_id::RoochMultiChainID;
use rooch_types::transaction::rooch::RoochTransaction;

use crate::cli_types::{CommandAction, TransactionOptions, WalletContextOptions};
use moveos::vm::dependency_order::sort_by_dependency_order;
use moveos_types::{
    addresses::MOVEOS_STD_ADDRESS, move_types::FunctionId, transaction::MoveAction,
};
use moveos_verifier::build::run_verifier;
use rooch_types::address::RoochAddress;
use rooch_types::error::{RoochError, RoochResult};
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
    /// Example: alice=0x1234, bob=0x5678
    ///
    /// Note: This will fail if there are duplicates in the Move.toml file remove those first.
    #[clap(long, parse(try_from_str = crate::utils::parse_map), default_value = "")]
    pub(crate) named_addresses: BTreeMap<String, String>,

    /// Command line input of multichain ids
    #[clap(short = 'm', long = "multichain-id", default_value = "20230103")]
    pub multichain_id: RoochMultiChainID,

    /// Whether publish modules by `MoveAction::ModuleBundle`?
    /// If not set, publish moduels through Move entry function
    /// `moveos_std::account_storage::publish_modules_entry`
    #[clap(long, parse(from_flag))]
    pub by_move_action: bool,
}

impl Publish {
    pub fn order_modules(modules: Modules) -> anyhow::Result<Vec<CompiledModule>> {
        //TODO ensure all module at same address.
        //include all module and dependency modules
        // let modules = self.package.all_modules_map();
        let graph = DependencyGraph::new(modules.iter_modules());
        let order_modules = graph.compute_topological_order()?;
        Ok(order_modules.cloned().collect())
    }
}

#[async_trait]
impl CommandAction<ExecuteTransactionResponseView> for Publish {
    async fn execute(self) -> RoochResult<ExecuteTransactionResponseView> {
        let context = self.context_options.build().await?;

        let package_path = self.move_args.package_path;
        let config = self.move_args.build_config;
        let mut config = config.clone();

        config.additional_named_addresses = context.parse_account_args(self.named_addresses)?;
        let config_cloned = config.clone();

        let package_path = match package_path {
            Some(package_path) => package_path,
            None => std::env::current_dir()?,
        };
        let mut package = config.compile_package_no_exit(&package_path, &mut stderr())?;

        run_verifier(package_path, config_cloned, &mut package)?;

        // let modules = package.root_modules_map().iter_modules_owned();
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
        let mut bundles: Vec<Vec<u8>> = vec![];
        // let sorted_modules = Self::order_modules(modules)?;
        let sorted_modules = sort_by_dependency_order(modules.iter_modules())?;
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

        if self.tx_options.sender_account.is_some()
            && pkg_address != context.parse_account_arg(self.tx_options.sender_account.unwrap())?
        {
            return Err(RoochError::CommandArgumentError(
                    "--sender-account required and the sender account must be the same as the package address"
                    .to_string(),
            ));
        }

        let sender: RoochAddress = pkg_address.into();
        eprintln!("Publish modules to address: {:?}", sender);
        let tx_result = if !self.by_move_action {
            let args = bcs::to_bytes(&bundles).unwrap();
            let action = MoveAction::new_function_call(
                FunctionId::new(
                    ModuleId::new(
                        MOVEOS_STD_ADDRESS,
                        Identifier::new("account_storage".to_owned()).unwrap(),
                    ),
                    Identifier::new("publish_modules_entry".to_owned()).unwrap(),
                ),
                vec![],
                vec![args],
            );
            match self.tx_options.authenticator {
                Some(authenticator) => {
                    let tx_data = context.build_rooch_tx_data(sender, action).await?;
                    //TODO the authenticator usually is associalted with the RoochTransactinData
                    //So we need to find a way to let user generate the authenticator based on the tx_data.
                    let tx = RoochTransaction::new(tx_data, authenticator.into());
                    context.execute(tx).await?
                }
                None => {
                    context
                        .sign_and_execute(sender, action, self.multichain_id)
                        .await?
                }
            }
        } else {
            let action = MoveAction::ModuleBundle(bundles);
            context
                .sign_and_execute(sender, action, self.multichain_id)
                .await?
        };
        context.assert_execute_success(tx_result)
    }
}
