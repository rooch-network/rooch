// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use std::collections::HashMap;
use std::str::FromStr;
use std::sync::Arc;

use clap::Parser;
use serde::{Deserialize, Serialize};

use crate::{BaseConfig, ConfigModule, RoochOpt};

#[derive(Clone, Default, Debug, PartialEq, Deserialize, Serialize, Parser)]
#[serde(deny_unknown_fields)]
pub struct DAConfig {
    #[serde(skip_serializing_if = "Option::is_none")]
    #[clap(
    name = "internal-da-server",
    long,
    help = "internal da server config"
    )]
    pub internal_da_server: Option<InternalDAServerConfig>,
    // TODO external da server config
    // TODO internal external policy
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum DAServerSubmitStrategy {
    All,
    // >= n/2+1
    Quorum,
    // >= number
    Number(usize),
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum InternalDAServerConfigType {
    Celestia(DAServerCelestiaConfig),
    OpenDA(DAServerOpenDAConfig),
}

#[derive(Clone, Default, Debug, PartialEq, Deserialize, Serialize, Parser)]
#[serde(deny_unknown_fields)]
pub struct InternalDAServerConfig {
    #[clap(
    name = "submit-strategy",
    long,
    default_value = "all",
    help = "specifies the strategy of submitting transactions to da servers.\
    'all' means all da servers must submit the transaction, 'quorum' means at least n/2+1 da servers must submit the transaction, 'number' means at least n da servers must submit the transaction."
    )]
    pub submit_strategy: Option<DAServerSubmitStrategy>,
    #[clap(
    name = "servers",
    long,
    help = "specifies the type of internal DA servers to be used. 'celestia' with corresponding Celestia server configuration, 'xxx' with corresponding xxx server configuration, etc."
    )]
    pub servers: Vec<InternalDAServerConfigType>,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum DAServerOpenDAScheme {
    // gcs(Google Could Service) config:
    // bucket
    // root
    // credential
    // predefined_acl
    // default_storage_class
    GCS,
    // s3 config:
    // root
    // bucket
    // region
    // endpoint
    // access_key_id
    // secret_access_key
    S3,
}

// Open DA provides ability to access various storage services
#[derive(Clone, Default, Debug, PartialEq, Deserialize, Serialize, Parser)]
#[serde(deny_unknown_fields)]
pub struct DAServerOpenDAConfig {
    #[clap(
    name = "scheme",
    long,
    help = "specifies the type of storage service to be used. 'gcs' with corresponding GCS server configuration, 's3' with corresponding S3 server configuration, etc."
    )]
    pub scheme: DAServerOpenDAScheme,
    #[clap(
    name = "config",
    long,
    help = "specifies the configuration of the storage service. 'gcs' with corresponding GCS server configuration, 's3' with corresponding S3 server configuration, etc."
    )]
    pub config: HashMap<String, String>,
}

#[derive(Clone, Default, Debug, PartialEq, Deserialize, Serialize, Parser)]
#[serde(deny_unknown_fields)]
pub struct DAServerCelestiaConfig {
    #[serde(skip_serializing_if = "Option::is_none")]
    #[clap(
    name = "namespace",
    long,
    env = "DA_CELESTIA_NAMESPACE",
    help = "celestia namespace"
    )]
    pub namespace: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[clap(
    name = "conn",
    long,
    env = "DA_CELESTIA_CONN",
    help = "celestia node connection"
    )]
    pub conn: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[clap(
    name = "auth-token",
    long,
    env = "DA_CELESTIA_AUTH_TOKEN",
    help = "celestia node auth token"
    )]
    pub auth_token: Option<String>,
    // for celestia:
    // support for up to 8 MB blocks, starting with 2MB at genesis and upgradeable through onchain governance.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[clap(
    name = "max-segment-size",
    long,
    env = "DA_CELESTIA_MAX_SEGMENT_SIZE",
    help = "max segment size, striking a balance between throughput and the constraints on blob size."
    )]
    pub max_segment_size: Option<u64>,
}

impl Default for DAServerCelestiaConfig {
    fn default() -> Self {
        Self {
            namespace: None,
            conn: None,
            auth_token: None,
            max_segment_size: Some(1024 * 1024),
        }
    }
}

impl DAServerCelestiaConfig {
    pub fn new_with_defaults(mut self) -> Self {
        let default = DAServerCelestiaConfig::default();
        if self.max_segment_size.is_none() {
            self.max_segment_size = default.max_segment_size;
        }
        self
    }
}

impl FromStr for DAConfig {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let deserialized = serde_json::from_str(s)?;
        Ok(deserialized)
    }
}

impl FromStr for InternalDAServerConfig {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let deserialized = serde_json::from_str(s)?;
        Ok(deserialized)
    }
}

impl ConfigModule for DAConfig {
    fn merge_with_opt(&mut self, opt: &RoochOpt, _base: Arc<BaseConfig>) -> anyhow::Result<()> {
        Ok(())
    }
}

