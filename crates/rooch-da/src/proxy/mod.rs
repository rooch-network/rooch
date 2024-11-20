// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::actor::messages::{AppendTransactionMessage, GetServerStatusMessage};
use crate::actor::server::DAServerActor;
use coerce::actor::ActorRef;
use rooch_types::da::state::DAServerStatus;

#[derive(Clone)]
pub struct DAServerProxy {
    pub actor: ActorRef<DAServerActor>,
}

impl DAServerProxy {
    pub fn new(actor: ActorRef<DAServerActor>) -> Self {
        Self { actor }
    }

    pub async fn get_status(&self) -> anyhow::Result<DAServerStatus> {
        self.actor.send(GetServerStatusMessage {}).await?
    }

    pub async fn append_tx(&self, msg: AppendTransactionMessage) -> anyhow::Result<()> {
        self.actor.send(msg).await?
    }
}
