// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use anyhow::{ensure, Result};
use dependency_order::sort_by_dependency_order;
use move_binary_format::CompiledModule;
use move_command_line_common::address::NumericalAddress;
use move_core_types::account_address::AccountAddress;
use move_package::{compilation::compiled_package::CompiledPackage, BuildConfig};
use moveos_types::addresses::MOVEOS_NAMED_ADDRESS_MAPPING;
use moveos_verifier::build::run_verifier;
use std::{collections::BTreeMap, io::stderr, path::PathBuf};

pub mod dependency_order;

#[derive(Debug, Clone)]
pub struct Stdlib {
    packages: Vec<StdlibPackage>,
}

#[derive(Debug, Clone)]
pub struct StdlibPackage {
    pub genesis_account: AccountAddress,
    pub path: PathBuf,
    pub package: CompiledPackage,
}

impl StdlibPackage {
    pub fn modules(&self) -> Result<Vec<CompiledModule>> {
        //include all root module, but do not include dependency modules
        let modules = self.package.root_modules_map();
        sort_by_dependency_order(modules.iter_modules())
    }
}

#[derive(Debug, Clone, Default)]
pub struct BuildOptions {
    pub named_addresses: BTreeMap<String, AccountAddress>,
    pub with_abis: bool,
    pub install_dir: Option<PathBuf>,
    pub skip_fetch_latest_git_deps: bool,
}

impl Stdlib {
    ///MoveOS builtin packages
    pub fn builtin_packages() -> [&'static str; 3] {
        //TODO move out rooch_framework and as a external framework arguments
        [
            "../moveos-stdlib/move-stdlib",
            "../moveos-stdlib/moveos-stdlib",
            "../../crates/rooch-framework",
        ]
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

    /// Build the MoveOS stdlib with exernal frameworks.
    /// The move_stdlib and moveos_stdlib packages are always built-in.
    pub fn build(option: BuildOptions) -> Result<Self> {
        //TODO build error map
        //Self::build_error_code_map();
        let mut packages = vec![];
        for stdlib in Self::builtin_packages().into_iter() {
            packages.push(Self::build_package(path_in_crate(stdlib), option.clone())?);
        }

        Ok(Self { packages })
    }

    pub fn build_package(package_path: PathBuf, options: BuildOptions) -> Result<StdlibPackage> {
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
        let mut compiled_package =
            build_config.compile_package_no_exit(&package_path, &mut stderr())?;

        let additional_named_address = options.named_addresses;
        run_verifier(
            &package_path,
            additional_named_address,
            &mut compiled_package,
        );
        let module_map = compiled_package.root_modules_map();
        let mut modules = module_map.iter_modules().into_iter();

        let genesis_account = *modules
            .next()
            .expect("the package must have one module at least")
            .self_id()
            .address();
        for module in modules {
            ensure!(
                module.self_id().address() == &genesis_account,
                "all modules must have same address"
            );
        }
        Ok(StdlibPackage {
            genesis_account,
            path: package_path,
            package: compiled_package,
        })
    }

    pub fn build_error_code_map() {
        let _path = path_in_crate("error_description.errmap");
        //TODO generate error code map
    }

    pub fn all_modules(&self) -> Result<Vec<CompiledModule>> {
        let mut modules = vec![];
        for package in self.packages.iter() {
            modules.extend(package.modules()?);
        }
        Ok(modules)
    }

    pub fn module_bundles(&self) -> Result<Vec<(AccountAddress, Vec<Vec<u8>>)>> {
        let mut bundles = vec![];
        for package in &self.packages {
            let mut module_bundle = vec![];
            for module in package.modules()? {
                let mut binary = vec![];
                module.serialize(&mut binary)?;
                module_bundle.push(binary);
            }
            bundles.push((package.genesis_account, module_bundle));
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
        let moveos_stdlib = Stdlib::build(BuildOptions::default()).unwrap();
        for stdlib_package in moveos_stdlib.packages {
            println!(
                "stdlib package: {}, path: {:?}, modules_count:{}",
                stdlib_package.genesis_account.short_str_lossless(),
                stdlib_package.path,
                stdlib_package.modules().unwrap().len()
            );
        }
    }
}
