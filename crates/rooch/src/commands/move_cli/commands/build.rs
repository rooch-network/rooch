// Copyright (c) The Move Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::extended_checks::build::build_model;
use crate::extended_checks::build::BYTECODE_VERSION;
use crate::extended_checks::metadata::{run_extended_checks, RuntimeModuleMetadataV1};
use crate::move_cli::types::AccountAddressWrapper;
use clap::*;
use codespan_reporting::diagnostic::Severity;
use move_cli::base::reroot_path;
use move_command_line_common::files::MOVE_COMPILED_EXTENSION;
use move_compiler::compiled_unit::CompiledUnit;
use move_core_types::language_storage::ModuleId;
use move_core_types::metadata::Metadata;
use move_package::compilation::compiled_package::CompiledPackage;
use move_package::compilation::package_layout::CompiledPackageLayout;
use move_package::BuildConfig;
use std::{collections::BTreeMap, path::PathBuf};
use termcolor::ColorChoice;
use termcolor::StandardStream;

/// The keys used to identify the metadata in the metadata section of the module bytecode.
/// This is more or less arbitrary, besides we should use some unique key to identify
/// Rooch specific metadata (`rooch::` here).
pub static ROOCH_METADATA_KEY: &[u8] = "rooch::metadata_v0".as_bytes();

/// Build the package at `path`. If no path is provided defaults to current directory.
#[derive(Parser)]
#[clap(name = "build")]
pub struct Build {
    /// Named addresses for the move binary
    ///
    /// Example: alice=0x1234, bob=0x5678
    ///
    /// Note: This will fail if there are duplicates in the Move.toml file remove those first.
    #[clap(long, parse(try_from_str = moveos_common::utils::parse_map), default_value = "")]
    pub(crate) named_addresses: BTreeMap<String, AccountAddressWrapper>,
}

impl Build {
    pub fn execute(self, path: Option<PathBuf>, config: BuildConfig) -> anyhow::Result<()> {
        let mut config = config;
        config.additional_named_addresses = self
            .named_addresses
            .into_iter()
            .map(|(key, value)| (key, value.account_address))
            .collect();

        let additional_named_address = config.additional_named_addresses.clone();

        let rerooted_path = reroot_path(path)?;
        if config.fetch_deps_only {
            if config.test_mode {
                config.dev_mode = true;
            }
            config.download_deps_for_package(&rerooted_path, &mut std::io::stdout())?;
            return Ok(());
        }

        let mut package = config.compile_package_no_exit(&rerooted_path, &mut std::io::stdout())?;

        let model = &build_model(rerooted_path.as_path(), additional_named_address, None)?;

        let runtime_metadata = run_extended_checks(model);
        // println!("runtime metadata {:?}", runtime_metadata);

        if model.diag_count(Severity::Warning) > 0 {
            let mut error_writer = StandardStream::stderr(ColorChoice::Auto);
            model.report_diag(&mut error_writer, Severity::Warning);
            if model.has_errors() {
                eprintln!("extended checks failed")
            }
        }

        inject_runtime_metadata(
            rerooted_path
                .join(CompiledPackageLayout::Root.path())
                .join(package.compiled_package_info.package_name.as_str()),
            &mut package,
            runtime_metadata,
        );

        Ok(())
    }
}

fn inject_runtime_metadata(
    package_path: PathBuf,
    pack: &mut CompiledPackage,
    metadata: BTreeMap<ModuleId, RuntimeModuleMetadataV1>,
) {
    for unit_with_source in pack.root_compiled_units.iter_mut() {
        match &mut unit_with_source.unit {
            CompiledUnit::Module(named_module) => {
                if let Some(module_metadata) = metadata.get(&named_module.module.self_id()) {
                    if !module_metadata.is_empty() {
                        let serialized_metadata =
                            bcs::to_bytes(&module_metadata).expect("BCS for RuntimeModuleMetadata");
                        named_module.module.metadata.push(Metadata {
                            key: ROOCH_METADATA_KEY.to_vec(),
                            value: serialized_metadata,
                        });

                        // Also need to update the .mv file on disk.
                        let path = package_path
                            .join(CompiledPackageLayout::CompiledModules.path())
                            .join(named_module.name.as_str())
                            .with_extension(MOVE_COMPILED_EXTENSION);
                        if path.is_file() {
                            let bytes = unit_with_source
                                .unit
                                .serialize(Option::from(BYTECODE_VERSION));
                            std::fs::write(path, bytes).unwrap();
                        }
                    }
                }
            }
            CompiledUnit::Script(_) => {}
        }
    }
}
