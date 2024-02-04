// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use anyhow::{anyhow, Result};
use async_trait::async_trait;
use coerce::actor::context::ActorContext;
use coerce::actor::message::Handler;
use coerce::actor::Actor;
use opendal::layers::RetryLayer;
use opendal::{Operator, Scheme};
use std::collections::HashMap;
use std::path::Path;
use xxhash_rust::xxh3::xxh3_64;

use crate::chunk::DABatchV0;
use rooch_config::da_config::{DAServerOpenDAConfig, OpenDAScheme};

use crate::messages::PutBatchInternalDAMessage;
use crate::segment::{Segment, SegmentID, SegmentV0, SEGMENT_V0_CHECKSUM_OFFSET};

pub struct DAServerOpenDAActor {
    max_segment_size: usize,
    operator: Operator,
}

// TODO get request and response
// 1. get by block number
// 2. get by batch hash
// 3. pull by stream
//

impl Actor for DAServerOpenDAActor {}

// TODO add FEC get for SDC protection (wrong response attacks)
impl DAServerOpenDAActor {
    pub async fn new(cfg: &DAServerOpenDAConfig) -> Result<DAServerOpenDAActor> {
        let mut config = cfg.clone();

        let op: Operator = match config.scheme {
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
                insert_default_from_env_or_const(
                    &mut config.config,
                    "default_storage_class",
                    "OPENDA_GCS_DEFAULT_STORAGE_CLASS",
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
            max_segment_size: cfg.max_segment_size.unwrap_or(4 * 1024 * 1024) as usize,
            operator: op,
        })
    }

    pub async fn pub_batch(&self, batch: PutBatchInternalDAMessage) -> Result<()> {
        // TODO using chunk builder to make segments:
        // 1. persist batch into buffer then return ok
        // 2. collect batch for better compression ratio
        // 3. split chunk into segments
        // 4. submit segments to celestia node
        // 5. record segment id in order
        // 6. clean up batch buffer

        // TODO more chunk version supports
        let chunk = DABatchV0 {
            version: 0,
            block_number: batch.batch.block_number,
            batch_hash: batch.batch.batch_hash,
            data: batch.batch.data,
        };
        let chunk_bytes = bcs::to_bytes(&chunk).unwrap();
        let segs = chunk_bytes.chunks(self.max_segment_size);
        let total = segs.len();

        // TODO explain why block number is a good idea: easy to get next block number for segments, then we could request chunk by block number
        let chunk_id = batch.batch.block_number;
        let segments = segs
            .enumerate()
            .map(|(i, data)| {
                SegmentV0 {
                    id: SegmentID {
                        chunk_id,
                        segment_number: i as u64,
                    },
                    is_last: i == total - 1, // extra info overhead is much smaller than max_block_size - max_segment_size
                    data_checksum: 0,
                    checksum: 0,
                    data: data.to_vec(),
                }
            })
            .collect::<Vec<_>>();

        for mut segment in segments {
            segment.data_checksum = xxh3_64(&segment.data);

            let mut bytes = segment.to_bytes();

            let fields = &bytes[0..SEGMENT_V0_CHECKSUM_OFFSET];
            segment.checksum = xxh3_64(fields);

            bytes.splice(
                SEGMENT_V0_CHECKSUM_OFFSET..SEGMENT_V0_CHECKSUM_OFFSET + 8,
                segment.checksum.to_le_bytes().iter().cloned(),
            );

            // TODO record ok segment in order
            // TODO segment indexer trait (local file, db, etc)
            self.operator.write(&segment.id.to_string(), bytes).await?; // TODO retry logic
        }

        Ok(())
    }
}

fn insert_default_from_env_or_const(
    config: &mut HashMap<String, String>,
    key: &str,
    env_var: &str,
    const_default: &str,
) {
    if !config.contains_key(key) {
        let value = std::env::var(env_var).unwrap_or(const_default.to_string());
        config.insert(key.to_string(), value);
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
    let max_times = max_retry_times.unwrap_or(4);
    op = op.layer(RetryLayer::new().with_max_times(max_times));
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
