// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use anyhow::Result;
use async_trait::async_trait;
use coerce::actor::context::ActorContext;
use coerce::actor::message::Handler;
use coerce::actor::Actor;

use crate::messages::{Batch, PutBatchMessage};
use crate::server::serverproxy::DAServerProxy;
use std::sync::{Arc, RwLock};

// TODO tx buffer for building batch
pub struct DAActor {
    servers: Arc<RwLock<Vec<Arc<dyn DAServerProxy + Send + Sync>>>>,
}

impl Actor for DAActor {}

impl DAActor {
    pub fn new(servers: Vec<Arc<dyn DAServerProxy + Send + Sync>>) -> Self {
        Self {
            servers: Arc::new(RwLock::new(servers)),
        }
    }

    pub async fn submit_batch(&self, batch: Batch) -> Result<()> {
        // TODO calc checksum
        // TODO richer policy for multi servers
        // TODO verify checksum
        // TODO retry policy & log

        let servers = self.servers.read().unwrap().to_vec();

        let futures: Vec<_> = servers
            .iter()
            .map(|server| {
                let server = Arc::clone(server);
                let batch = batch.clone();
                async move {
                    server
                        .put_batch(PutBatchMessage {
                            batch: batch.clone(),
                        })
                        .await
                }
            })
            .collect();

        for future in futures {
            future.await?;
        }
        Ok(())
    }
}

#[async_trait]
impl Handler<Batch> for DAActor {
    async fn handle(&mut self, msg: Batch, _ctx: &mut ActorContext) -> Result<()> {
        self.submit_batch(msg).await
    }
}
