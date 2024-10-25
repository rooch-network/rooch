// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::actor::messages::{GetServerStatusMessage, PutDABatchMessage};
use crate::actor::server::DAServerActor;
use coerce::actor::ActorRef;
use rooch_types::da;
use rooch_types::da::batch::SignedDABatchMeta;

#[derive(Clone)]
pub struct DAServerProxy {
    pub actor: ActorRef<DAServerActor>,
}

impl DAServerProxy {
    pub fn new(actor: ActorRef<DAServerActor>) -> Self {
        Self { actor }
    }

    pub async fn get_status(&self) -> anyhow::Result<da::state::ServerStatus> {
        self.actor.send(GetServerStatusMessage {}).await?
    }

    pub async fn pub_batch(&self, msg: PutDABatchMessage) -> anyhow::Result<SignedDABatchMeta> {
        self.actor.send(msg).await?
    }
}
