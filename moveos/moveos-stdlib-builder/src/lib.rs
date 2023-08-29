// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use anyhow::{ensure, Result};
use codespan_reporting::term::termcolor::{ColorChoice, StandardStream};
use dependency_order::sort_by_dependency_order;
use move_binary_format::{errors::Location, CompiledModule};
use move_core_types::account_address::AccountAddress;
use move_model::model::GlobalEnv;
use move_package::{compilation::compiled_package::CompiledPackage, BuildConfig, ModelConfig};
use moveos_verifier::build::run_verifier;
use serde::{Deserialize, Serialize};
use std::{
    fs::{self, File},
    io::{stderr, Write},
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

#[derive(Debug, Clone)]
pub struct StdlibBuildConfig {
    // The path of the stdlib project
    pub path: PathBuf,
    pub error_prefix: String,
    pub error_code_map_output_file: PathBuf,
    pub document_template: PathBuf,
    pub document_output_directory: PathBuf,
    pub build_config: BuildConfig,
}

impl StdlibBuildConfig {
    pub fn build(self, deps: &Vec<StdlibBuildConfig>) -> Result<StdlibPackage> {
        let package_path = self.path.clone();
        let mut compiled_package = self
            .build_config
            .clone()
            .compile_package_no_exit(&package_path, &mut stderr())?;

        run_verifier(
            &package_path,
            self.build_config.clone(),
            &mut compiled_package,
        )?;
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
        let model = self.build_config.clone().move_model_for_package(
            &self.path,
            ModelConfig {
                all_files_as_targets: false,
                target_filter: None,
            },
        )?;

        let deps_doc_paths = deps
            .iter()
            .map(|dep| {
                pathdiff::diff_paths(dep.document_output_directory.as_path(), self.path.as_path())
                    .expect("path diff return none")
                    .to_string_lossy()
                    .to_string()
            })
            .collect::<Vec<String>>();

        self.build_doc(&model, deps_doc_paths)?;
        self.build_error_code_map(&model)?;

        if model.has_errors() {
            let mut error_writer = StandardStream::stderr(ColorChoice::Auto);
            model.report_diag(
                &mut error_writer,
                codespan_reporting::diagnostic::Severity::Error,
            );
        }
        anyhow::ensure!(
            !model.has_errors(),
            "Errors encountered while build stdlib!"
        );

        StdlibPackage::new(genesis_account, compiled_package)
    }

    fn build_doc(&self, model: &GlobalEnv, deps_doc_paths: Vec<String>) -> Result<()> {
        fs::remove_dir_all(self.document_output_directory.as_path())?;
        println!("Generated move documents at {:?}, deps: {:?}", self.document_output_directory.as_path(), deps_doc_paths);
        let options = move_docgen::DocgenOptions {
            root_doc_templates: vec![self.document_template.to_string_lossy().to_string()],
            include_specs: false,
            include_impl: true,
            include_private_fun: false,
            output_directory: self.document_output_directory.to_string_lossy().to_string(),
            compile_relative_to_output_dir: false,
            doc_path: deps_doc_paths,
            ..Default::default()
        };

        let generator = move_docgen::Docgen::new(model, &options);

        for (file, content) in generator.gen() {
            let path = PathBuf::from(&file);
            fs::create_dir_all(path.parent().unwrap())?;
            fs::write(path.as_path(), content)?;
        }
        Ok(())
    }

    fn build_error_code_map(&self, model: &GlobalEnv) -> Result<()> {
        let error_map_gen_opt = move_errmapgen::ErrmapOptions {
            error_prefix: self.error_prefix.clone(),
            output_file: self
                .error_code_map_output_file
                .to_string_lossy()
                .to_string(),
            ..Default::default()
        };

        let mut errmap_gen = move_errmapgen::ErrmapGen::new(model, &error_map_gen_opt);
        errmap_gen.gen();
        errmap_gen.save_result();

        Ok(())
    }
}

impl Stdlib {

    /// Build the stdlib or framework packages
    pub fn build(build_configs: Vec<StdlibBuildConfig>) -> Result<Self> {
        let mut packages = vec![];
        let mut deps = vec![];
        for build_config in build_configs {
            packages.push(build_config.clone().build(&deps)?);
            deps.push(build_config);
        }
        Ok(Self { packages })
    }

    pub fn decode(bytes: &[u8]) -> Result<Self> {
        let stdlib = bcs::from_bytes(bytes)?;
        Ok(stdlib)
    }

    pub fn load_from_file<P: AsRef<Path>>(file: P) -> Result<Self> {
        let stdlib = bcs::from_bytes(&std::fs::read(file)?)?;
        Ok(stdlib)
    }

    pub fn save_to_file<P: AsRef<Path>>(&self, file: P) -> Result<()> {
        let mut file = File::create(file)?;
        let contents = bcs::to_bytes(&self)?;
        file.write_all(&contents)?;
        Ok(())
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
