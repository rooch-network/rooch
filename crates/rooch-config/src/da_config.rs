// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::config::Config;
use crate::{retrieve_map_config_value, BaseConfig, MapConfigValueSource};
use hex::encode;
use moveos_types::h256::sha2_256_of;
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt::Display;
use std::path::{Path, PathBuf};
use std::str::FromStr;
use std::sync::Arc;

static R_DEFAULT_OPENDA_FS_DIR: Lazy<PathBuf> = Lazy::new(|| PathBuf::from("openda-fs"));

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum DAServerSubmitStrategy {
    // = n
    All,
    // >= n/2+1
    Quorum,
    // >= number, at least 1
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
                    Err(format!("invalid da server submit strategy: {}", s))
                }
            }
        }
    }
}

impl Display for DAServerSubmitStrategy {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DAServerSubmitStrategy::All => write!(f, "all"),
            DAServerSubmitStrategy::Quorum => write!(f, "quorum"),
            DAServerSubmitStrategy::Number(n) => write!(f, "{}", n),
        }
    }
}

#[derive(Clone, Debug, Default, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum OpenDAScheme {
    // local filesystem, main config:
    // root: file path
    #[default]
    Fs,
    // gcs(Google Could Service), main config:
    // bucket
    // credential/credential_path (using path instead)
    Gcs,
    // s3, main config:
    // bucket
    // region
    // endpoint
    // access_key_id
    // secret_access_key
    S3,
    // Avail App Light Client, main config:
    // endpoint
    Avail,
    // Celestia, main config:
    // endpoint
    // Option<auth_token>
    Celestia,
}

impl Display for OpenDAScheme {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            OpenDAScheme::Fs => write!(f, "fs"),
            OpenDAScheme::Gcs => write!(f, "gcs"),
            OpenDAScheme::S3 => write!(f, "s3"),
            OpenDAScheme::Avail => write!(f, "avail"),
            OpenDAScheme::Celestia => write!(f, "celestia"),
        }
    }
}

impl FromStr for OpenDAScheme {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "gcs" => Ok(OpenDAScheme::Gcs),
            "s3" => Ok(OpenDAScheme::S3),
            "fs" => Ok(OpenDAScheme::Fs),
            "avail" => Ok(OpenDAScheme::Avail),
            "celestia" => Ok(OpenDAScheme::Celestia),
            _ => Err("open-da scheme no match"),
        }
    }
}

// OpenDAScheme to OpenDALScheme
impl From<OpenDAScheme> for opendal::Scheme {
    fn from(scheme: OpenDAScheme) -> Self {
        match scheme {
            OpenDAScheme::Fs => opendal::Scheme::Fs,
            OpenDAScheme::Gcs => opendal::Scheme::Gcs,
            OpenDAScheme::S3 => opendal::Scheme::S3,
            OpenDAScheme::Avail => opendal::Scheme::Custom("avail"),
            OpenDAScheme::Celestia => opendal::Scheme::Custom("celestia"),
        }
    }
}

#[derive(Clone, Default, Debug, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
#[serde(rename_all = "kebab-case")]
pub struct DAConfig {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub da_backend: Option<DABackendConfig>,
    #[serde(skip_serializing_if = "Option::is_none")]
    /// The first block to be submitted.
    /// If not set, all blocks will be submitted.
    pub da_min_block_to_submit: Option<u128>,
    #[serde(skip)]
    base: Option<Arc<BaseConfig>>,
}

impl Display for DAConfig {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            serde_json::to_string(self).map_err(|_| std::fmt::Error)?
        )?;
        Ok(())
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

impl DAConfig {
    pub(crate) fn init(&mut self, base: Arc<BaseConfig>) -> anyhow::Result<()> {
        self.base = Some(base);

        let default_fs_root = self.get_openda_fs_dir();

        if let Some(da_backend_cfg) = &mut self.da_backend {
            let backends_configs = &mut da_backend_cfg.backends;
            for backend_config in backends_configs {
                #[allow(irrefutable_let_patterns)]
                if let DABackendConfigType::OpenDa(open_da_config) = backend_config {
                    if matches!(open_da_config.scheme, OpenDAScheme::Fs) {
                        if let Some(fs_str) = default_fs_root.to_str() {
                            let var_source = retrieve_map_config_value(
                                &mut open_da_config.config,
                                "root",
                                None,
                                Some(fs_str),
                            );
                            if let MapConfigValueSource::Default = var_source {
                                if !default_fs_root.exists() {
                                    std::fs::create_dir_all(default_fs_root.clone()).map_err(
                                        |e| {
                                            anyhow::anyhow!(
                                                "Failed to create OpenDA fs dir: {:?}",
                                                e
                                            )
                                        },
                                    )?;
                                }
                            }
                        } else {
                            return Err(anyhow::anyhow!(
                                "Invalid UTF-8 path: {:?}",
                                default_fs_root
                            ));
                        }
                    }
                }
            }
        }

        Ok(())
    }

    fn base(&self) -> &BaseConfig {
        self.base.as_ref().expect("Config should init.")
    }

    pub fn data_dir(&self) -> &Path {
        self.base().data_dir()
    }

    pub fn get_openda_fs_dir(&self) -> PathBuf {
        self.data_dir().join(R_DEFAULT_OPENDA_FS_DIR.as_path())
    }
}

#[derive(Clone, Default, Debug, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
#[serde(deny_unknown_fields)]
pub struct DABackendConfig {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub submit_strategy: Option<DAServerSubmitStrategy>, // specifies the submission strategy of DA. 'all' with all backends, 'quorum' with quorum backends, 'n' with n backends, etc.
    pub backends: Vec<DABackendConfigType>, // specifies the type of DA backends to be used. 'celestia' with corresponding Celestia backend configuration, 'foo' with corresponding foo backend configuration, etc.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub background_submit_interval: Option<u64>, // specifies the interval of background submit in seconds. If not set, the default value is 600s.
}

impl DABackendConfig {
    pub fn calculate_submit_threshold(&mut self) -> usize {
        self.adjust_submit_strategy(); // Make sure submit_strategy is adjusted before calling this function.

        let backends_count = self.backends.len();
        match self.submit_strategy {
            Some(DAServerSubmitStrategy::All) => backends_count,
            Some(DAServerSubmitStrategy::Quorum) => backends_count / 2 + 1,
            Some(DAServerSubmitStrategy::Number(number)) => number,
            None => backends_count, // Default to 'All' if submit_strategy is None
        }
    }

    fn adjust_submit_strategy(&mut self) {
        // Set default strategy to All if it's None.
        let strategy = self
            .submit_strategy
            .get_or_insert(DAServerSubmitStrategy::All);

        let backends_count = self.backends.len();

        // If it's Number, adjust the value to be within [1, n].
        if let DAServerSubmitStrategy::Number(ref mut num) = strategy {
            *num = std::cmp::max(1, std::cmp::min(*num, backends_count));
        }
    }
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum DABackendConfigType {
    OpenDa(DABackendOpenDAConfig),
}

#[derive(Clone, Default, Debug, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
#[serde(deny_unknown_fields)]
/// Open DA provides ability to access various storage services
pub struct DABackendOpenDAConfig {
    /// specifies the type of storage service to be used. 'gcs' with corresponding GCS server configuration, 's3' with corresponding S3 server configuration, etc
    #[serde(default)]
    pub scheme: OpenDAScheme,
    /// specifies the configuration of the storage service. 'gcs' with corresponding GCS server configuration, 's3' with corresponding S3 server configuration, etc.
    pub config: HashMap<String, String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    /// for fs backend:
    /// <namespace>/<segment_id> is the path to store the segment.
    /// If not set, the <derive_genesis_namespace>/<segment_id> is the full path
    /// If root is set in config, the <root>/<namespace>/<segment_id> is the full path
    /// for celestia:
    /// must be existed, it's Namespace in hex
    pub namespace: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    /// max segment size.
    /// Set at crates/rooch-da/src/backend/openda if None.
    pub max_segment_size: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    /// maximum number of attempts to retransmit a failed segment submission.
    pub max_retires: Option<usize>,
}

/// Derive a namespace from genesis config for DA backend (as default namespace open-da backend)
/// first 8 chars of sha256 of genesis in hex is used as namespace
pub fn derive_genesis_namespace(genesis: &[u8]) -> String {
    let raw = encode(sha2_256_of(genesis).0);
    raw.chars().take(8).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn calculate_submit_threshold() {
        let mut da_backend_config = DABackendConfig {
            submit_strategy: Some(DAServerSubmitStrategy::All),
            backends: vec![
                DABackendConfigType::OpenDa(DABackendOpenDAConfig {
                    scheme: OpenDAScheme::Fs,
                    config: HashMap::new(),
                    namespace: None,
                    max_segment_size: None,
                    max_retires: None,
                }),
                DABackendConfigType::OpenDa(DABackendOpenDAConfig {
                    scheme: OpenDAScheme::Fs,
                    config: HashMap::new(),
                    namespace: None,
                    max_segment_size: None,
                    max_retires: None,
                }),
            ],
            background_submit_interval: None,
        };
        assert_eq!(da_backend_config.calculate_submit_threshold(), 2);

        da_backend_config.submit_strategy = Some(DAServerSubmitStrategy::Quorum);
        assert_eq!(da_backend_config.calculate_submit_threshold(), 2);

        da_backend_config.submit_strategy = Some(DAServerSubmitStrategy::Number(1));
        assert_eq!(da_backend_config.calculate_submit_threshold(), 1);

        da_backend_config.submit_strategy = Some(DAServerSubmitStrategy::Number(3));
        assert_eq!(da_backend_config.calculate_submit_threshold(), 2);

        da_backend_config.submit_strategy = None;
        assert_eq!(da_backend_config.calculate_submit_threshold(), 2);
    }

    #[test]
    fn da_config_from_str() {
        let da_config_str = r#"{"da-backend": {"submit-strategy": "all",
        "backends": [{"open-da": {"scheme": "gcs", "config": {"bucket": "test-bucket", "credential": "test-credential"}}},
        {"open-da": {"scheme": "celestia", "config": {"endpoint": "test-conn", "auth_token": "test-auth"}, "namespace": "000000000000000000000000000000000000000102030405060708090a"}},
        {"open-da": {"scheme": "fs", "config": {}}}]}, "da-min-block-to-submit": 340282366920938463463374607431768211455}"#;

        let exp_gcs_config = DABackendOpenDAConfig {
            scheme: OpenDAScheme::Gcs,
            config: vec![
                ("bucket".to_string(), "test-bucket".to_string()),
                ("credential".to_string(), "test-credential".to_string()),
            ]
            .into_iter()
            .collect(),
            namespace: None,
            max_segment_size: None,
            max_retires: None,
        };
        let exp_celestia_config = DABackendOpenDAConfig {
            scheme: OpenDAScheme::Celestia,
            config: vec![
                ("endpoint".to_string(), "test-conn".to_string()),
                ("auth_token".to_string(), "test-auth".to_string()),
            ]
            .into_iter()
            .collect(),
            namespace: Some(
                "000000000000000000000000000000000000000102030405060708090a".to_string(),
            ),
            max_segment_size: None,
            max_retires: None,
        };
        let exp_fs_config = DABackendOpenDAConfig {
            scheme: OpenDAScheme::Fs,
            config: HashMap::new(),
            namespace: None,
            max_segment_size: None,
            max_retires: None,
        };
        let exp_da_config = DAConfig {
            da_backend: Some(DABackendConfig {
                submit_strategy: Some(DAServerSubmitStrategy::All),
                backends: vec![
                    DABackendConfigType::OpenDa(exp_gcs_config.clone()),
                    DABackendConfigType::OpenDa(exp_celestia_config.clone()),
                    DABackendConfigType::OpenDa(exp_fs_config.clone()),
                ],
                background_submit_interval: None,
            }),
            da_min_block_to_submit: Some(340282366920938463463374607431768211455),
            base: None,
        };
        match DAConfig::from_str(da_config_str) {
            Ok(da_config) => {
                assert_eq!(da_config, exp_da_config);
            }
            Err(e) => {
                println!(
                    "expected: {:?}",
                    serde_json::to_string(&exp_da_config).unwrap()
                );
                panic!("Error parsing DA Config: {}", e)
            }
        }

        let da_config_str = "{\"da-backend\": {\"backends\": [{\"open-da\": {\"scheme\": \"fs\", \"config\": {}}}]}}";
        let exp_da_config = DAConfig {
            da_backend: Some(DABackendConfig {
                submit_strategy: None,
                backends: vec![DABackendConfigType::OpenDa(exp_fs_config.clone())],
                background_submit_interval: None,
            }),
            da_min_block_to_submit: None,
            base: None,
        };
        match DAConfig::from_str(da_config_str) {
            Ok(da_config) => {
                assert_eq!(da_config, exp_da_config);
            }
            Err(e) => {
                panic!("Error parsing DA Config: {}", e)
            }
        }
    }
}
