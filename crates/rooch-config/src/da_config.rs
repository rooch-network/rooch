// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use std::str::FromStr;

use clap::Parser;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum DAServerType {
    Celestia(DAServerCelestiaConfig),
}

#[derive(Clone, Default, Debug, Deserialize, PartialEq, Serialize, Parser)]
#[serde(deny_unknown_fields)]
pub struct DAConfig {
    #[serde(skip_serializing_if = "Option::is_none")]
    #[clap(name = "da-server", long, help = "specifies the type of DA server to be used. 'celestia' with corresponding Celestia server configuration, 'xxx' with corresponding xxx server configuration, etc.")]
    pub da_server: Option<DAServerType>,
}

#[derive(Clone, Default, Debug, Deserialize, PartialEq, Serialize, Parser)]
#[serde(deny_unknown_fields)]
pub struct DAServerCelestiaConfig {
    #[serde(skip_serializing_if = "Option::is_none")]
    #[clap(name = "namespace", long, env = "DA_CELESTIA_NAMESPACE", help = "celestia namespace")]
    pub namespace: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[clap(name = "conn", long, env = "DA_CELESTIA_CONN", help = "celestia node connection")]
    pub conn: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[clap(name = "auth-token", long, env = "DA_CELESTIA_AUTH_TOKEN", help = "celestia node auth token")]
    pub auth_token: Option<String>,
    // for celestia:
    // support for up to 8 MB blocks, starting with 2MB at genesis and upgradeable through onchain governance.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[clap(name = "max-segment-size", long, env = "DA_CELESTIA_MAX_SEGMENT_SIZE", help = "max segment size, striking a balance between throughput and the constraints on blob size.")]
    pub max_segment_size: Option<u64>,
}

impl Default for DAServerCelestiaConfig {
    fn default() -> Self {
        Self {
            namespace: None,
            conn: None,
            auth_token: None,
            max_segment_size: Some(1 * 1024 * 1024),
        }
    }
}

impl FromStr for DAServerType {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let deserialized = serde_json::from_str(s)?;
        Ok(deserialized)
    }
}
