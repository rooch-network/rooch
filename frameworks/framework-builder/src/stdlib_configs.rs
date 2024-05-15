// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::{path_in_crate, Stdlib, StdlibBuildConfig};
use anyhow::Result;
use move_package::BuildConfig;
use once_cell::sync::Lazy;

static STDLIB_BUILD_CONFIGS: Lazy<Vec<StdlibBuildConfig>> = Lazy::new(|| {
    let move_stdlib_path = path_in_crate("../move-stdlib")
        .canonicalize()
        .expect("canonicalize path failed");
    let moveos_stdlib_path = path_in_crate("../moveos-stdlib")
        .canonicalize()
        .expect("canonicalize path failed");
    let rooch_framework_path = path_in_crate("../rooch-framework")
        .canonicalize()
        .expect("canonicalize path failed");

    let bitcoin_move_path = path_in_crate("../bitcoin-move")
        .canonicalize()
        .expect("canonicalize path failed");

    let rooch_nursery_path = path_in_crate("../rooch-nursery")
        .canonicalize()
        .expect("canonicalize path failed");

    vec![
        StdlibBuildConfig {
            path: move_stdlib_path.clone(),
            error_prefix: "E".to_string(),
            error_code_map_output_file: move_stdlib_path.join("error_description.errmap"),
            document_template: move_stdlib_path.join("doc_template/README.md"),
            document_output_directory: move_stdlib_path.join("doc"),
            build_config: BuildConfig::default(),
            stable: true,
        },
        StdlibBuildConfig {
            path: moveos_stdlib_path.clone(),
            error_prefix: "Error".to_string(),
            error_code_map_output_file: moveos_stdlib_path.join("error_description.errmap"),
            document_template: moveos_stdlib_path.join("doc_template/README.md"),
            document_output_directory: moveos_stdlib_path.join("doc"),
            build_config: BuildConfig::default(),
            stable: true,
        },
        StdlibBuildConfig {
            path: rooch_framework_path.clone(),
            error_prefix: "Error".to_string(),
            error_code_map_output_file: rooch_framework_path.join("error_description.errmap"),
            document_template: rooch_framework_path.join("doc_template/README.md"),
            document_output_directory: rooch_framework_path.join("doc"),
            build_config: BuildConfig::default(),
            stable: true,
        },
        StdlibBuildConfig {
            path: bitcoin_move_path.clone(),
            error_prefix: "Error".to_string(),
            error_code_map_output_file: bitcoin_move_path.join("error_description.errmap"),
            document_template: bitcoin_move_path.join("doc_template/README.md"),
            document_output_directory: bitcoin_move_path.join("doc"),
            build_config: BuildConfig::default(),
            stable: true,
        },
        StdlibBuildConfig {
            path: rooch_nursery_path.clone(),
            error_prefix: "Error".to_string(),
            error_code_map_output_file: rooch_nursery_path.join("error_description.errmap"),
            document_template: rooch_nursery_path.join("doc_template/README.md"),
            document_output_directory: rooch_nursery_path.join("doc"),
            build_config: BuildConfig::default(),
            stable: false,
        },
    ]
});

pub fn build_stdlib(stable: bool) -> Result<Stdlib> {
    let configs = if stable {
        STDLIB_BUILD_CONFIGS
            .iter()
            .filter(|config| config.stable)
            .cloned()
            .collect()
    } else {
        STDLIB_BUILD_CONFIGS.clone()
    };
    Stdlib::build(configs)
}
