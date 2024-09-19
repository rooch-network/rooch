// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use anyhow::{anyhow, Result};
use async_trait::async_trait;
use coerce::actor::context::ActorContext;
use coerce::actor::message::Handler;
use coerce::actor::Actor;
use opendal::layers::{LoggingLayer, RetryLayer};
use opendal::{Operator, Scheme};
use rooch_config::config::retrieve_map_config_value;
use std::collections::HashMap;
use std::path::Path;

use crate::chunk::{Chunk, ChunkV0};
use rooch_config::da_config::{DAServerOpenDAConfig, OpenDAScheme};

use crate::messages::PutBatchInternalDAMessage;

pub struct DAServerOpenDAActor {
    max_segment_size: usize,
    operator: Operator,
}

pub const DEFAULT_MAX_SEGMENT_SIZE: u64 = 4 * 1024 * 1024;
pub const DEFAULT_MAX_RETRY_TIMES: usize = 4;

impl Actor for DAServerOpenDAActor {}

impl DAServerOpenDAActor {
    pub async fn new(cfg: &DAServerOpenDAConfig) -> Result<DAServerOpenDAActor> {
        let mut config = cfg.clone();

        let op: Operator = match config.scheme {
            OpenDAScheme::Fs => {
                // root must be existed
                if !config.config.contains_key("root") {
                    return Err(anyhow!(
                        "key 'root' must be existed in config for scheme {:?}",
                        OpenDAScheme::Fs
                    ));
                }
                new_retry_operator(Scheme::Fs, config.config, None).await?
            }
            OpenDAScheme::Gcs => {
                // If certain keys don't exist in the map, set them from environment
                if !config.config.contains_key("bucket") {
                    if let Ok(bucket) = std::env::var("OPENDA_GCS_BUCKET") {
                        config.config.insert("bucket".to_string(), bucket);
                    }
                }
                if !config.config.contains_key("root") {
                    if let Ok(root) = std::env::var("OPENDA_GCS_ROOT") {
                        config.config.insert("root".to_string(), root);
                    }
                }
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
                    "STANDARD",
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
                        return Err(anyhow!("either 'credential' or 'credential_path' must exist in config for scheme {:?}", OpenDAScheme::Gcs));
                    }
                }

                // After setting defaults, proceed with creating Operator
                new_retry_operator(Scheme::Gcs, config.config, None).await?
            }
            _ => Err(anyhow!("unsupported open-da scheme: {:?}", config.scheme))?,
        };

        Ok(Self {
            max_segment_size: cfg.max_segment_size.unwrap_or(DEFAULT_MAX_SEGMENT_SIZE) as usize,
            operator: op,
        })
    }

    pub async fn pub_batch(&self, batch_msg: PutBatchInternalDAMessage) -> Result<()> {
        let chunk: ChunkV0 = batch_msg.batch.into();
        let segments = chunk.to_segments(self.max_segment_size);
        for segment in segments {
            let bytes = segment.to_bytes();
            match self
                .operator
                .write(&segment.get_id().to_string(), bytes)
                .await
            {
                Ok(_) => {
                    log::info!(
                        "submitted segment to open-da node, segment: {:?}",
                        segment.get_id(),
                    );
                }
                Err(e) => {
                    log::warn!(
                        "failed to submit segment to open-da node, segment_id: {:?}, error:{:?}",
                        segment.get_id(),
                        e,
                    );
                    return Err(e.into());
                }
            }
        }

        Ok(())
    }
}

fn check_config_exist(
    scheme: OpenDAScheme,
    config: &HashMap<String, String>,
    key: &str,
) -> Result<()> {
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
) -> Result<Operator> {
    let mut op = Operator::via_map(scheme, config)?;
    let max_times = max_retry_times.unwrap_or(DEFAULT_MAX_RETRY_TIMES);
    op = op
        .layer(RetryLayer::new().with_max_times(max_times))
        .layer(LoggingLayer::default());
    op.check().await?;
    Ok(op)
}

#[async_trait]
impl Handler<PutBatchInternalDAMessage> for DAServerOpenDAActor {
    async fn handle(
        &mut self,
        msg: PutBatchInternalDAMessage,
        _ctx: &mut ActorContext,
    ) -> Result<()> {
        self.pub_batch(msg).await
    }
}
