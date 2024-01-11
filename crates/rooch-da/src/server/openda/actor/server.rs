// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use anyhow::Result;
use async_trait::async_trait;
use coerce::actor::context::ActorContext;
use coerce::actor::message::Handler;
use coerce::actor::Actor;
use opendal::{Operator, Scheme};

use rooch_config::da_config::{DAServerOpenDAConfig, OpenDAScheme};

use crate::messages::{PutBatchMessage, PutBatchResult};
use crate::server::segment::{Segment, SegmentID};

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
        let config = cfg.clone();

        let op: Operator = match config.scheme {
            OpenDAScheme::S3 => Operator::via_map(Scheme::S3, config.config)?,
            OpenDAScheme::GCS => Operator::via_map(Scheme::Gcs, config.config)?,
        };

        Ok(Self {
            max_segment_size: cfg.max_segment_size.unwrap() as usize,
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
                Segment {
                    id: SegmentID {
                        chunk_id,
                        segment_id: i as u64,
                    },
                    is_last: i == total - 1, // extra info overhead is much smaller than max_block_size - max_segment_size
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
