// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::messages::PutBatchInternalDAMessage;
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

    pub async fn submit_batch(&self, msg: PutBatchInternalDAMessage) -> anyhow::Result<()> {
        self.actor.send(msg).await?
    }
}

#[async_trait]
impl DAServerProxy for DAServerOpenDAProxy {
    async fn public_batch(&self, msg: PutBatchInternalDAMessage) -> anyhow::Result<()> {
        self.submit_batch(msg).await
    }
}
