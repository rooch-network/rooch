// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use anyhow::{anyhow, Result};
use framework_builder::{stdlib_version::StdlibVersion, Stdlib};
use include_dir::{include_dir, Dir};

pub mod error_descriptions;

pub(crate) const STATIC_FRAMEWORK_DIR: Dir = include_dir!("released");

pub fn load_stdlib(version: StdlibVersion) -> Result<Stdlib> {
    STATIC_FRAMEWORK_DIR
        .get_file(version.dir_with_file())
        .ok_or_else(|| anyhow!("stdlib not found"))
        .and_then(|f| {
            Stdlib::decode(f.contents()).map_err(|e| {
                anyhow!(
                    "Load stdlib from static include file {:?} failed: {:?}",
                    f.path(),
                    e
                )
            })
        })
}
