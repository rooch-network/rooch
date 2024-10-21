// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::backend::DABackend;
use anyhow::anyhow;
use async_trait::async_trait;
use opendal::layers::{LoggingLayer, RetryLayer};
use opendal::{Operator, Scheme};
use rooch_config::da_config::{DABackendOpenDAConfig, OpenDAScheme};
use rooch_config::retrieve_map_config_value;
use rooch_types::da::batch::DABatch;
use rooch_types::da::chunk::{Chunk, ChunkV0};
use rooch_types::da::segment::SegmentID;
use std::collections::HashMap;

pub const DEFAULT_MAX_SEGMENT_SIZE: u64 = 4 * 1024 * 1024;
pub const DEFAULT_MAX_RETRY_TIMES: usize = 4;

#[async_trait]
impl DABackend for OpenDABackend {
    async fn submit_batch(&self, batch: DABatch) -> anyhow::Result<()> {
        self.pub_batch(batch).await
    }
}

pub struct OpenDABackend {
    prefix: String,
    scheme: OpenDAScheme,
    max_segment_size: usize,
    operator: Operator,
}

fn check_backend_config(
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
        OpenDAScheme::S3 => {
            todo!("s3 backend is not implemented yet");
        }
    }
}

impl OpenDABackend {
    pub async fn new(
        cfg: &DABackendOpenDAConfig,
        genesis_namespace: String,
    ) -> anyhow::Result<OpenDABackend> {
        let backend_config = cfg.clone();
        let prefix = backend_config.namespace.unwrap_or(genesis_namespace);
        let scheme = backend_config.scheme;
        let mut map_config = backend_config.config;
        check_backend_config(scheme.clone(), &mut map_config)?;

        let op: Operator =
            new_retry_operator(Scheme::from(scheme.clone()), map_config, None).await?;

        Ok(Self {
            prefix,
            scheme,
            max_segment_size: cfg.max_segment_size.unwrap_or(DEFAULT_MAX_SEGMENT_SIZE) as usize,
            operator: op,
        })
    }

    pub async fn pub_batch(&self, batch: DABatch) -> anyhow::Result<()> {
        let chunk: ChunkV0 = batch.into();

        let prefix = self.prefix.clone();
        let max_segment_size = self.max_segment_size;
        let segments = chunk.to_segments(max_segment_size);
        for segment in segments {
            let bytes = segment.to_bytes();

            match self
                .write_segment(segment.get_id(), bytes, Some(prefix.clone()))
                .await
            {
                Ok(_) => {
                    log::info!(
                        "submitted segment to open-da scheme: {:?}, segment_id: {:?}",
                        self.scheme,
                        segment.get_id(),
                    );
                }
                Err(e) => {
                    log::warn!(
                        "failed to submit segment to open-da scheme: {:?}, segment_id: {:?}, error:{:?}",
                        self.scheme,
                        segment.get_id(),
                        e,
                    );
                    return Err(e);
                }
            }
        }

        Ok(())
    }

    async fn write_segment(
        &self,
        segment_id: SegmentID,
        segment_bytes: Vec<u8>,
        prefix: Option<String>,
    ) -> anyhow::Result<()> {
        let path = match prefix {
            Some(prefix) => format!("{}/{}", prefix, segment_id),
            None => segment_id.to_string(),
        };
        let mut w = self.operator.writer(&path).await?;
        w.write(segment_bytes).await?;
        w.close().await?;
        Ok(())
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

async fn new_retry_operator(
    scheme: Scheme,
    config: HashMap<String, String>,
    max_retry_times: Option<usize>,
) -> anyhow::Result<Operator> {
    let mut op = Operator::via_map(scheme, config)?;
    let max_times = max_retry_times.unwrap_or(DEFAULT_MAX_RETRY_TIMES);
    op = op
        .layer(RetryLayer::new().with_max_times(max_times))
        .layer(LoggingLayer::default());
    op.check().await?;
    Ok(op)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[test]
    fn test_check_backend_config_fs() {
        let scheme = OpenDAScheme::Fs;
        let mut map_config = HashMap::new();
        let result = check_backend_config(scheme.clone(), &mut map_config);
        assert!(
            result.is_err(),
            "FS scheme should return Err if 'root' is missing"
        );

        map_config.insert("root".to_string(), "/some/path".to_string());
        let result = check_backend_config(scheme, &mut map_config);
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
        let result = check_backend_config(scheme.clone(), &mut map_config);
        assert!(
            result.is_err(),
            "GCS scheme should return Err if 'bucket' is missing"
        );

        map_config.insert("bucket".to_string(), "test_bucket".to_string());
        let result = check_backend_config(scheme.clone(), &mut map_config);
        assert!(
            result.is_ok(),
            "GCS scheme should return Ok if 'bucket' and 'credential' are provided"
        );

        map_config.remove("credential");
        map_config.insert(
            "credential_path".to_string(),
            "test_credential_path".to_string(),
        );
        let result2 = check_backend_config(scheme.clone(), &mut map_config);
        assert!(
            result2.is_ok(),
            "GCS scheme should return Ok if 'bucket' and 'credential_path' are provided"
        );

        map_config.remove("credential_path");

        let result3 = check_backend_config(scheme, &mut map_config);
        assert!(result3.is_err(), "GCS scheme should return Err if neither 'credential' nor 'credential_path' are provided");

        assert_eq!(map_config.get("default_storage_class").unwrap(), "STANDARD");
    }
}
