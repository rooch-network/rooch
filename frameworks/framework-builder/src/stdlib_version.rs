// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::{release_dir, Stdlib, STATIC_FRAMEWORK_DIR};
use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use std::cmp::Ordering;
use std::fmt::Formatter;
use std::fmt::{Debug, Display};
use std::path::PathBuf;
use std::str::FromStr;

#[derive(Clone, Copy, Default, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
pub enum StdlibVersion {
    #[default]
    Latest,
    Version(VersionNumber),
}

type VersionNumber = u64;

impl StdlibVersion {
    pub fn new(version: u64) -> Self {
        if version == 0 {
            StdlibVersion::Latest
        } else {
            StdlibVersion::Version(version)
        }
    }

    pub fn as_string(&self) -> String {
        match self {
            StdlibVersion::Latest => "latest".to_string(),
            StdlibVersion::Version(version) => format!("{}", version),
        }
    }

    pub fn version(&self) -> u64 {
        match self {
            StdlibVersion::Latest => 0,
            StdlibVersion::Version(version) => *version,
        }
    }

    pub fn is_latest(&self) -> bool {
        matches!(self, StdlibVersion::Latest)
    }

    /// If `version`` is compatible with previous version
    pub fn compatible_with_previous(_version: &StdlibVersion) -> bool {
        true
    }

    fn dir_with_file(&self) -> PathBuf {
        PathBuf::from(self.to_string()).join("stdlib")
    }

    pub fn output_file(&self) -> PathBuf {
        release_dir().join(self.dir_with_file())
    }

    pub(crate) fn load_from_file(&self) -> Result<Stdlib> {
        let file = self.output_file();
        Stdlib::load_from_file(file)
    }

    /// Load stdlib from static include file
    pub fn load(&self) -> Result<Stdlib> {
        STATIC_FRAMEWORK_DIR
            .get_file(self.dir_with_file())
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

    pub fn save(&self, stdlib: &Stdlib) -> Result<()> {
        let file = self.output_file();
        let parent = file
            .parent()
            .ok_or_else(|| anyhow!("Parent dir not found"))?;
        if !parent.exists() {
            std::fs::create_dir_all(parent)
                .map_err(|e| anyhow!("Create dir {:?} failed: {:?}", parent, e))?;
        }
        stdlib
            .save_to_file(&file)
            .map_err(|e| anyhow!("Save stdlib to {:?} failed: {:?}", file, e))
    }
}

impl PartialOrd for StdlibVersion {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for StdlibVersion {
    fn cmp(&self, other: &Self) -> Ordering {
        match (self, other) {
            (StdlibVersion::Latest, StdlibVersion::Latest) => Ordering::Equal,
            (StdlibVersion::Latest, _) => Ordering::Greater,
            (_, StdlibVersion::Latest) => Ordering::Less,
            (StdlibVersion::Version(self_v), StdlibVersion::Version(other_v)) => {
                self_v.cmp(other_v)
            }
        }
    }
}

impl FromStr for StdlibVersion {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "latest" => Ok(StdlibVersion::Latest),
            s => Ok(Self::new(s.parse()?)),
        }
    }
}

impl Display for StdlibVersion {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            StdlibVersion::Latest => f.write_str("latest"),
            StdlibVersion::Version(version) => f.write_str(version.to_string().as_str()),
        }
    }
}
