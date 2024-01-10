// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use std::collections::HashMap;
use std::error::Error;
use std::str::FromStr;

use clap::Parser;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::config::Config;
use crate::RoochOpt;

#[derive(Clone, Default, Debug, PartialEq, Deserialize, Serialize, Parser)]
#[serde(deny_unknown_fields)]
pub struct DAConfig {
    #[serde(skip_serializing_if = "Option::is_none")]
    #[clap(name = "internal-da-server", long, help = "internal da server config")]
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

impl FromStr for DAServerSubmitStrategy {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "all" => Ok(DAServerSubmitStrategy::All),
            "quorum" => Ok(DAServerSubmitStrategy::Quorum),
            _ => {
                if let Ok(n) = s.parse::<usize>() {
                    Ok(DAServerSubmitStrategy::Number(n))
                } else {
                    Err(format!("Invalid value: {}", s))
                }
            }
        }
    }
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum InternalDAServerConfigType {
    Celestia(DAServerCelestiaConfig),
    OpenDA(DAServerOpenDAConfig),
}

impl FromStr for InternalDAServerConfigType {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let v: Value = serde_json::from_str(s).map_err(|_| format!("Invalid JSON: {}", s))?;

        if let Some(obj) = v.as_object() {
            if let Some(celestia) = obj.get("celestia") {
                let celestia_config: DAServerCelestiaConfig =
                    serde_json::from_value(celestia.clone())
                        .map_err(|_| format!("Invalid Celestia config: {}", celestia))?;
                Ok(InternalDAServerConfigType::Celestia(celestia_config))
            } else if let Some(openda) = obj.get("openda") {
                let openda_config: DAServerOpenDAConfig = serde_json::from_value(openda.clone())
                    .map_err(|_| format!("Invalid OpenDA config: {}", openda))?;
                Ok(InternalDAServerConfigType::OpenDA(openda_config))
            } else {
                Err(format!("Invalid value: {}", s))
            }
        } else {
            Err(format!("Invalid value: {}", s))
        }
    }
}

#[derive(Clone, Default, Debug, PartialEq, Deserialize, Serialize, Parser)]
#[serde(deny_unknown_fields)]
pub struct InternalDAServerConfig {
    #[serde(skip_serializing_if = "Option::is_none")]
    #[clap(
        name = "submit-strategy",
        long,
        help = "specifies the type of internal DA servers to be used. 'celestia' with corresponding Celestia server configuration, 'xxx' with corresponding xxx server configuration, etc."
    )]
    pub submit_strategy: Option<DAServerSubmitStrategy>,
    #[clap(
        name = "servers",
        long,
        help = "specifies the type of internal DA servers to be used. 'celestia' with corresponding Celestia server configuration, 'xxx' with corresponding xxx server configuration, etc."
    )]
    pub servers: Vec<InternalDAServerConfigType>,
}

#[derive(Clone, Debug, Default, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum OpenDAScheme {
    // gcs(Google Could Service) config:
    // bucket
    // root
    // credential
    // predefined_acl
    // default_storage_class
    #[default]
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

impl FromStr for OpenDAScheme {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "gcs" => Ok(OpenDAScheme::GCS),
            "s3" => Ok(OpenDAScheme::S3),
            _ => Err("open-da no match"),
        }
    }
}

fn parse_hashmap(
    s: &str,
) -> Result<HashMap<String, String>, Box<dyn Error + Send + Sync + 'static>> {
    s.split(',')
        .map(|kv| {
            let mut parts = kv.splitn(2, '=');
            match (parts.next(), parts.next()) {
                (Some(key), Some(value)) => Ok((key.to_string(), value.to_string())),
                _ => Err("Each key=value pair must be separated by a comma".into()),
            }
        })
        .collect()
}

// Open DA provides ability to access various storage services
#[derive(Clone, Default, Debug, PartialEq, Deserialize, Serialize, Parser)]
#[serde(deny_unknown_fields)]
pub struct DAServerOpenDAConfig {
    #[clap(
        name = "scheme",
        long,
        value_enum,
        default_value = "gcs",
        help = "specifies the type of storage service to be used. 'gcs' with corresponding GCS server configuration, 's3' with corresponding S3 server configuration, etc."
    )]
    pub scheme: OpenDAScheme,
    #[clap(
    name = "config",
    long,
    value_parser = parse_hashmap,
    help = "specifies the configuration of the storage service. 'gcs' with corresponding GCS server configuration, 's3' with corresponding S3 server configuration, etc."
    )]
    pub config: HashMap<String, String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    #[clap(
        name = "max-segment-size",
        long,
        help = "max segment size, striking a balance between throughput and the constraints on blob size."
    )]
    pub max_segment_size: Option<u64>,
}

#[derive(Clone, Debug, PartialEq, Deserialize, Serialize, Parser)]
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

impl Config for DAConfig {}

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

impl DAConfig {
    pub fn merge_with_opt(&mut self, opt: &RoochOpt) -> anyhow::Result<()> {
        if let Some(ref da_config) = opt.da {
            // TODO merge with field checking
            *self = da_config.clone();
        }
        Ok(())
    }
}
