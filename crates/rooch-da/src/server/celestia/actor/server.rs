// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use anyhow::Result;
use async_trait::async_trait;
use celestia_types::nmt::Namespace;
use coerce::actor::context::ActorContext;
use coerce::actor::message::Handler;
use coerce::actor::Actor;

use crate::chunk::{Chunk, ChunkV0};
use rooch_config::da_config::DAServerCelestiaConfig;

use crate::messages::PutBatchInternalDAMessage;
use crate::server::celestia::backend::Backend;

pub struct DAServerCelestiaActor {
    max_segment_size: usize,
    backend: Backend,
}

impl Actor for DAServerCelestiaActor {}

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

    pub async fn public_batch(&self, batch_msg: PutBatchInternalDAMessage) -> Result<()> {
        let chunk: ChunkV0 = batch_msg.batch.into();
        let segments = chunk.to_segments(self.max_segment_size);
        for segment in segments {
            let result = self.backend.submit(segment).await?;
            log::info!(
                "submitted segment to celestia node, segment_id: {:?}, namespace: {:?}, commitment: {:?}, height: {}",
                result.segment_id,
                result.namespace,
                result.commitment,
                result.height,
            );
        }

        Ok(())
    }
}

#[async_trait]
impl Handler<PutBatchInternalDAMessage> for DAServerCelestiaActor {
    async fn handle(
        &mut self,
        msg: PutBatchInternalDAMessage,
        _ctx: &mut ActorContext,
    ) -> Result<()> {
        self.public_batch(msg).await
    }
}
