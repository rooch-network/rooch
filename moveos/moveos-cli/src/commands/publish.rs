// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use clap::Parser;
use move_binary_format::CompiledModule;
use move_bytecode_utils::dependency_graph::DependencyGraph;
use move_cli::base::reroot_path;
use move_package::BuildConfig;
use moveos::types::transaction::{MoveTransaction, SimpleTransaction};
use moveos_client::Client;
use std::io::stderr;
use std::path::PathBuf;

#[derive(Parser)]
pub struct Publish {
    #[clap(flatten)]
    client: Client,
}

impl Publish {
    pub async fn execute(
        self,
        package_path: Option<PathBuf>,
        config: BuildConfig,
    ) -> anyhow::Result<()> {
        let rerooted_path = reroot_path(package_path)?;
        let package = config.compile_package_no_exit(&rerooted_path, &mut stderr())?;

        //TODO ensure all module at same address.
        //include all module and dependency modules
        let modules = package.all_modules_map();
        let graph = DependencyGraph::new(modules.iter_modules());
        let order_modules = graph.compute_topological_order()?;
        let modules: Vec<CompiledModule> = order_modules.cloned().collect();

        // TODO: check all modules at same address
        let id = modules[0].self_id();
        // List all module ids and unique them
        let mut module_ids = modules
            .clone()
            .into_iter()
            .map(|m| m.self_id().address().clone())
            .collect::<Vec<_>>();
        module_ids.sort();
        module_ids.dedup();

        println!("module id: {:?}", module_ids);

        let mut bundles: Vec<Vec<u8>> = vec![];
        for module in modules {
            let mut binary: Vec<u8> = vec![];
            module.serialize(&mut binary)?;
            bundles.push(binary);
        }

        let sender = *id.address();
        let txn = MoveTransaction::ModuleBundle(bundles);
        let txn = SimpleTransaction::new(sender, txn);
        self.client.submit(txn).await?;
        Ok(())
    }
}
