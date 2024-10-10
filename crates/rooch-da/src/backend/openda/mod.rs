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
use std::path::Path;

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

impl OpenDABackend {
    pub async fn new(
        cfg: &DABackendOpenDAConfig,
        genesis_namespace: String,
    ) -> anyhow::Result<OpenDABackend> {
        let mut config = cfg.clone();

        let op: Operator = match config.scheme {
            OpenDAScheme::Fs => {
                // root must be existed
                check_config_exist(OpenDAScheme::Fs, &config.config, "root")?;
                new_retry_operator(Scheme::Fs, config.config, None).await?
            }
            OpenDAScheme::Gcs => {
                retrieve_map_config_value(
                    &mut config.config,
                    "bucket",
                    Some("OPENDA_GCS_BUCKET"),
                    Some("rooch-openda-dev"),
                );
                if !config.config.contains_key("credential") {
                    if let Ok(credential) = std::env::var("OPENDA_GCS_CREDENTIAL") {
                        config.config.insert("credential".to_string(), credential);
                    }
                }
                if config.config.contains_key("credential") {
                    let credential = {
                        let credential_path = Path::new(config.config.get("credential").unwrap());

                        if credential_path.exists() {
                            Some(config.config.get("credential").unwrap().to_string())
                        } else {
                            None
                        }
                    };

                    // it's a path, using credential_path instead
                    if let Some(credential) = credential {
                        config.config.remove("credential");
                        config
                            .config
                            .insert("credential_path".to_string(), credential);
                    }
                }
                retrieve_map_config_value(
                    &mut config.config,
                    "default_storage_class",
                    Some("OPENDA_GCS_DEFAULT_STORAGE_CLASS"),
                    Some("STANDARD"),
                );

                check_config_exist(OpenDAScheme::Gcs, &config.config, "bucket")?;
                match (
                    check_config_exist(OpenDAScheme::Gcs, &config.config, "credential"),
                    check_config_exist(OpenDAScheme::Gcs, &config.config, "credential_path"),
                ) {
                    (Ok(_), Ok(_)) => (),

                    // credential existed
                    (Ok(_), Err(_)) => (),
                    // credential_path existed
                    (Err(_), Ok(_)) => (),

                    (Err(_), Err(_)) => {
                        return Err(anyhow!(
                            "credential no found in config for scheme {:?}",
                            OpenDAScheme::Gcs
                        ));
                    }
                }

                // After setting defaults, proceed with creating Operator
                new_retry_operator(Scheme::Gcs, config.config, None).await?
            }
            OpenDAScheme::S3 => {
                todo!("s3 backend is not implemented yet");
            }
        };

        Ok(Self {
            prefix: config.namespace.unwrap_or(genesis_namespace),
            scheme: config.scheme,
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
