// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

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
    #[serde(skip_serializing_if = "ServerType::is_none")]
    #[clap(name = "da-server", long, help = "specifies the type of DA server to be used. 'celestia' with corresponding Celestia server configuration, 'xxx' with corresponding xxx server configuration, etc.")]
    pub da_server: Option<DAServerType>,
}

#[derive(Clone, Default, Debug, Deserialize, PartialEq, Serialize, Parser)]
#[serde(deny_unknown_fields)]
pub struct DAServerCelestiaConfig {
    #[serde(skip_serializing_if = "Option::is_none")]
    #[clap(name = "celestia-namespace", long, help = "DA backend celestia namespace")]
    pub namespace: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[clap(name = "celestia-conn", long, help = "DA backend celestia node connection")]
    pub conn_str: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[clap(name = "celestia-auth-token", long, help = "DA backend celestia node auth token")]
    pub auth_token: Option<String>,
}