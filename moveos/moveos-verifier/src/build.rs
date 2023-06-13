use crate::metadata::{run_extended_checks, RuntimeModuleMetadataV1};
use codespan_reporting::diagnostic::Severity;
use move_command_line_common::files::MOVE_COMPILED_EXTENSION;
use move_compiler::compiled_unit::CompiledUnit;
use move_core_types::account_address::AccountAddress;
use move_core_types::language_storage::ModuleId;
use move_core_types::metadata::Metadata;
use move_model::model::GlobalEnv;
use move_package::compilation::compiled_package::CompiledPackage;
use move_package::compilation::package_layout::CompiledPackageLayout;
use move_package::{BuildConfig, ModelConfig};
use std::collections::BTreeMap;
use std::path::Path;
use termcolor::{ColorChoice, StandardStream};

pub const BYTECODE_VERSION: u32 = 6;

/// The keys used to identify the metadata in the metadata section of the module bytecode.
/// This is more or less arbitrary, besides we should use some unique key to identify
/// Rooch specific metadata (`rooch::` here).
pub static ROOCH_METADATA_KEY: &[u8] = "rooch::metadata_v0".as_bytes();

pub fn build_model(
    package_path: &Path,
    additional_named_addresses: BTreeMap<String, AccountAddress>,
    target_filter: Option<String>,
) -> anyhow::Result<GlobalEnv> {
    let build_config = BuildConfig {
        dev_mode: false,
        additional_named_addresses,
        architecture: None,
        generate_abis: false,
        generate_docs: false,
        install_dir: None,
        test_mode: false,
        force_recompilation: false,
        fetch_deps_only: false,
        skip_fetch_latest_git_deps: true,
        bytecode_version: Some(BYTECODE_VERSION),
        lock_file: None,
    };
    build_config.move_model_for_package(
        package_path,
        ModelConfig {
            target_filter,
            all_files_as_targets: false,
        },
    )
}

pub fn run_verifier<P: AsRef<Path>>(
    package_path: P,
    additional_named_address: BTreeMap<String, AccountAddress>,
    package: &mut CompiledPackage,
) {
    let model = build_model(package_path.as_ref(), additional_named_address, None).unwrap();

    let runtime_metadata = run_extended_checks(&model);

    if model.diag_count(Severity::Warning) > 0 {
        let mut error_writer = StandardStream::stderr(ColorChoice::Auto);
        model.report_diag(&mut error_writer, Severity::Warning);
        if model.has_errors() {
            eprintln!("extended checks failed")
        }
    }

    inject_runtime_metadata(
        package_path
            .as_ref()
            .join(CompiledPackageLayout::Root.path())
            .join(package.compiled_package_info.package_name.as_str()),
        package,
        runtime_metadata,
    );
}

pub fn inject_runtime_metadata<P: AsRef<Path>>(
    package_path: P,
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
                            .as_ref()
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
