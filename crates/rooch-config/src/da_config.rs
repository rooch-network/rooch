// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::config::Config;
use crate::{retrieve_map_config_value, BaseConfig, MapConfigValueSource};
use moveos_types::h256::H256;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt::Display;
use std::path::{Path, PathBuf};
use std::str::FromStr;
use std::sync::Arc;

const DA_NAMESPACE_FROM_GENESIS_LENGTH: usize = 8;
const DEFAULT_OPENDA_FS_DIR: &str = "openda-fs";
// Default background submit interval: 5 seconds
// a smaller interval helps to reduce the delay of blocks-making and submitting.
//
// After the first background submit job which, the cursor will be updated to the last submitted block number.
// Only a few database operations are needed to catch up with the latest block numbers after a restart,
// so it's okay to have a small interval.
pub const DEFAULT_DA_BACKGROUND_SUBMIT_INTERVAL: u64 = 5;

/// This enum specifies the strategy for submitting DA data.
///
/// `All` means all backends must submit.
/// `Quorum` means a majority (>= n/2+1) must submit.
/// `Number(n)` means at least `n` backends must submit.
///
/// No matter what the strategy is, an independent process will sync all the data to all backends.
/// Eventual consistency is guaranteed.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum DASubmitStrategy {
    All,
    Quorum,
    Number(usize),
}

impl FromStr for DASubmitStrategy {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "all" => Ok(DASubmitStrategy::All),
            "quorum" => Ok(DASubmitStrategy::Quorum),
            _ => {
                if let Ok(n) = s.parse::<usize>() {
                    Ok(DASubmitStrategy::Number(n))
                } else {
                    Err(format!("invalid da submit strategy: {}", s))
                }
            }
        }
    }
}

impl Display for DASubmitStrategy {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DASubmitStrategy::All => write!(f, "all"),
            DASubmitStrategy::Quorum => write!(f, "quorum"),
            DASubmitStrategy::Number(n) => write!(f, "{}", n),
        }
    }
}

/// Represents the available Open-DA schemes supported by the backend.
///
/// Each enum variant corresponds to a specific backend type and its respective configuration.
#[derive(Clone, Debug, Default, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum OpenDAScheme {
    /// Local file system backend.
    ///
    /// Main configuration:
    /// - `root`: The root file path for storing data files.
    #[default]
    Fs,

    /// Google Cloud Storage (GCS) backend.
    ///
    /// Main configuration:
    /// - `bucket`: The storage bucket.
    /// - `credential`: The authentication credential (or `credential_path`, using a file path).
    Gcs,

    /// Amazon S3-compatible backend.
    ///
    /// Main configuration:
    /// - `bucket`: The storage bucket.
    /// - `region`: The AWS region.
    /// - `endpoint`: The S3 endpoint URL.
    /// - `access_key_id`: The AWS access key ID.
    /// - `secret_access_key`: The AWS secret access key.
    S3,

    /// Avail Fusion backend, supporting TurboDA and Light Client.
    ///
    /// Main configuration:
    /// - `turbo_endpoint`: The TurboDA service endpoint.
    /// - `turbo_auth_token`: The authentication token for TurboDA.
    /// - `light_endpoint`: The Light Client service endpoint.
    Avail,

    /// Celestia backend.
    ///
    /// Main configuration:
    /// - `endpoint`: The Celestia service endpoint.
    /// - `auth_token` (optional): The authentication token for accessing the Celestia backend.
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

/// Configuration for Data Availability (DA).
///
/// This struct controls how the node interacts with DA backends and specifies the starting point
/// for submitting blocks to DA. It balances flexibility, efficiency, and clarity while ensuring
/// compatibility with other configuration components.
#[derive(Clone, Default, Debug, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
#[serde(rename_all = "kebab-case")]
pub struct DAConfig {
    /// Specifies the configuration for the DA backends.
    ///
    /// This contains details about the backends used to ensure data availability,
    /// such as their types and additional configuration options. If not set, no DA
    /// backends will be used.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub da_backend: Option<DABackendConfig>,

    /// The first block to be submitted to the DA.
    ///
    /// If left unset, all blocks will be submitted starting from the genesis block.
    /// This allows flexibility in choosing whether to submit old blocks or just newly
    /// created ones.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub da_min_block_to_submit: Option<u128>,
    /// Specifies the interval for background submission in seconds.
    /// If not set, the default value is `DEFAULT_DA_BACKGROUND_SUBMIT_INTERVAL`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub background_submit_interval: Option<u64>,

    /// Internal reference to the base configuration.
    ///
    /// This is used internally by the node to access basic configuration details
    /// (e.g., data directories) and is initialized when the configuration is loaded.
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

        self.background_submit_interval
            .get_or_insert(DEFAULT_DA_BACKGROUND_SUBMIT_INTERVAL);

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
        self.data_dir().join(DEFAULT_OPENDA_FS_DIR)
    }
}

/// Configuration for DA (Data Availability) backends.
///
/// This struct defines how the node interacts with different DA backends,
/// including their types and the strategy used for submitting data to them.
#[derive(Clone, Default, Debug, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
#[serde(deny_unknown_fields)]
pub struct DABackendConfig {
    /// Configures the submission strategy for DA operations.
    ///
    /// This option defines how many backends are required to successfully process data submissions:
    /// - `All`: All backends must successfully submit the data.
    /// - `Quorum`: A majority (>= n/2 + 1) of backends must submit.
    /// - `Number(n)`: At least `n` backends must submit.
    ///
    /// If not set, the default behavior is equivalent to requiring `All`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub submit_strategy: Option<DASubmitStrategy>,

    /// Specifies the types of DA backends to be used.
    ///
    /// Each backend entry corresponds to a specific configuration.
    /// For example,
    /// - `OpenDA`: Configured for access to storage solutions like S3, GCS, etc.
    /// - Additional backend types can extend this field as the system grows.
    pub backends: Vec<DABackendConfigType>,
}

impl DABackendConfig {
    const DEFAULT_SUBMIT_STRATEGY: DASubmitStrategy = DASubmitStrategy::Number(1);

    pub fn calculate_submit_threshold(&mut self) -> usize {
        self.adjust_submit_strategy(); // Make sure submit_strategy is adjusted before calling this function.

        let backends_count = self.backends.len();
        match self.submit_strategy {
            Some(DASubmitStrategy::All) => backends_count,
            Some(DASubmitStrategy::Quorum) => backends_count / 2 + 1,
            Some(DASubmitStrategy::Number(number)) => number,
            None => 1, // Default to 1
        }
    }

    fn adjust_submit_strategy(&mut self) {
        let strategy = self
            .submit_strategy
            .get_or_insert(Self::DEFAULT_SUBMIT_STRATEGY);

        let backends_count = self.backends.len();

        // If it's Number, adjust the value to be within [1, n].
        if let DASubmitStrategy::Number(ref mut num) = strategy {
            *num = std::cmp::max(1, std::cmp::min(*num, backends_count));
        }
    }
}

/// Represents the type of DA (Data Availability) backend configuration.
///
/// Each variant corresponds to a specific backend type and its associated configuration.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum DABackendConfigType {
    /// OpenDA backend configuration.
    ///
    /// This variant contains the configuration specific to OpenDA, enabling access
    /// to various storage backends (e.g., Avail, Celestia, S3, GCS, etc.).
    OpenDa(DABackendOpenDAConfig),
}

/// Configuration for the Open DA backend.
///
/// Open DA provides the ability to interact with various backend implementations.
/// Each backend is defined by its unique configuration options.
#[derive(Clone, Default, Debug, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
#[serde(deny_unknown_fields)]
pub struct DABackendOpenDAConfig {
    /// Specifies the type of backend to be used.
    /// The `scheme` informs the backend logic on how to handle the associated configuration.
    #[serde(default)]
    pub scheme: OpenDAScheme,

    /// Specifies the detailed configuration for the selected backend.
    pub config: HashMap<String, String>,

    /// Specifies the namespace for data storage, depending on the backend.
    ///
    /// - **Filesystem-like backends** (e.g., S3, GCS, local filesystem):
    ///   - The path is structured as `<namespace>/<segment_id>` to store the segment.
    ///   - If not set:
    ///     - `<derive_genesis_namespace>/<segment_id>` is used as the full path.
    ///     - If the `root` field is set in the `config`, the full path becomes `<root>/<namespace>/<segment_id>`.
    /// - **Celestia**:
    ///   - The namespace must already exist and is specified directly in hexadecimal format.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub namespace: Option<String>,

    /// Specifies the maximum segment size (in bytes).
    ///
    /// - If not set, the backend implementation will use its default value.
    /// - This helps determine the maximum allowed size for data segments.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_segment_size: Option<u64>,

    /// Specifies the maximum number of retry attempts for failed segment submissions.
    ///
    /// - If not set, the backend implementation will determine the default number of retries.
    /// - This configuration can help fine-tune the reliability of segment submission in case of transient errors.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_retries: Option<usize>,
}

/// Derives a namespace from the genesis hash for the DA backend.
/// The resulting namespace is generated by taking the first NAMESPACE_LENGTH hexadecimal characters
/// of the genesis_hash.
pub fn derive_namespace_from_genesis(genesis_hash: H256) -> String {
    let encoded_hash = hex::encode(genesis_hash.0);
    encoded_hash
        .chars()
        .take(DA_NAMESPACE_FROM_GENESIS_LENGTH)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn calculate_submit_threshold() {
        let mut da_backend_config = DABackendConfig {
            submit_strategy: Some(DASubmitStrategy::All),
            backends: vec![
                DABackendConfigType::OpenDa(DABackendOpenDAConfig {
                    scheme: OpenDAScheme::Fs,
                    config: HashMap::new(),
                    namespace: None,
                    max_segment_size: None,
                    max_retries: None,
                }),
                DABackendConfigType::OpenDa(DABackendOpenDAConfig {
                    scheme: OpenDAScheme::Fs,
                    config: HashMap::new(),
                    namespace: None,
                    max_segment_size: None,
                    max_retries: None,
                }),
            ],
        };
        assert_eq!(da_backend_config.calculate_submit_threshold(), 2);

        da_backend_config.submit_strategy = Some(DASubmitStrategy::Quorum);
        assert_eq!(da_backend_config.calculate_submit_threshold(), 2);

        da_backend_config.submit_strategy = Some(DASubmitStrategy::Number(1));
        assert_eq!(da_backend_config.calculate_submit_threshold(), 1);

        da_backend_config.submit_strategy = Some(DASubmitStrategy::Number(3));
        assert_eq!(da_backend_config.calculate_submit_threshold(), 2);

        da_backend_config.submit_strategy = None;
        assert_eq!(da_backend_config.calculate_submit_threshold(), 1);
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
            max_retries: None,
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
            max_retries: None,
        };
        let exp_fs_config = DABackendOpenDAConfig {
            scheme: OpenDAScheme::Fs,
            config: HashMap::new(),
            namespace: None,
            max_segment_size: None,
            max_retries: None,
        };
        let exp_da_config = DAConfig {
            da_backend: Some(DABackendConfig {
                submit_strategy: Some(DASubmitStrategy::All),
                backends: vec![
                    DABackendConfigType::OpenDa(exp_gcs_config.clone()),
                    DABackendConfigType::OpenDa(exp_celestia_config.clone()),
                    DABackendConfigType::OpenDa(exp_fs_config.clone()),
                ],
            }),
            da_min_block_to_submit: Some(340282366920938463463374607431768211455),
            background_submit_interval: None,
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
            }),
            da_min_block_to_submit: None,
            background_submit_interval: None,
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

        let da_config_str = "{\"da-backend\":{\"backends\":[{\"open-da\":{\"scheme\":\"gcs\",\"config\":{\"bucket\":\"$OPENDA_GCP_TESTNET_BUCKET\",\"credential\":\"$OPENDA_GCP_TESTNET_CREDENTIAL\"}}},{\"open-da\":{\"scheme\":\"avail\",\"config\":{\"turbo_endpoint\":\"$TURBO_DA_TURING_ENDPOINT\",\"turbo_auth_token\":\"$TURBO_DA_TURING_TOKEN\"}}}]}}";
        let exp_da_config = DAConfig {
            da_backend: Some(DABackendConfig {
                submit_strategy: None,
                backends: vec![
                    DABackendConfigType::OpenDa(DABackendOpenDAConfig {
                        scheme: OpenDAScheme::Gcs,
                        config: vec![
                            (
                                "bucket".to_string(),
                                "$OPENDA_GCP_TESTNET_BUCKET".to_string(),
                            ),
                            (
                                "credential".to_string(),
                                "$OPENDA_GCP_TESTNET_CREDENTIAL".to_string(),
                            ),
                        ]
                        .into_iter()
                        .collect(),
                        namespace: None,
                        max_segment_size: None,
                        max_retries: None,
                    }),
                    DABackendConfigType::OpenDa(DABackendOpenDAConfig {
                        scheme: OpenDAScheme::Avail,
                        config: vec![
                            (
                                "turbo_endpoint".to_string(),
                                "$TURBO_DA_TURING_ENDPOINT".to_string(),
                            ),
                            (
                                "turbo_auth_token".to_string(),
                                "$TURBO_DA_TURING_TOKEN".to_string(),
                            ),
                        ]
                        .into_iter()
                        .collect(),
                        namespace: None,
                        max_segment_size: None,
                        max_retries: None,
                    }),
                ],
            }),
            da_min_block_to_submit: None,
            background_submit_interval: None,
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
