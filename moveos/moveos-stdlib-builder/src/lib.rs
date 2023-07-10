// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use anyhow::{ensure, Result};
use dependency_order::sort_by_dependency_order;
use move_binary_format::{errors::Location, CompiledModule};
use move_cli::base::docgen::Docgen;
use move_core_types::account_address::AccountAddress;
use move_package::{compilation::compiled_package::CompiledPackage, BuildConfig};
use moveos_verifier::build::run_verifier;
use serde::{Deserialize, Serialize};
use std::{
    collections::BTreeMap,
    io::stderr,
    path::{Path, PathBuf},
};

pub mod dependency_order;

#[derive(Clone, Debug, Hash, Eq, PartialEq, Serialize, Deserialize)]
pub struct Stdlib {
    packages: Vec<StdlibPackage>,
}

#[derive(Clone, Debug, Hash, Eq, PartialEq, Serialize, Deserialize)]
pub struct StdlibPackage {
    pub genesis_account: AccountAddress,
    pub modules: Vec<Vec<u8>>,
}

impl StdlibPackage {
    pub fn new(genesis_account: AccountAddress, compiled_package: CompiledPackage) -> Result<Self> {
        //include all root module, but do not include dependency modules
        let modules = compiled_package.root_modules_map();
        let modules = sort_by_dependency_order(modules.iter_modules())?
            .into_iter()
            .map(|module| {
                let mut bytes = vec![];
                module.serialize(&mut bytes)?;
                Ok(bytes)
            })
            .collect::<Result<Vec<Vec<u8>>>>()?;
        Ok(Self {
            genesis_account,
            modules,
        })
    }

    pub fn modules(&self) -> Result<Vec<CompiledModule>> {
        self.modules
            .iter()
            .map(|module| {
                let compiled_module = CompiledModule::deserialize(module.as_slice())
                    .map_err(|e| e.finish(Location::Undefined))?;
                Ok(compiled_module)
            })
            .collect::<Result<Vec<CompiledModule>>>()
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
        let mut compiled_package = build_config
            .clone()
            .compile_package_no_exit(&package_path, &mut stderr())?;

        run_verifier(&package_path, build_config.clone(), &mut compiled_package)?;
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
        Self::build_doc(package_path.as_path(), build_config)?;
        StdlibPackage::new(genesis_account, compiled_package)
    }

    fn build_doc(package_path: &Path, build_config: BuildConfig) -> Result<()> {
        let current_path = std::env::current_dir().unwrap();
        let docgen = Docgen {
            section_level_start: Option::None,
            exclude_private_fun: true,
            exclude_specs: true,
            independent_specs: false,
            exclude_impl: false,
            toc_depth: Option::None,
            no_collapsed_sections: false,
            output_directory: None,
            template: vec!["doc_template/README.md".to_owned()],
            references_file: Option::None,
            include_dep_diagrams: false,
            include_call_diagrams: false,
            compile_relative_to_output_dir: false,
        };
        docgen.execute(Option::Some(package_path.to_path_buf()), build_config)?;
        std::env::set_current_dir(current_path).unwrap();
        Ok(())
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

fn path_in_crate<S>(relative: S) -> PathBuf
where
    S: AsRef<Path>,
{
    let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    path.push(relative);
    path
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_package() {
        let moveos_stdlib = Stdlib::build(BuildOptions::default()).unwrap();
        for stdlib_package in moveos_stdlib.packages {
            println!(
                "stdlib package: {}, modules_count:{}",
                stdlib_package.genesis_account.short_str_lossless(),
                stdlib_package.modules().unwrap().len()
            );
        }
    }
}
