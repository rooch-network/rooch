// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use moveos_types::addresses::MOVEOS_NAMED_ADDRESS_MAPPING;
use anyhow::Result;
use move_binary_format::CompiledModule;
use move_bytecode_utils::dependency_graph::DependencyGraph;
use move_command_line_common::address::NumericalAddress;
use move_core_types::account_address::AccountAddress;
use move_package::{compilation::compiled_package::CompiledPackage, BuildConfig};
use std::{collections::BTreeMap, io::stderr, path::PathBuf};

pub mod natives;

const ERROR_DESCRIPTIONS: &[u8] = include_bytes!("../error_description.errmap");

pub fn error_descriptions() -> &'static [u8] {
    ERROR_DESCRIPTIONS
}

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
        "rooch-framework"
    }

    pub fn named_addresses() -> BTreeMap<String, NumericalAddress> {
        let mut address_mapping = move_stdlib::move_stdlib_named_addresses();
        address_mapping.extend(
            MOVEOS_NAMED_ADDRESS_MAPPING
                .iter()
                .map(|(name, addr)| (name.to_string(), NumericalAddress::parse_str(addr).unwrap())),
        );
        address_mapping
    }

    /// Build moveos_stdlib package
    pub fn build() -> Result<Self> {
        let options = BuildOptions::default();
        Self::build_error_code_map();
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
            //TODO set bytecode version
            bytecode_version: None,
        };
        build_config.compile_package_no_exit(&package_path, &mut stderr())
    }

    pub fn build_error_code_map() {
        let _path = path_in_crate("error_description.errmap");
        //TODO generate error code map
    }

    pub fn modules(&self) -> Result<Vec<CompiledModule>> {
        //TODO ensure all module at same address.
        //include all module and dependency modules
        let modules = self.package.all_modules_map();
        let graph = DependencyGraph::new(modules.iter_modules());
        let order_modules = graph.compute_topological_order()?;
        Ok(order_modules.cloned().collect())
    }

    pub fn into_module_bundles(self) -> Result<Vec<Vec<u8>>> {
        let mut bundles = vec![];
        for module in self.modules()? {
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
        let moveos_stdlib = Framework::build().unwrap();
        let modules_count = moveos_stdlib.package.root_modules().count();
        let bundles = moveos_stdlib.into_module_bundles().unwrap();
        print!("modules_count:{}, bundles:{}", modules_count, bundles.len());
        assert!(bundles.len() > modules_count);
    }
}
