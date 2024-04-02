// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::actor::{messages::ExecuteTransactionMessage, processor::PipelineProcessorActor};
use anyhow::Result;
use coerce::actor::ActorRef;
use rooch_types::transaction::{rooch::RoochTransaction, ExecuteTransactionResponse};

#[derive(Clone)]
pub struct PipelineProcessorProxy {
    pub actor: ActorRef<PipelineProcessorActor>,
}

impl PipelineProcessorProxy {
    pub fn new(actor: ActorRef<PipelineProcessorActor>) -> Self {
        Self { actor }
    }

    pub async fn execute_tx(&self, tx: RoochTransaction) -> Result<ExecuteTransactionResponse> {
        self.actor.send(ExecuteTransactionMessage { tx }).await?
    }
}

impl From<ActorRef<PipelineProcessorActor>> for PipelineProcessorProxy {
    fn from(actor: ActorRef<PipelineProcessorActor>) -> Self {
        Self::new(actor)
    }
}
