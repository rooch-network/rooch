use move_core_types::account_address::AccountAddress;
use move_model::model::GlobalEnv;
use move_package::{BuildConfig, ModelConfig};
use std::collections::BTreeMap;
use std::path::Path;

pub(crate) const BYTECODE_VERSION: u32 = 6;

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
