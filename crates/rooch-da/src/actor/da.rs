// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use anyhow::Result;
use async_trait::async_trait;
use coerce::actor::context::ActorContext;
use coerce::actor::message::Handler;

use crate::messages::Batch;
use crate::server::serverproxy::DAServerProxy;

// TODO tx buffer for building batch
pub struct DAActor {
    servers: Vec<Box<dyn DAServerProxy>>,
}

impl DAActor {
    pub fn new(servers: Vec<Box<dyn DAServerProxy>>) -> Self {
        Self { servers }
    }

    pub fn submit_batch(&self, batch: Batch) -> Result<()> {
        // TODO calc checksum
        // TODO richer policy for multi servers
        for server in self.servers.iter() {
            // TODO verify checksum
            // TODO retry policy & log
            server.put_batch(batch.clone())?;
        }
        Ok(())
    }
}

#[async_trait]
impl Handler<Batch> for DAActor {
    async fn handle(&mut self, msg: Batch, _ctx: &mut ActorContext) -> Result<()> {
        self.submit_batch(msg)
    }
}
