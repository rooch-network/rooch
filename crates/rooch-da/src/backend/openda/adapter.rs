// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::backend::openda::avail::{
    AvailFusionClientConfig, DEFAULT_AVAIL_MAX_RETRIES, DEFAULT_AVAIL_MAX_SEGMENT_SIZE,
};
use crate::backend::openda::celestia::{
    CelestiaAdapter, WrappedNamespace, DEFAULT_CELESTIA_MAX_RETRIES,
    DEFAULT_CELESTIA_MAX_SEGMENT_SIZE,
};
use crate::backend::openda::opendal::OpenDalAdapter;
use anyhow::anyhow;
use async_trait::async_trait;
use rooch_config::da_config::{DABackendOpenDAConfig, OpenDAScheme};
use rooch_config::retrieve_map_config_value;
use rooch_types::da::segment::SegmentID;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

const DEFAULT_MAX_SEGMENT_SIZE: u64 = 8 * 1024 * 1024;
pub(crate) const DEFAULT_MAX_RETRY_TIMES: usize = 3;

/// OpenDAAdapter connecting to OpenDA-compatible backends
#[async_trait]
pub(crate) trait OpenDAAdapter: Sync + Send {
    async fn submit_segment(
        &self,
        segment_id: SegmentID,
        segment_bytes: &[u8],
        is_last_segment: bool,
    ) -> anyhow::Result<()>;
}

#[derive(Clone)]
pub struct AdapterSubmitStat {
    inner: Arc<RwLock<AdapterSubmitStatInner>>,
}

struct AdapterSubmitStatInner {
    chunk_id: u128,
    segment_number_sum: u64,
    latest_done_chunk_id: u128,
}

impl Default for AdapterSubmitStat {
    fn default() -> Self {
        Self::new()
    }
}

impl AdapterSubmitStat {
    pub fn new() -> Self {
        Self {
            inner: Arc::new(RwLock::new(AdapterSubmitStatInner {
                chunk_id: 0,
                segment_number_sum: 0,
                latest_done_chunk_id: 0,
            })),
        }
    }

    pub async fn add_done_segment(&self, segment_id: SegmentID, is_last_segment: bool) {
        let mut inner = self.inner.write().await;
        if segment_id.chunk_id != inner.chunk_id {
            // new chunk
            inner.segment_number_sum = 0;
            inner.chunk_id = segment_id.chunk_id;
        }
        inner.segment_number_sum += segment_id.segment_number;
        if is_last_segment {
            let mut exp_segment_number_sum = 0;
            for i in 0..=segment_id.segment_number {
                exp_segment_number_sum += i;
            }
            if exp_segment_number_sum == inner.segment_number_sum {
                // only accept segments added in order
                inner.latest_done_chunk_id = inner.chunk_id;
            }
        }
    }

    pub async fn get_latest_done_chunk_id(&self) -> u128 {
        let inner = self.inner.read().await;
        inner.latest_done_chunk_id
    }
}

#[derive(Clone)]
pub(crate) struct OpenDAAdapterConfig {
    pub(crate) namespace: String,
    pub(crate) max_segment_size: usize,
    pub(crate) max_retries: usize,
    pub(crate) scheme: OpenDAScheme,
    pub(crate) scheme_config: HashMap<String, String>,
}

impl OpenDAAdapterConfig {
    pub(crate) fn derive_from_open_da_config(
        open_da_config: &DABackendOpenDAConfig,
    ) -> anyhow::Result<Self> {
        let scheme = open_da_config.scheme.clone();
        let namespace = open_da_config.namespace.clone().ok_or(anyhow!(
            "namespace must have been initialed before creating OpenDAAdapterConfig"
        ))?;
        let mut scheme_config = open_da_config.config.clone();
        check_scheme_config(scheme.clone(), &mut scheme_config, namespace.clone())?;

        let (default_max_segment_size, default_max_retries) = match scheme {
            OpenDAScheme::Avail => (DEFAULT_AVAIL_MAX_SEGMENT_SIZE, DEFAULT_AVAIL_MAX_RETRIES),
            OpenDAScheme::Celestia => (
                DEFAULT_CELESTIA_MAX_SEGMENT_SIZE,
                DEFAULT_CELESTIA_MAX_RETRIES,
            ),
            _ => (DEFAULT_MAX_SEGMENT_SIZE, DEFAULT_MAX_RETRY_TIMES),
        };
        let max_retries = open_da_config.max_retries.unwrap_or(default_max_retries);
        let max_segment_size = open_da_config
            .max_segment_size
            .unwrap_or(default_max_segment_size) as usize;

        Ok(OpenDAAdapterConfig {
            namespace,
            max_segment_size,
            max_retries,
            scheme,
            scheme_config,
        })
    }

    pub(crate) async fn build(
        &self,
        stats: AdapterSubmitStat,
    ) -> anyhow::Result<Box<dyn OpenDAAdapter>> {
        let max_retries = self.max_retries;
        let scheme = self.scheme.clone();
        let scheme_config = self.scheme_config.clone();

        let operator: Box<dyn OpenDAAdapter> = match scheme {
            OpenDAScheme::Avail => {
                let avail_fusion_config =
                    AvailFusionClientConfig::from_scheme_config(scheme_config, max_retries)?;
                let avail_fusion_client = avail_fusion_config.build_client(stats)?;
                Box::new(avail_fusion_client)
            }
            OpenDAScheme::Celestia => {
                let namespace = WrappedNamespace::from_string(&self.namespace.clone())?;
                Box::new(
                    CelestiaAdapter::new(
                        namespace.into_inner(),
                        &scheme_config["endpoint"],
                        scheme_config.get("auth_token").map(|s| s.as_str()),
                        max_retries,
                        stats,
                    )
                    .await?,
                )
            }
            _ => {
                let adapter =
                    OpenDalAdapter::new(scheme, scheme_config, max_retries, stats).await?;
                Box::new(adapter)
            }
        };
        Ok(operator)
    }
}

fn check_scheme_config(
    scheme: OpenDAScheme,
    config: &mut HashMap<String, String>,
    namespace: String,
) -> anyhow::Result<()> {
    match scheme {
        OpenDAScheme::Fs => {
            // root must be existed
            check_config_exist(OpenDAScheme::Fs, config, "root")?;
        }
        OpenDAScheme::Gcs => {
            retrieve_map_config_value(config, "bucket", Some("OPENDA_GCS_BUCKET"), None);

            retrieve_map_config_value(config, "credential", Some("OPENDA_GCS_CREDENTIAL"), None);
            retrieve_map_config_value(
                config,
                "credential_path",
                Some("OPENDA_GCS_CREDENTIAL_PATH"),
                None,
            );

            retrieve_map_config_value(
                config,
                "default_storage_class",
                Some("OPENDA_GCS_DEFAULT_STORAGE_CLASS"),
                Some("STANDARD"),
            );

            check_config_exist(OpenDAScheme::Gcs, config, "bucket")?;
            match (
                check_config_exist(OpenDAScheme::Gcs, config, "credential"),
                check_config_exist(OpenDAScheme::Gcs, config, "credential_path"),
            ) {
                (Ok(_), Ok(_)) => Ok(()),
                // credential existed
                (Ok(_), Err(_)) => Ok(()),
                // credential_path existed
                (Err(_), Ok(_)) => Ok(()),

                (Err(_), Err(_)) => Err(anyhow!(
                    "credential no found in config for scheme {:?}",
                    OpenDAScheme::Gcs
                )),
            }?;
        }

        OpenDAScheme::Celestia => {
            check_config_exist(OpenDAScheme::Celestia, config, "endpoint")?;
        }
        _ => {}
    };

    // Set "root" in config for Filesystem-like backends (if not, `root` will be ignored directly)
    //   - If not set:
    //     - using /`namespace`.
    //     - If the `root` field is set in the `config`, set it to `<original_root>/<namespace>`.
    let namespace_without_first_slash = namespace.trim_start_matches('/');
    if let Some(root) = config.get("root") {
        let root = root.clone();
        config.insert(
            "root".to_string(),
            format!("{}/{}", root, namespace_without_first_slash),
        );
    } else {
        config.insert(
            "root".to_string(),
            format!("/{}", namespace_without_first_slash),
        );
    }
    Ok(())
}

fn check_config_exist(
    scheme: OpenDAScheme,
    config: &HashMap<String, String>,
    key: &str,
) -> anyhow::Result<()> {
    if config.contains_key(key) {
        Ok(())
    } else {
        Err(anyhow!(
            "key {} must be existed in config for scheme {:?}",
            key,
            scheme
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    const TEST_NAMESPACE: &str = "test_namespace";
    const TEST_NAMESPACE_SLASH: &str = "/test_namespace";

    #[test]
    fn check_scheme_config_fs() {
        let scheme = OpenDAScheme::Fs;
        let mut map_config = HashMap::new();
        let result =
            check_scheme_config(scheme.clone(), &mut map_config, TEST_NAMESPACE.to_string());
        assert!(
            result.is_err(),
            "FS scheme should return Err if 'root' is missing"
        );

        map_config.insert("root".to_string(), "/some/path".to_string());
        let result = check_scheme_config(scheme, &mut map_config, TEST_NAMESPACE.to_string());
        assert!(
            result.is_ok(),
            "FS scheme should return Ok if 'root' is provided"
        );
        assert_eq!(map_config.get("root").unwrap(), "/some/path/test_namespace");
    }

    #[test]
    fn check_scheme_config_gcs() {
        let scheme = OpenDAScheme::Gcs;
        let mut map_config = HashMap::new();
        map_config.insert("credential".to_string(), "test_credential".to_string());
        let result =
            check_scheme_config(scheme.clone(), &mut map_config, TEST_NAMESPACE.to_string());
        assert!(
            result.is_err(),
            "GCS scheme should return Err if 'bucket' is missing"
        );

        map_config.insert("bucket".to_string(), "test_bucket".to_string());
        let result =
            check_scheme_config(scheme.clone(), &mut map_config, TEST_NAMESPACE.to_string());
        assert!(
            result.is_ok(),
            "GCS scheme should return Ok if 'bucket' and 'credential' are provided"
        );

        assert_eq!(map_config.get("root").unwrap(), "/test_namespace");
        map_config.insert("root".to_string(), "/some/path".to_string());
        let result = check_scheme_config(
            scheme.clone(),
            &mut map_config,
            TEST_NAMESPACE_SLASH.to_string(),
        );
        assert!(result.is_ok(), "{}", result.unwrap_err());
        assert_eq!(map_config.get("root").unwrap(), "/some/path/test_namespace");

        map_config.remove("credential");
        map_config.insert(
            "credential_path".to_string(),
            "test_credential_path".to_string(),
        );
        let result2 =
            check_scheme_config(scheme.clone(), &mut map_config, TEST_NAMESPACE.to_string());
        assert!(
            result2.is_ok(),
            "GCS scheme should return Ok if 'bucket' and 'credential_path' are provided"
        );

        map_config.remove("credential_path");

        let result3 = check_scheme_config(scheme, &mut map_config, TEST_NAMESPACE.to_string());
        assert!(result3.is_err(), "GCS scheme should return Err if neither 'credential' nor 'credential_path' are provided");

        assert_eq!(map_config.get("default_storage_class").unwrap(), "STANDARD");
    }

    #[tokio::test]
    async fn test_adapter_submit_stats() {
        let stats = AdapterSubmitStat::new();
        assert_eq!(stats.get_latest_done_chunk_id().await, 0);

        let segment_id1 = SegmentID {
            chunk_id: 1,
            segment_number: 0,
        };
        stats.add_done_segment(segment_id1, false).await;
        assert_eq!(stats.get_latest_done_chunk_id().await, 0);

        let segment_id2 = SegmentID {
            chunk_id: 1,
            segment_number: 1,
        };
        stats.add_done_segment(segment_id2, true).await;
        assert_eq!(stats.get_latest_done_chunk_id().await, 1);

        let segment_id3 = SegmentID {
            chunk_id: 2,
            segment_number: 0,
        };
        stats.add_done_segment(segment_id3, false).await;
        assert_eq!(stats.get_latest_done_chunk_id().await, 1);

        let segment_id4 = SegmentID {
            chunk_id: 2,
            segment_number: 1,
        };
        stats.add_done_segment(segment_id4, false).await;
        assert_eq!(stats.get_latest_done_chunk_id().await, 1);

        let segment_id5 = SegmentID {
            chunk_id: 2,
            segment_number: 2,
        };
        stats.add_done_segment(segment_id5, true).await;
        assert_eq!(stats.get_latest_done_chunk_id().await, 2);
    }
}
