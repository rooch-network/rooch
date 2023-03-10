// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use anyhow::Result;
use move_bytecode_utils::dependency_graph::DependencyGraph;
use move_core_types::account_address::AccountAddress;
use move_package::{compilation::compiled_package::CompiledPackage, BuildConfig};
use std::{
    collections::{BTreeMap, HashMap, HashSet},
    io::stderr,
    path::PathBuf,
};

pub struct Framework {
    package: CompiledPackage,
}

#[derive(Debug, Clone, Default)]
pub struct BuildOptions {
    pub named_addresses: BTreeMap<String, AccountAddress>,
    pub with_abis: bool,
    pub install_dir: Option<PathBuf>,
    pub skip_fetch_latest_git_deps: bool,
}

impl Framework {
    pub fn package() -> &'static str {
        "moveos-stdlib"
    }

    /// Build framework package
    pub fn build() -> Result<Self> {
        let options = BuildOptions::default();
        let package = Self::build_package(path_in_crate(Self::package()), options)?;
        Ok(Self { package })
    }

    pub fn build_package(package_path: PathBuf, options: BuildOptions) -> Result<CompiledPackage> {
        let build_config = BuildConfig {
            dev_mode: false,
            additional_named_addresses: options.named_addresses.clone(),
            architecture: None,
            generate_abis: options.with_abis,
            generate_docs: false,
            install_dir: options.install_dir.clone(),
            test_mode: false,
            force_recompilation: false,
            fetch_deps_only: false,
            skip_fetch_latest_git_deps: options.skip_fetch_latest_git_deps,
            lock_file: None,
        };
        build_config.compile_package_no_exit(&package_path, &mut stderr())
    }

    pub fn into_module_bundles(self) -> Result<Vec<Vec<u8>>> {
        let mut bundles = vec![];
        //TODO ensure all module at same address.
        //include all module and dependency modules
        let modules = self.package.all_modules_map();
        let graph = DependencyGraph::new(modules.iter_modules());
        for module in graph.compute_topological_order()? {
            let mut binary = vec![];
            module.serialize(&mut binary)?;
            bundles.push(binary);
        }
        Ok(bundles)
    }
}

pub fn path_in_crate<S>(relative: S) -> PathBuf
where
    S: Into<String>,
{
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(relative.into())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_package() {
        let framework = Framework::build().unwrap();
        let modules_count = framework.package.root_modules().count();
        let bundles = framework.into_module_bundles().unwrap();
        print!("modules_count:{}, bundles:{}", modules_count, bundles.len());
        assert!(bundles.len() > modules_count);
    }
}
