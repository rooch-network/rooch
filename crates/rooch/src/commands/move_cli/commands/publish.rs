// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::move_cli::types::{AccountAddressWrapper, TransactionOptions};
use anyhow::ensure;
use clap::Parser;
use move_binary_format::file_format::CompiledModule;
use move_bytecode_utils::dependency_graph::DependencyGraph;
use move_package::BuildConfig;
use moveos::vm::dependency_order::sort_by_dependency_order;
use moveos_client::Client;
use moveos_types::transaction::{MoveTransaction, SimpleTransaction};
use std::collections::BTreeMap;
use std::io::stderr;
use std::path::PathBuf;

use move_bytecode_utils::Modules;

#[derive(Parser)]
pub struct Publish {
    #[clap(flatten)]
    client: Client,

    #[clap(flatten)]
    txn_options: TransactionOptions,

    /// Named addresses for the move binary
    ///
    /// Example: alice=0x1234, bob=0x5678
    ///
    /// Note: This will fail if there are duplicates in the Move.toml file remove those first.
    #[clap(long, parse(try_from_str = moveos_common::utils::parse_map), default_value = "")]
    pub(crate) named_addresses: BTreeMap<String, AccountAddressWrapper>,
}

impl Publish {
    pub async fn execute(
        self,
        package_path: Option<PathBuf>,
        config: BuildConfig,
    ) -> anyhow::Result<()> {
        let mut config = config.clone();
        config.additional_named_addresses = self
            .named_addresses
            .clone()
            .into_iter()
            .map(|(key, value)| (key, value.account_address))
            .collect();

        let package_path = match package_path {
            Some(package_path) => package_path,
            None => std::env::current_dir()?,
        };
        let package = config.compile_package_no_exit(&package_path, &mut stderr())?;
        // let modules = package.root_modules_map().iter_modules_owned();
        let modules = package.root_modules_map();

        // let pkg_address = modules..iter_modules_owned() modules[0].self_id().address().to_owned();
        let pkg_address = modules.iter_modules_owned()[0]
            .self_id()
            .address()
            .to_owned();
        let mut bundles: Vec<Vec<u8>> = vec![];
        println!("Packaging Modules:");
        // let sorted_modules = Self::order_modules(modules)?;
        let sorted_modules = sort_by_dependency_order(modules.iter_modules())?;
        for module in sorted_modules {
            println!("\t {}", module.self_id());
            let module_address = module.self_id().address().to_owned();
            ensure!(
                module_address == pkg_address,
                "module's address ({:?}) not same as package module address {:?}",
                module_address,
                pkg_address.clone(),
            );
            let mut binary: Vec<u8> = vec![];
            module.serialize(&mut binary)?;
            bundles.push(binary);
        }
        assert!(
            self.txn_options.sender_account.is_some()
                && pkg_address == self.txn_options.sender_account.unwrap(),
            "--sender-account required and the sender account must be the same as the package address"
        );
        let txn = MoveTransaction::ModuleBundle(bundles);
        let txn = SimpleTransaction::new(pkg_address, txn);
        self.client.submit_txn(txn).await?;
        Ok(())
    }

    pub fn order_modules(modules: Modules) -> anyhow::Result<Vec<CompiledModule>> {
        //TODO ensure all module at same address.
        //include all module and dependency modules
        // let modules = self.package.all_modules_map();
        let graph = DependencyGraph::new(modules.iter_modules());
        let order_modules = graph.compute_topological_order()?;
        Ok(order_modules.cloned().collect())
    }
}
