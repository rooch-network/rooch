// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::backend::openda::avail::AvailClient;
use anyhow::anyhow;
use async_trait::async_trait;
use opendal::layers::{LoggingLayer, RetryLayer};
use opendal::Scheme;
use rooch_config::da_config::{DABackendOpenDAConfig, OpenDAScheme};
use rooch_config::retrieve_map_config_value;
use rooch_types::da::segment::SegmentID;
use std::collections::HashMap;

const DEFAULT_MAX_SEGMENT_SIZE: u64 = 4 * 1024 * 1024;
const DEFAULT_AVAIL_MAX_SEGMENT_SIZE: u64 = 512 * 1024;
const DEFAULT_MAX_RETRY_TIMES: usize = 4;

#[async_trait]
pub(crate) trait Operator: Sync + Send {
    async fn submit_segment(
        &self,
        segment_id: SegmentID,
        segment_bytes: Vec<u8>,
        prefix: Option<String>,
    ) -> anyhow::Result<()>;
}

#[async_trait]
impl Operator for opendal::Operator {
    async fn submit_segment(
        &self,
        segment_id: SegmentID,
        segment_bytes: Vec<u8>,
        prefix: Option<String>,
    ) -> anyhow::Result<()> {
        let path = match prefix {
            Some(prefix) => format!("{}/{}", prefix, segment_id),
            None => segment_id.to_string(),
        };
        let mut w = self.writer(&path).await?;
        w.write(segment_bytes).await?;
        w.close().await?;
        Ok(())
    }
}

pub(crate) async fn new_operator(
    scheme: OpenDAScheme,
    config: HashMap<String, String>,
    max_retry_times: Option<usize>,
) -> anyhow::Result<Box<dyn Operator>> {
    let operator: Box<dyn Operator> = match scheme {
        OpenDAScheme::Avail => Box::new(AvailClient::new(&config["endpoint"])?),
        _ => {
            let mut op = opendal::Operator::via_iter(Scheme::from(scheme), config)?;
            let max_times = max_retry_times.unwrap_or(DEFAULT_MAX_RETRY_TIMES);
            op = op
                .layer(RetryLayer::new().with_max_times(max_times))
                .layer(LoggingLayer::default());
            op.check().await?;
            Box::new(op)
        }
    };
    Ok(operator)
}

pub(crate) struct OperatorConfig {
    pub(crate) prefix: String,
    pub(crate) scheme: OpenDAScheme,
    pub(crate) max_segment_size: usize,
}

impl OperatorConfig {
    pub(crate) fn from_backend_config(
        cfg: DABackendOpenDAConfig,
        genesis_namespace: String,
    ) -> anyhow::Result<(Self, HashMap<String, String>)> {
        let backend_config = cfg.clone();
        let prefix = backend_config.namespace.unwrap_or(genesis_namespace);
        let scheme = backend_config.scheme;
        let mut map_config = backend_config.config;
        check_map_config(scheme.clone(), &mut map_config)?;

        let default_max_segment_size = match scheme {
            OpenDAScheme::Avail => DEFAULT_AVAIL_MAX_SEGMENT_SIZE,
            _ => DEFAULT_MAX_SEGMENT_SIZE,
        };

        let max_segment_size = cfg.max_segment_size.unwrap_or(default_max_segment_size) as usize;

        Ok((
            OperatorConfig {
                prefix,
                scheme,
                max_segment_size,
            },
            map_config,
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
