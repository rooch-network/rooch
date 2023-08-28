// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::{CONFIG_GENESIS_PREFIX_NAME, CONFIG_STARTUP_INFO_PREFIX_NAME};
use anyhow::Result;
use moveos_types::genesis_info::GenesisInfo;
use moveos_types::startup_info::StartupInfo;
use raw_store::{derive_store, CodecKVStore, StoreInstance};
use std::string::ToString;

pub const STARTUP_INFO_KEY: &str = "startup_info";
pub const GENESIS_KEY: &str = "genesis";

derive_store!(
    StartupInfoStore,
    String,
    StartupInfo,
    CONFIG_STARTUP_INFO_PREFIX_NAME
);
derive_store!(
    GenesisStore,
    String,
    GenesisInfo,
    CONFIG_GENESIS_PREFIX_NAME
);

pub trait ConfigStore {
    fn get_startup_info(&self) -> Result<Option<StartupInfo>>;

    fn save_startup_info(&self, startup_info: StartupInfo) -> Result<()>;

    fn get_genesis(&self) -> Result<Option<GenesisInfo>>;

    fn save_genesis(&self, genesis_info: GenesisInfo) -> Result<()>;
}

#[derive(Clone)]
pub struct ConfigDBStore {
    startup_store: StartupInfoStore,
    genesis_store: GenesisStore,
}

impl ConfigDBStore {
    pub fn new(instance: StoreInstance) -> Self {
        ConfigDBStore {
            startup_store: StartupInfoStore::new(instance.clone()),
            genesis_store: GenesisStore::new(instance),
        }
    }

    pub fn get_startup_info(&self) -> Result<Option<StartupInfo>> {
        self.startup_store.kv_get(STARTUP_INFO_KEY.to_string())
    }

    pub fn save_startup_info(&self, startup_info: StartupInfo) -> Result<()> {
        self.startup_store
            .put_sync(STARTUP_INFO_KEY.to_string(), startup_info)
    }

    pub fn get_genesis(&self) -> Result<Option<GenesisInfo>> {
        self.genesis_store.kv_get(GENESIS_KEY.to_string())
    }

    pub fn save_genesis(&self, genesis_info: GenesisInfo) -> Result<()> {
        self.genesis_store
            .put_sync(GENESIS_KEY.to_string(), genesis_info)
    }
}
