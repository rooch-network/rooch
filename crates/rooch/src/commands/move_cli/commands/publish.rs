// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use async_trait::async_trait;
use clap::Parser;

use move_binary_format::file_format::CompiledModule;
use move_bytecode_utils::dependency_graph::DependencyGraph;
use move_bytecode_utils::Modules;
use move_cli::Move;
use rooch_rpc_api::jsonrpc_types::ExecuteTransactionResponseView;

use crate::cli_types::{CommandAction, TransactionOptions, WalletContextOptions};
use moveos::vm::dependency_order::sort_by_dependency_order;
use moveos_types::transaction::MoveAction;
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

        let additional_named_address = config.additional_named_addresses.clone();

        let package_path = match package_path {
            Some(package_path) => package_path,
            None => std::env::current_dir()?,
        };
        let mut package = config.compile_package_no_exit(&package_path, &mut stderr())?;

        run_verifier(package_path, additional_named_address, &mut package)?;

        // let modules = package.root_modules_map().iter_modules_owned();
        let modules = package.root_modules_map();

        // let pkg_address = modules..iter_modules_owned() modules[0].self_id().address().to_owned();
        let pkg_address = modules.iter_modules_owned()[0]
            .self_id()
            .address()
            .to_owned();
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

        let action = MoveAction::ModuleBundle(bundles);

        let sender: RoochAddress = pkg_address.into();
        eprintln!("Publish modules to address: {:?}", sender);
        context.sign_and_execute(sender, action).await
    }
}
