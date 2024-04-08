// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use std::path::{Path, PathBuf};

use anyhow::Result;
use framework_builder::{Stdlib, StdlibBuildConfig};
use move_package::BuildConfig;
use once_cell::sync::Lazy;

static STDLIB_BUILD_CONFIGS: Lazy<Vec<StdlibBuildConfig>> = Lazy::new(|| {
    let move_stdlib_path = path_in_crate("../../frameworks/move-stdlib")
        .canonicalize()
        .expect("canonicalize path failed");
    let moveos_stdlib_path = path_in_crate("../../frameworks/moveos-stdlib")
        .canonicalize()
        .expect("canonicalize path failed");
    let rooch_framework_path = path_in_crate("../../frameworks/rooch-framework")
        .canonicalize()
        .expect("canonicalize path failed");

    let bitcoin_move_path = path_in_crate("../../frameworks/bitcoin-move")
        .canonicalize()
        .expect("canonicalize path failed");
    let generated_dir = generated_dir();

    vec![
        StdlibBuildConfig {
            path: move_stdlib_path.clone(),
            error_prefix: "E".to_string(),
            error_code_map_output_file: generated_dir.join("move_std_error_description.errmap"),
            document_template: move_stdlib_path.join("doc_template/README.md"),
            document_output_directory: move_stdlib_path.join("doc"),
            build_config: BuildConfig::default(),
        },
        StdlibBuildConfig {
            path: moveos_stdlib_path.clone(),
            error_prefix: "Error".to_string(),
            error_code_map_output_file: generated_dir.join("moveos_std_error_description.errmap"),
            document_template: moveos_stdlib_path.join("doc_template/README.md"),
            document_output_directory: moveos_stdlib_path.join("doc"),
            build_config: BuildConfig::default(),
        },
        StdlibBuildConfig {
            path: rooch_framework_path.clone(),
            error_prefix: "Error".to_string(),
            error_code_map_output_file: generated_dir
                .join("rooch_framework_error_description.errmap"),
            document_template: rooch_framework_path.join("doc_template/README.md"),
            document_output_directory: rooch_framework_path.join("doc"),
            build_config: BuildConfig::default(),
        },
        StdlibBuildConfig {
            path: bitcoin_move_path.clone(),
            error_prefix: "Error".to_string(),
            error_code_map_output_file: generated_dir.join("bitcoin_move_error_description.errmap"),
            document_template: bitcoin_move_path.join("doc_template/README.md"),
            document_output_directory: bitcoin_move_path.join("doc"),
            build_config: BuildConfig::default(),
        },
    ]
});

pub fn build_stdlib() -> Result<Stdlib> {
    framework_builder::Stdlib::build(STDLIB_BUILD_CONFIGS.clone())
}

pub fn build_and_save_stdlib() -> Result<()> {
    std::fs::create_dir_all(generated_dir())?;
    let stdlib = build_stdlib()?;
    stdlib.save_to_file(stdlib_output_file())
}

pub fn stdlib_output_file() -> PathBuf {
    generated_dir().join("stdlib")
}

fn generated_dir() -> PathBuf {
    path_in_crate("../rooch-genesis/generated")
}

fn path_in_crate<S>(relative: S) -> PathBuf
where
    S: AsRef<Path>,
{
    let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    path.push(relative);
    path
}
