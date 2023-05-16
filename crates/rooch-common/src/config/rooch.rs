// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use rooch_key::keystore::{AccountKeystore, Keystore};
use rooch_types::address::RoochAddress;
use serde::Deserialize;
use serde::Serialize;
use serde_with::serde_as;
use std::fmt::{Display, Formatter, Result, Write};

use crate::config::server::ServerConfig;
use crate::config::Config;

use std::{fs, path::PathBuf};

const ROOCH_DIR: &str = ".rooch";
pub const ROOCH_CONFIG_DIR: &str = "rooch_config";
pub const ROOCH_CONFIG: &str = "rooch.yaml";
pub const ROOCH_KEYSTORE_FILENAME: &str = "rooch.keystore";
pub const AUTHORITIES_DB_NAME: &str = "authorities_db";

pub fn rooch_config_dir() -> anyhow::Result<PathBuf, anyhow::Error> {
    match std::env::var_os("ROOCH_CONFIG_DIR") {
        Some(config_env) => Ok(config_env.into()),
        None => match dirs::home_dir() {
            Some(v) => Ok(v.join(ROOCH_DIR).join(ROOCH_CONFIG_DIR)),
            None => anyhow::bail!("Cannot obtain home directory path"),
        },
    }
    .and_then(|dir| {
        if !dir.exists() {
            fs::create_dir_all(dir.clone())?;
        }
        Ok(dir)
    })
}

pub fn rooch_config_path() -> anyhow::Result<PathBuf, anyhow::Error> {
    Ok(rooch_config_dir()?.join(ROOCH_CONFIG))
}

#[serde_as]
#[derive(Serialize, Deserialize)]
pub struct RoochConfig {
    pub keystore: Keystore,
    pub active_address: Option<RoochAddress>,
    pub server: Option<ServerConfig>,
}

impl RoochConfig {
    pub fn new(keystore: Keystore) -> Self {
        RoochConfig {
            keystore,
            active_address: None,
            server: None,
        }
    }
}

impl Config for RoochConfig {}

impl Display for RoochConfig {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        let mut writer = String::new();

        writeln!(
            writer,
            "Managed addresses : {}",
            self.keystore.addresses().len()
        )?;
        write!(writer, "Active address: ")?;
        match self.active_address {
            Some(r) => writeln!(writer, "{}", r)?,
            None => writeln!(writer, "None")?,
        };
        writeln!(writer, "{}", self.keystore)?;
        write!(writer, "server: ")?;
        match &self.server {
            Some(r) => writeln!(writer, "{}", r)?,
            None => writeln!(writer, "None")?,
        };
        write!(f, "{}", writer)
    }
}
