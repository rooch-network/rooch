// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use clap::Parser;
use itertools::Itertools;
use move_binary_format::{
    compatibility::Compatibility, errors::PartialVMResult, normalized::Module, CompiledModule,
};
use moveos_stdlib_builder::Stdlib;
use rooch_genesis_builder::build_stdlib;
use rooch_types::stdlib_version::StdlibVersion;
use std::collections::HashMap;
use std::path::Path;
use std::path::PathBuf;

#[derive(Parser)]
#[clap(
    name = "rooch-framework-release",
    author = "The Rooch Core Contributors"
)]
struct StdlibOpts {
    /// Version number for compiled stdlib, starting from 1 and increasing continuously.
    #[clap(short = 'v', long, value_name = "VERSION")]
    version: Option<u64>,

    /// don't check compatibility between the old and new standard library
    #[clap(short = 'n', long)]
    no_check_compatibility: bool,
}

fn main() {
    let opts: StdlibOpts = StdlibOpts::parse();

    let version = StdlibVersion::new(opts.version.unwrap_or(0));
    let pre_version = if let Some(version_num) = opts.version {
        assert!(
            version_num > 0,
            "The version number must start from 1 and increase continuously"
        );
        if version_num > 1 {
            Some(StdlibVersion::new(version_num - 1))
        } else {
            None
        }
    } else {
        // Read dirname in compiled dir, to get the max version number
        let max_version = current_max_version();
        if max_version > 0 {
            Some(StdlibVersion::new(max_version))
        } else {
            None
        }
    };

    let curr_stdlib = build_stdlib().unwrap();
    // check compatibility
    if let Some(pre_version) = pre_version {
        if !opts.no_check_compatibility {
            let prev_stdlib = Stdlib::load_from_file(stdlib_output_file(&pre_version.as_string()))
                .expect(&format!(
                    "load previous stdlib (version {:}) failed",
                    pre_version.as_string()
                ));
            assert_stdlib_compatibility(&curr_stdlib, &prev_stdlib);
        }
    }

    // Only save the stdlib with given version number
    if version != StdlibVersion::Latest {
        curr_stdlib
            .save_to_file(stdlib_output_file(&version.as_string()))
            .unwrap();
    }
}

/// Check whether the new stdlib is compatible with the old stdlib
fn assert_stdlib_compatibility(curr_stdlib: &Stdlib, prev_stdlib: &Stdlib) {
    let new_modules_map = curr_stdlib
        .all_modules()
        .expect("Extract modules from new stdlib failed")
        .into_iter()
        .map(|module| (module.self_id(), module))
        .collect::<HashMap<_, _>>();
    let old_modules_map = prev_stdlib
        .all_modules()
        .expect("Extract modules from old stdlib failed")
        .into_iter()
        .map(|module| (module.self_id(), module))
        .collect::<HashMap<_, _>>();

    let incompatible_module_ids = new_modules_map
        .values()
        .into_iter()
        .filter_map(|module| {
            let module_id = module.self_id();
            if let Some(old_module) = old_modules_map.get(&module_id) {
                let compatibility = check_compiled_module_compat(old_module, module);
                if compatibility.is_err() {
                    Some(module_id)
                } else {
                    None
                }
            } else {
                println!("Module {:?} is new module.", module_id);
                None
            }
        })
        .collect::<Vec<_>>();

    if !incompatible_module_ids.is_empty() {
        eprintln!(
            "Modules {} is incompatible with previous version!",
            incompatible_module_ids
                .into_iter()
                .map(|module_id| module_id.to_string())
                .join(","),
        );
        std::process::exit(1);
    }

    let deleted_module_ids = old_modules_map
        .keys()
        .into_iter()
        .filter_map(|module_id| {
            if !new_modules_map.contains_key(module_id) {
                Some(module_id.clone())
            } else {
                None
            }
        })
        .collect::<Vec<_>>();

    if !deleted_module_ids.is_empty() {
        eprintln!(
            "Modules {} is deleted!",
            deleted_module_ids
                .into_iter()
                .map(|module_id| module_id.to_string())
                .join(","),
        );
        std::process::exit(1);
    }
}

/// check module compatibility
fn check_compiled_module_compat(
    new_module: &CompiledModule,
    old_module: &CompiledModule,
) -> PartialVMResult<()> {
    let new_m = Module::new(new_module);
    let old_m = Module::new(old_module);
    // TODO: config compatibility through global configuration
    let compat = Compatibility::full_check();
    compat.check(&old_m, &new_m)
}

/// Read max version number except `latest` from stdlib release dir
fn current_max_version() -> u64 {
    let mut max_version = 0;
    for entry in release_dir().read_dir().unwrap() {
        let entry = entry.unwrap();
        let dirname = entry.file_name();
        if let Some(dirname_str) = dirname.to_str() {
            if let Ok(version) = dirname_str.parse::<u64>() {
                if version > max_version {
                    max_version = version;
                }
            }
        }
    }
    max_version
}

fn stdlib_output_file(version_str: &str) -> PathBuf {
    let version_dir = release_dir().join(version_str);
    if !version_dir.exists() {
        std::fs::create_dir_all(&version_dir)
            .expect(&format!("Create dir {:?} failed", version_dir));
    }
    version_dir.join("stdlib")
}

fn release_dir() -> PathBuf {
    path_in_crate(format!("../rooch-framework-release/compiled/",))
}

fn path_in_crate<S>(relative: S) -> PathBuf
where
    S: AsRef<Path>,
{
    let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    path.push(relative);
    path
}
