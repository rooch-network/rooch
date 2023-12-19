// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use coerce::actor::ActorRef;

use crate::messages::{PutBatchMessage, PutBatchResult};
use crate::server::celestia::actor::server::DAServerCelestiaActor;

#[derive(Clone)]
pub struct DAServerCelestiaProxy {
    pub actor: ActorRef<DAServerCelestiaActor>,
}

impl DAServerCelestiaProxy {
    pub fn new(actor: ActorRef<DAServerCelestiaActor>) -> Self {
        Self { actor }
    }

    pub async fn submit_batch(&self, msg: PutBatchMessage) -> anyhow::Result<PutBatchResult> {
        self.actor.send(msg).await?
    }
}
