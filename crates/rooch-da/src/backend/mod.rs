// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use async_trait::async_trait;
use coerce::actor::message::Message;
use rooch_types::da::batch::DABatch;
use rooch_types::da::chunk::Chunk;
use serde::{Deserialize, Serialize};

pub mod celestia;
pub mod openda;

#[async_trait]
pub trait DABackend: Sync + Send {
    async fn submit_batch(&self, batch: DABatch) -> anyhow::Result<()>;
}

// DABackendNopProxy is a no-op implementation of DABackendProxy
pub struct DABackendNopProxy;

#[async_trait]
impl DABackend for DABackendNopProxy {
    async fn submit_batch(&self, _batch: DABatch) -> anyhow::Result<()> {
        Ok(())
    }
}
