// Copyright (c) The Move Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::move_cli::types::AccountAddressWrapper;
use clap::*;
use move_cli::base::reroot_path;
use move_package::BuildConfig;
use std::{collections::BTreeMap, path::PathBuf};

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

        let rerooted_path = reroot_path(path)?;
        if config.fetch_deps_only {
            if config.test_mode {
                config.dev_mode = true;
            }
            config.download_deps_for_package(&rerooted_path, &mut std::io::stdout())?;
            return Ok(());
        }

        config.compile_package_no_exit(&rerooted_path, &mut std::io::stdout())?;

        Ok(())
    }
}
