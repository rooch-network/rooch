// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use anyhow::Result;
use async_trait::async_trait;
use celestia_types::nmt::Namespace;
use coerce::actor::context::ActorContext;
use coerce::actor::message::Handler;
use coerce::actor::Actor;

use rooch_config::da_config::DAServerCelestiaConfig;

use crate::messages::{PutBatchMessage, PutBatchResult};
use crate::server::celestia::backend::Backend;
use crate::server::segment::{Segment, SegmentID};

pub struct DAServerCelestiaActor {
    max_segment_size: usize,
    backend: Backend,
}

// TODO get request and response
// 1. get by block number
// 2. get by batch hash
// 3. pull by stream
//

impl Actor for DAServerCelestiaActor {}

// TODO add FEC get for SDC protection (wrong response attacks)
impl DAServerCelestiaActor {
    pub async fn new(cfg: &DAServerCelestiaConfig) -> Self {
        let namespace_str = cfg.namespace.as_ref().unwrap().clone();
        let namespace: Namespace = serde_yaml::from_str(&namespace_str).unwrap();
        let conn_str = cfg.conn.as_ref().unwrap().clone();
        let token = cfg.auth_token.as_ref().unwrap().clone();

        Self {
            max_segment_size: cfg.max_segment_size.unwrap() as usize,
            backend: Backend::new(namespace, &conn_str, &token).await,
        }
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
            self.backend.submit(segment).await?;
        }
        Ok(PutBatchResult::default())
    }
}

#[async_trait]
impl Handler<PutBatchMessage> for DAServerCelestiaActor {
    async fn handle(
        &mut self,
        msg: PutBatchMessage,
        _ctx: &mut ActorContext,
    ) -> Result<PutBatchResult> {
        self.pub_batch(msg).await
    }
}
