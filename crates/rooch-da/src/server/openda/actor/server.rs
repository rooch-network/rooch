// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use anyhow::Result;
use async_trait::async_trait;
use coerce::actor::context::ActorContext;
use coerce::actor::message::Handler;
use coerce::actor::Actor;
use opendal::{Operator, Scheme};
use std::collections::HashMap;
use opendal::layers::RetryLayer;

use rooch_config::da_config::{DAServerOpenDAConfig, OpenDAScheme};

use crate::messages::{PutBatchMessage, PutBatchResult};
use crate::segment::{SegmentID, SegmentV0};

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
            OpenDAScheme::S3 => {
                new_retry_operator(Scheme::S3, config.config, None)?
            },
            OpenDAScheme::GCS => {
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
                insert_default_from_env_or_const(
                    &mut config.config,
                    "predefined_acl",
                    "OPENDA_GCS_PREDEFINED_ACL",
                    "publicRead",
                );
                insert_default_from_env_or_const(
                    &mut config.config,
                    "default_storage_class",
                    "OPENDA_GCS_DEFAULT_STORAGE_CLASS",
                    "STANDARD",
                );

                // After setting defaults, proceed with creating Operator
                new_retry_operator(Scheme::Gcs, config.config, None)?
            }
        };

        Ok(Self {
            max_segment_size: cfg.max_segment_size.unwrap_or(4 * 1024 * 1024) as usize,
            operator: op,
        })
    }

    pub async fn pub_batch(&self, batch: PutBatchMessage) -> Result<PutBatchResult> {
        // TODO using chunk builder to make segments:
        // 1. persist batch into buffer then return ok
        // 2. collect batch for better compression ratio
        // 3. split chunk into segments
        // 4. submit segments to celestia node
        // 5. record segment id in order
        // 6. clean up batch buffer
        let segs = batch.batch.data.chunks(self.max_segment_size);
        let total = segs.len();

        let chunk_id = batch.batch.meta.block_number;
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

        for segment in segments {
            // TODO record ok segment in order
            // TODO segment indexer trait (local file, db, etc)
            let data = bcs::to_bytes(&segment).unwrap();
            self.operator.write(&segment.id.to_string(), data).await?; // TODO retry logic
        }

        Ok(PutBatchResult::default())
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

fn new_retry_operator(
    scheme: Scheme,
    config: HashMap<String, String>,
    max_retry_times: Option<usize>,
) -> Result<Operator> {
    let mut op = Operator::via_map(scheme, config)?;
    let max_times = max_retry_times.unwrap_or(4);
    op = op.layer(RetryLayer::new().with_max_times(max_times));
    Ok(op)
}

#[async_trait]
impl Handler<PutBatchMessage> for DAServerOpenDAActor {
    async fn handle(
        &mut self,
        msg: PutBatchMessage,
        _ctx: &mut ActorContext,
    ) -> Result<PutBatchResult> {
        self.pub_batch(msg).await
    }
}
