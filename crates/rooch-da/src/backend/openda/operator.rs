// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::backend::openda::avail::{AvailFusionClientConfig, DEFAULT_AVAIL_MAX_SEGMENT_SIZE};
use crate::backend::openda::celestia::{
    CelestiaClient, WrappedNamespace, DEFAULT_CELESTIA_MAX_SEGMENT_SIZE,
};
use crate::backend::openda::opendal::BACK_OFF_MIN_DELAY;
use anyhow::anyhow;
use async_trait::async_trait;
use opendal::layers::{LoggingLayer, RetryLayer};
use opendal::Scheme;
use rooch_config::da_config::{DABackendOpenDAConfig, OpenDAScheme};
use rooch_config::retrieve_map_config_value;
use rooch_types::da::segment::SegmentID;
use std::collections::HashMap;

const DEFAULT_MAX_SEGMENT_SIZE: u64 = 8 * 1024 * 1024;
pub(crate) const DEFAULT_MAX_RETRY_TIMES: usize = 4;

#[async_trait]
pub(crate) trait Operator: Sync + Send {
    async fn submit_segment(
        &self,
        segment_id: SegmentID,
        segment_bytes: Vec<u8>,
        prefix: Option<String>,
    ) -> anyhow::Result<()>;
}

pub(crate) async fn new_operator(
    operator_config: OperatorConfig,
    scheme_config: HashMap<String, String>,
) -> anyhow::Result<Box<dyn Operator>> {
    let max_retries = operator_config.max_retries;
    let scheme = operator_config.scheme.clone();

    let operator: Box<dyn Operator> = match scheme {
        OpenDAScheme::Avail => {
            let avail_fusion_config =
                AvailFusionClientConfig::from_scheme_config(scheme_config, max_retries)?;
            let avail_fusion_client = avail_fusion_config.build_client()?;
            Box::new(avail_fusion_client)
        }
        OpenDAScheme::Celestia => {
            let namespace = WrappedNamespace::from_string(&operator_config.namespace.clone())?;
            Box::new(
                CelestiaClient::new(
                    namespace.into_inner(),
                    &scheme_config["endpoint"],
                    scheme_config.get("auth_token").map(|s| s.as_str()),
                    max_retries,
                )
                .await?,
            )
        }
        _ => {
            let mut op = opendal::Operator::via_iter(Scheme::from(scheme), scheme_config)?;
            op = op
                .layer(
                    RetryLayer::new()
                        .with_max_times(max_retries)
                        .with_min_delay(BACK_OFF_MIN_DELAY),
                )
                .layer(LoggingLayer::default());
            op.check().await?;
            Box::new(op)
        }
    };
    Ok(operator)
}

#[derive(Clone)]
pub(crate) struct OperatorConfig {
    pub(crate) namespace: String,
    pub(crate) scheme: OpenDAScheme,
    pub(crate) max_segment_size: usize,
    pub(crate) max_retries: usize,
}

impl OperatorConfig {
    pub(crate) fn from_backend_config(
        cfg: DABackendOpenDAConfig,
        genesis_namespace: String,
    ) -> anyhow::Result<(Self, HashMap<String, String>)> {
        let backend_config = cfg.clone();
        let max_retries = backend_config
            .max_retires
            .unwrap_or(DEFAULT_MAX_RETRY_TIMES);
        let scheme = backend_config.scheme;
        if scheme == OpenDAScheme::Celestia && backend_config.namespace.is_none() {
            return Err(anyhow!(
                "namespace must be provided for scheme {:?}",
                scheme
            ));
        }
        let namespace = backend_config.namespace.unwrap_or(genesis_namespace);
        let mut scheme_config = backend_config.config;
        check_map_config(scheme.clone(), &mut scheme_config)?;

        let default_max_segment_size = match scheme {
            OpenDAScheme::Avail => DEFAULT_AVAIL_MAX_SEGMENT_SIZE,
            OpenDAScheme::Celestia => DEFAULT_CELESTIA_MAX_SEGMENT_SIZE,
            _ => DEFAULT_MAX_SEGMENT_SIZE,
        };

        let max_segment_size = cfg.max_segment_size.unwrap_or(default_max_segment_size) as usize;

        Ok((
            OperatorConfig {
                namespace,
                scheme,
                max_segment_size,
                max_retries,
            },
            scheme_config,
        ))
    }
}

fn check_map_config(
    scheme: OpenDAScheme,
    map_config: &mut HashMap<String, String>,
) -> anyhow::Result<()> {
    match scheme {
        OpenDAScheme::Fs => {
            // root must be existed
            check_config_exist(OpenDAScheme::Fs, map_config, "root")
        }
        OpenDAScheme::Gcs => {
            retrieve_map_config_value(map_config, "bucket", Some("OPENDA_GCS_BUCKET"), None);

            retrieve_map_config_value(
                map_config,
                "credential",
                Some("OPENDA_GCS_CREDENTIAL"),
                None,
            );
            retrieve_map_config_value(
                map_config,
                "credential_path",
                Some("OPENDA_GCS_CREDENTIAL_PATH"),
                None,
            );

            retrieve_map_config_value(
                map_config,
                "default_storage_class",
                Some("OPENDA_GCS_DEFAULT_STORAGE_CLASS"),
                Some("STANDARD"),
            );

            check_config_exist(OpenDAScheme::Gcs, map_config, "bucket")?;
            match (
                check_config_exist(OpenDAScheme::Gcs, map_config, "credential"),
                check_config_exist(OpenDAScheme::Gcs, map_config, "credential_path"),
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
            }
        }
        OpenDAScheme::Avail => check_config_exist(OpenDAScheme::Avail, map_config, "endpoint"),
        OpenDAScheme::Celestia => {
            check_config_exist(OpenDAScheme::Celestia, map_config, "endpoint")
        }
        OpenDAScheme::S3 => {
            todo!("s3 backend is not implemented yet");
        }
    }
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

    #[test]
    fn test_check_backend_config_fs() {
        let scheme = OpenDAScheme::Fs;
        let mut map_config = HashMap::new();
        let result = check_map_config(scheme.clone(), &mut map_config);
        assert!(
            result.is_err(),
            "FS scheme should return Err if 'root' is missing"
        );

        map_config.insert("root".to_string(), "/some/path".to_string());
        let result = check_map_config(scheme, &mut map_config);
        assert!(
            result.is_ok(),
            "FS scheme should return Ok if 'root' is provided"
        );
    }

    #[test]
    fn test_check_backend_config_gcs() {
        let scheme = OpenDAScheme::Gcs;
        let mut map_config = HashMap::new();
        map_config.insert("credential".to_string(), "test_credential".to_string());
        let result = check_map_config(scheme.clone(), &mut map_config);
        assert!(
            result.is_err(),
            "GCS scheme should return Err if 'bucket' is missing"
        );

        map_config.insert("bucket".to_string(), "test_bucket".to_string());
        let result = check_map_config(scheme.clone(), &mut map_config);
        assert!(
            result.is_ok(),
            "GCS scheme should return Ok if 'bucket' and 'credential' are provided"
        );

        map_config.remove("credential");
        map_config.insert(
            "credential_path".to_string(),
            "test_credential_path".to_string(),
        );
        let result2 = check_map_config(scheme.clone(), &mut map_config);
        assert!(
            result2.is_ok(),
            "GCS scheme should return Ok if 'bucket' and 'credential_path' are provided"
        );

        map_config.remove("credential_path");

        let result3 = check_map_config(scheme, &mut map_config);
        assert!(result3.is_err(), "GCS scheme should return Err if neither 'credential' nor 'credential_path' are provided");

        assert_eq!(map_config.get("default_storage_class").unwrap(), "STANDARD");
    }
}
