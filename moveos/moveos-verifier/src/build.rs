// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::metadata::{run_extended_checks, RuntimeModuleMetadataV1};
use codespan_reporting::diagnostic::Severity;
use itertools::Itertools;
use move_binary_format::CompiledModule;
use move_command_line_common::address::NumericalAddress;
use move_command_line_common::files::MOVE_COMPILED_EXTENSION;
use move_compiler::compiled_unit::CompiledUnit;
use move_compiler::shared::{NamedAddressMap, PackagePaths};
use move_compiler::Flags;
use move_core_types::account_address::AccountAddress;
use move_core_types::language_storage::ModuleId;
use move_core_types::metadata::Metadata;
use move_ir_types::ast::CopyableVal_;
use move_ir_types::ast::Exp_;
use move_ir_types::ast::Metadata as ASTMetadata;
use move_model::model::GlobalEnv;
use move_model::options::ModelBuilderOptions;
use move_model::run_model_builder_with_options_and_compilation_flags;
use move_package::compilation::compiled_package::CompiledPackage;
use move_package::compilation::package_layout::CompiledPackageLayout;
use move_package::resolution::resolution_graph::{
    Renaming, ResolvedGraph, ResolvedPackage, ResolvedTable,
};
use move_package::{BuildConfig, ModelConfig};
use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;
use termcolor::{ColorChoice, StandardStream};

#[derive(Debug, Clone)]
pub struct ModelBuilder {
    resolution_graph: ResolvedGraph,
    model_config: ModelConfig,
}

impl ModelBuilder {
    pub fn create(resolution_graph: ResolvedGraph, model_config: ModelConfig) -> Self {
        Self {
            resolution_graph,
            model_config,
        }
    }

    // NOTE: If there are now renamings, then the root package has the global resolution of all named
    // addresses in the package graph in scope. So we can simply grab all of the source files
    // across all packages and build the Move model from that.
    // TODO: In the future we will need a better way to do this to support renaming in packages
    // where we want to support building a Move model.
    pub fn build_model(&self) -> anyhow::Result<GlobalEnv> {
        // Make sure no renamings have been performed
        if let Some(pkg_name) = self.resolution_graph.contains_renaming() {
            anyhow::bail!(
                "Found address renaming in package '{}' when \
                    building Move model -- this is currently not supported",
                pkg_name
            )
        }

        // Targets are all files in the root package
        let root_name = &self.resolution_graph.root_package.package.name;
        let root_package = self.resolution_graph.get_package(root_name).clone();
        let deps_source_info = self
            .resolution_graph
            .package_table
            .iter()
            .filter_map(|(nm, pkg)| {
                if nm == root_name {
                    return None;
                }
                let mut dep_source_paths = pkg
                    .get_sources(&self.resolution_graph.build_options)
                    .unwrap();
                let mut source_available = true;
                // If source is empty, search bytecode
                if dep_source_paths.is_empty() {
                    dep_source_paths = pkg.get_bytecodes().unwrap();
                    source_available = false;
                }
                Some(Ok((
                    *nm,
                    dep_source_paths,
                    &pkg.resolution_table,
                    source_available,
                )))
            })
            .collect::<anyhow::Result<Vec<_>>>()?;
        let (target, deps) = make_source_and_deps_for_compiler(
            &self.resolution_graph,
            &root_package,
            deps_source_info,
        )?;
        let (all_targets, all_deps) = if self.model_config.all_files_as_targets {
            let mut targets = vec![target];
            targets.extend(deps.into_iter().map(|(p, _)| p).collect_vec());
            (targets, vec![])
        } else {
            (vec![target], deps)
        };
        let (all_targets, all_deps) = match &self.model_config.target_filter {
            Some(filter) => {
                let mut new_targets = vec![];
                let mut new_deps = all_deps.into_iter().map(|(p, _)| p).collect_vec();
                for PackagePaths {
                    name,
                    paths,
                    named_address_map,
                } in all_targets
                {
                    let (true_targets, false_targets): (Vec<_>, Vec<_>) =
                        paths.into_iter().partition(|t| t.contains(filter));
                    if !true_targets.is_empty() {
                        new_targets.push(PackagePaths {
                            name,
                            paths: true_targets,
                            named_address_map: named_address_map.clone(),
                        })
                    }
                    if !false_targets.is_empty() {
                        new_deps.push(PackagePaths {
                            name,
                            paths: false_targets,
                            named_address_map,
                        })
                    }
                }
                (new_targets, new_deps)
            }
            None => (
                all_targets,
                all_deps.into_iter().map(|(p, _)| p).collect_vec(),
            ),
        };

        run_model_builder_with_options(all_targets, all_deps, ModelBuilderOptions::default())
    }
}

use move_symbol_pool::{Symbol as MoveSymbol, Symbol};

/// Build the move model with default compilation flags and custom options and a set of provided
/// named addreses.
/// This collects transitive dependencies for move sources from the provided directory list.
fn run_model_builder_with_options<
    Paths: Into<MoveSymbol> + Clone,
    NamedAddress: Into<MoveSymbol> + Clone,
>(
    move_sources: Vec<PackagePaths<Paths, NamedAddress>>,
    deps: Vec<PackagePaths<Paths, NamedAddress>>,
    options: ModelBuilderOptions,
) -> anyhow::Result<GlobalEnv> {
    let flag = Flags::verification().set_keep_testing_functions(true);
    run_model_builder_with_options_and_compilation_flags(move_sources, deps, options, flag)
}

fn make_source_and_deps_for_compiler(
    resolution_graph: &ResolvedGraph,
    root: &ResolvedPackage,
    deps: Vec<(
        /* name */ Symbol,
        /* source paths */ Vec<Symbol>,
        /* address mapping */ &ResolvedTable,
        /* whether src is available */ bool,
    )>,
) -> anyhow::Result<(
    /* sources */ PackagePaths,
    /* deps */ Vec<(PackagePaths, bool)>,
)> {
    let deps_package_paths = deps
        .into_iter()
        .map(|(name, source_paths, resolved_table, src_flag)| {
            let paths = source_paths
                .into_iter()
                .collect::<BTreeSet<_>>()
                .into_iter()
                .collect::<Vec<_>>();
            let named_address_map = named_address_mapping_for_compiler(resolved_table);
            Ok((
                PackagePaths {
                    name: Some(name),
                    paths,
                    named_address_map,
                },
                src_flag,
            ))
        })
        .collect::<anyhow::Result<Vec<_>>>()?;
    let root_named_addrs = apply_named_address_renaming(
        root.source_package.package.name,
        named_address_mapping_for_compiler(&root.resolution_table),
        &root.renaming,
    );
    let sources = root.get_sources(&resolution_graph.build_options)?;
    let source_package_paths = PackagePaths {
        name: Some(root.source_package.package.name),
        paths: sources,
        named_address_map: root_named_addrs,
    };
    Ok((source_package_paths, deps_package_paths))
}

fn named_address_mapping_for_compiler(
    resolution_table: &ResolvedTable,
) -> BTreeMap<Symbol, NumericalAddress> {
    resolution_table
        .iter()
        .map(|(ident, addr)| {
            let parsed_addr =
                NumericalAddress::new(addr.into_bytes(), move_compiler::shared::NumberFormat::Hex);
            (*ident, parsed_addr)
        })
        .collect::<BTreeMap<_, _>>()
}

fn apply_named_address_renaming(
    current_package_name: Symbol,
    address_resolution: BTreeMap<Symbol, NumericalAddress>,
    renaming: &Renaming,
) -> NamedAddressMap {
    let package_renamings = renaming
        .iter()
        .filter_map(|(rename_to, (package_name, from_name))| {
            if package_name == &current_package_name {
                Some((from_name, *rename_to))
            } else {
                None
            }
        })
        .collect::<BTreeMap<_, _>>();

    address_resolution
        .into_iter()
        .map(|(name, value)| {
            let new_name = package_renamings.get(&name).copied();
            (new_name.unwrap_or(name), value)
        })
        .collect()
}

pub const BYTECODE_VERSION: u32 = 6;

/// The keys used to identify the metadata in the metadata section of the module bytecode.
/// This is more or less arbitrary, besides we should use some unique key to identify
/// Rooch specific metadata (`rooch::` here).
pub static ROOCH_METADATA_KEY: &[u8] = "rooch::metadata_v0".as_bytes();

pub fn build_model(
    package_path: &Path,
    additional_named_addresses: BTreeMap<String, AccountAddress>,
    dev_mode: bool,
    target_filter: Option<String>,
) -> anyhow::Result<GlobalEnv> {
    let build_config = BuildConfig {
        dev_mode,
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
    };
    build_config.move_model_for_package(
        package_path,
        ModelConfig {
            target_filter,
            all_files_as_targets: true,
        },
    )
}

pub fn build_model_with_test_attr(
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
    };
    let resolved_graph =
        build_config.resolution_graph_for_package(package_path, &mut std::io::stdout())?;
    let model_config = ModelConfig {
        target_filter,
        all_files_as_targets: true,
    };
    ModelBuilder::create(resolved_graph, model_config).build_model()
}

pub fn run_verifier<P: AsRef<Path>>(
    package_path: P,
    build_config: BuildConfig,
    package: &mut CompiledPackage,
) -> anyhow::Result<bool> {
    let model = build_model(
        package_path.as_ref(),
        build_config.additional_named_addresses,
        build_config.dev_mode,
        None,
    )
    .unwrap();

    let runtime_metadata = run_extended_checks(&model);

    if model.diag_count(Severity::Warning) > 0 {
        let mut error_writer = StandardStream::stderr(ColorChoice::Auto);
        model.report_diag(&mut error_writer, Severity::Warning);
        if model.has_errors() {
            return Err(anyhow::Error::msg("extended checks failed"));
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

    Ok(true)
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
                        log::debug!(
                            "\n\nstart dump data structs map {:?}",
                            named_module.module.self_id().to_string()
                        );
                        for (k, v) in module_metadata.data_struct_map.iter() {
                            log::debug!("{:?} -> {:?}", k, v);
                        }
                        log::debug!("\n");
                        for (k, v) in module_metadata.data_struct_func_map.iter() {
                            log::debug!("{:?} -> {:?}", k, v);
                        }
                        log::debug!(
                            "start dump data structs map {:?}\n\n",
                            named_module.module.self_id().to_string()
                        );

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

pub fn compile_and_inject_metadata(
    compiled_module: &CompiledModule,
    ast_metadata: ASTMetadata,
) -> CompiledModule {
    let mut module = compiled_module.clone();

    let mut rooch_metadata = RuntimeModuleMetadataV1::default();
    for (metadata_type, metadata_item) in ast_metadata.value {
        if metadata_type == "private_generics" {
            let mut private_generics_map: BTreeMap<String, Vec<usize>> = BTreeMap::new();
            for (metadata_key, metadata_value) in metadata_item.iter() {
                let mut generic_type_indices: Vec<usize> = Vec::new();
                for idx_expr in metadata_value.iter() {
                    let expr_value = idx_expr.value.clone();
                    if let Exp_::Value(copyable_val) = expr_value {
                        if let CopyableVal_::U64(u64_value) = copyable_val.value {
                            generic_type_indices.push(u64_value as usize);
                        }
                    }
                }
                private_generics_map.insert(metadata_key.clone(), generic_type_indices);
            }

            rooch_metadata.private_generics_indices = private_generics_map;
        }

        if metadata_type == "data_struct" {
            let mut data_structs_map: BTreeMap<String, bool> = BTreeMap::new();
            for (metadata_key, metadata_value) in metadata_item.iter() {
                for idx_expr in metadata_value.iter() {
                    let expr_value = idx_expr.value.clone();
                    if let Exp_::Value(copyable_val) = expr_value {
                        if let CopyableVal_::Bool(bool_value) = copyable_val.value {
                            data_structs_map.insert(metadata_key.clone(), bool_value);
                        }
                    }
                }
            }

            rooch_metadata.data_struct_map = data_structs_map;
        }
    }

    let serialized_metadata =
        bcs::to_bytes(&rooch_metadata).expect("BCS for RuntimeModuleMetadata");
    module.metadata.push(Metadata {
        key: ROOCH_METADATA_KEY.to_vec(),
        value: serialized_metadata,
    });

    module
}
