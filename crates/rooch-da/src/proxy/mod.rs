// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::actor::messages::PutDABatchMessage;
use crate::actor::server::DAServerActor;
use coerce::actor::ActorRef;

#[derive(Clone)]
pub struct DAServerProxy {
    pub actor: ActorRef<DAServerActor>,
}

impl DAServerProxy {
    pub fn new(actor: ActorRef<DAServerActor>) -> Self {
        Self { actor }
    }

    pub async fn pub_batch(&self, msg: PutDABatchMessage) -> anyhow::Result<()> {
        self.actor.send(msg).await?
    }
}
