// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

pub mod store_config;

use std::path::PathBuf;

pub const ROOCH_DIR: &str = ".rooch";
pub const ROOCH_CONFIG_DIR: &str = "rooch_config";
pub const ROOCH_CLIENT_CONFIG: &str = "rooch.yaml";
pub const ROOCH_SERVER_CONFIG: &str = "server.yaml";
pub const ROOCH_KEYSTORE_FILENAME: &str = "rooch.keystore";

pub fn get_rooch_config_dir() -> anyhow::Result<PathBuf, anyhow::Error> {
    match std::env::var_os("ROOCH_CONFIG_DIR") {
        Some(config_env) => Ok(config_env.into()),
        None => match dirs::home_dir() {
            Some(v) => Ok(v.join(ROOCH_DIR).join(ROOCH_CONFIG_DIR)),
            None => anyhow::bail!("Cannot obtain home directory path"),
        },
    }
}
