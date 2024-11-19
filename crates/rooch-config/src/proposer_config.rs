// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::config::Config;
use clap::Parser;
use serde::{Deserialize, Serialize};
use std::str::FromStr;

#[derive(Clone, Default, Debug, Deserialize, PartialEq, Serialize, Parser)]
#[serde(deny_unknown_fields)]
pub struct ProposerConfig {
    #[serde(skip_serializing_if = "Option::is_none")]
    #[clap(
        name = "proposer-init-offset",
        long,
        help = "The initial block number offset of the proposer"
    )]
    pub init_offset: Option<u128>,
}

impl Config for ProposerConfig {}

impl std::fmt::Display for ProposerConfig {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            serde_json::to_string(self).map_err(|_e| std::fmt::Error)?
        )
    }
}

impl FromStr for ProposerConfig {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> anyhow::Result<Self, Self::Err> {
        let deserialized: ProposerConfig = serde_json::from_str(s)?;
        Ok(deserialized)
    }
}
