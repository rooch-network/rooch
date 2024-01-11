// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::messages::{PutBatchMessage, PutBatchResult};
use crate::server::openda::actor::server::DAServerOpenDAActor;
use crate::server::serverproxy::DAServerProxy;
use async_trait::async_trait;
use coerce::actor::ActorRef;

#[derive(Clone)]
pub struct DAServerOpenDAProxy {
    pub actor: ActorRef<DAServerOpenDAActor>,
}

impl DAServerOpenDAProxy {
    pub fn new(actor: ActorRef<DAServerOpenDAActor>) -> Self {
        Self { actor }
    }

    pub async fn submit_batch(&self, msg: PutBatchMessage) -> anyhow::Result<PutBatchResult> {
        self.actor.send(msg).await?
    }
}

#[async_trait]
impl DAServerProxy for DAServerOpenDAProxy {
    async fn put_batch(&self, msg: PutBatchMessage) -> anyhow::Result<PutBatchResult> {
        self.submit_batch(msg).await
    }
}
