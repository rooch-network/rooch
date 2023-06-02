// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use anyhow::{Context, Result};
use serde::{de::DeserializeOwned, Serialize};
use std::{fmt::Debug, fs, path::Path, path::PathBuf};

pub const ROOCH_DIR: &str = ".rooch";
pub const ROOCH_CONFIG_DIR: &str = "rooch_config";
pub const ROOCH_CLIENT_CONFIG: &str = "rooch.yaml";
pub const ROOCH_SERVER_CONFIG: &str = "server.yaml";
pub const ROOCH_KEYSTORE_FILENAME: &str = "rooch.keystore";

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

pub trait Config
where
    Self: DeserializeOwned + Serialize,
{
    fn persisted(self, path: &Path) -> PersistedConfig<Self> {
        PersistedConfig {
            inner: self,
            path: path.to_path_buf(),
        }
    }

    fn load<P: AsRef<Path>>(path: P) -> Result<Self, anyhow::Error> {
        let path = path.as_ref();
        let reader = fs::File::open(path)
            .with_context(|| format!("Unable to load config from {}", path.display()))?;
        Ok(serde_yaml::from_reader(reader)?)
    }

    fn save<P: AsRef<Path>>(&self, path: P) -> Result<(), anyhow::Error> {
        let path = path.as_ref();
        let config = serde_yaml::to_string(&self)?;
        fs::write(path, config)
            .with_context(|| format!("Unable to save config to {}", path.display()))?;
        Ok(())
    }
}

#[derive(Debug)]
pub struct PersistedConfig<C> {
    inner: C,
    path: PathBuf,
}

impl<C> PersistedConfig<C>
where
    C: Config,
{
    pub fn read(path: &Path) -> Result<C, anyhow::Error> {
        Config::load(path)
    }

    pub fn save(&self) -> Result<(), anyhow::Error> {
        self.inner.save(&self.path)
    }

    pub fn into_inner(self) -> C {
        self.inner
    }

    pub fn path(&self) -> &Path {
        &self.path
    }
}

impl<C> std::ops::Deref for PersistedConfig<C> {
    type Target = C;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl<C> std::ops::DerefMut for PersistedConfig<C> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}
