// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::CONFIG_PREFIX_NAME;
use anyhow::Result;
use moveos_types::startup_info::StartupInfo;
use raw_store::{derive_store, CodecKVStore};
use std::string::ToString;

pub const STARTUP_INFO_KEY: &str = "startup_info";

derive_store!(StartupInfoDBStore, String, StartupInfo, CONFIG_PREFIX_NAME);

pub trait ConfigStore {
    fn get_startup_info(&self) -> Result<Option<StartupInfo>>;

    fn save_startup_info(&self, startup_info: StartupInfo) -> Result<()>;
}

impl ConfigStore for StartupInfoDBStore {
    fn get_startup_info(&self) -> Result<Option<StartupInfo>> {
        self.kv_get(STARTUP_INFO_KEY.to_string())
    }

    fn save_startup_info(&self, startup_info: StartupInfo) -> Result<()> {
        self.put_sync(STARTUP_INFO_KEY.to_string(), startup_info)
    }
}
