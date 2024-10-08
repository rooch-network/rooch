// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::actor::{
    messages::{
        ExecuteL1BlockMessage, ExecuteL1TxMessage, ExecuteL2TxMessage, GetServiceStatusMessage,
    },
    processor::PipelineProcessorActor,
};
use anyhow::Result;
use coerce::actor::ActorRef;
use rooch_types::{
    service_status::ServiceStatus,
    transaction::{
        rooch::RoochTransaction, ExecuteTransactionResponse, L1BlockWithBody, L1Transaction,
    },
};

#[derive(Clone)]
pub struct PipelineProcessorProxy {
    pub actor: ActorRef<PipelineProcessorActor>,
}

impl PipelineProcessorProxy {
    pub fn new(actor: ActorRef<PipelineProcessorActor>) -> Self {
        Self { actor }
    }

    pub async fn execute_l2_tx(&self, tx: RoochTransaction) -> Result<ExecuteTransactionResponse> {
        self.actor.send(ExecuteL2TxMessage { tx }).await?
    }

    pub async fn execute_l1_block(
        &self,
        tx: L1BlockWithBody,
    ) -> Result<ExecuteTransactionResponse> {
        self.actor.send(ExecuteL1BlockMessage { tx }).await?
    }

    pub async fn execute_l1_tx(&self, tx: L1Transaction) -> Result<ExecuteTransactionResponse> {
        self.actor.send(ExecuteL1TxMessage { tx }).await?
    }

    pub async fn get_service_status(&self) -> Result<ServiceStatus> {
        self.actor.send(GetServiceStatusMessage {}).await?
    }
}

impl From<ActorRef<PipelineProcessorActor>> for PipelineProcessorProxy {
    fn from(actor: ActorRef<PipelineProcessorActor>) -> Self {
        Self::new(actor)
    }
}
