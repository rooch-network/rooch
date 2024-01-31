// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use async_trait::async_trait;
use coerce::actor::ActorRef;

use crate::messages::PutBatchInternalDAMessage;
use crate::server::celestia::actor::server::DAServerCelestiaActor;
use crate::server::serverproxy::DAServerProxy;

#[derive(Clone)]
pub struct DAServerCelestiaProxy {
    pub actor: ActorRef<DAServerCelestiaActor>,
}

impl DAServerCelestiaProxy {
    pub fn new(actor: ActorRef<DAServerCelestiaActor>) -> Self {
        Self { actor }
    }

    pub async fn submit_batch(&self, msg: PutBatchInternalDAMessage) -> anyhow::Result<()> {
        self.actor.send(msg).await?
    }
}

#[async_trait]
impl DAServerProxy for DAServerCelestiaProxy {
    async fn public_batch(&self, msg: PutBatchInternalDAMessage) -> anyhow::Result<()> {
        self.submit_batch(msg).await
    }
}
