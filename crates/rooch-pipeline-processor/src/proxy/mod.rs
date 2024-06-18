// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::actor::{
    messages::{ExecuteL1BlockMessage, ExecuteL1TxMessage, ExecuteL2TxMessage},
    processor::PipelineProcessorActor,
};
use anyhow::Result;
use coerce::actor::ActorRef;
use moveos_types::moveos_std::tx_context::TxContext;
use rooch_types::{
    address::BitcoinAddress,
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
        ctx: TxContext,
        tx: L1BlockWithBody,
        sequencer_address: BitcoinAddress,
    ) -> Result<ExecuteTransactionResponse> {
        self.actor
            .send(ExecuteL1BlockMessage {
                ctx,
                tx,
                sequencer_address,
            })
            .await?
    }

    pub async fn execute_l1_tx(
        &self,
        ctx: TxContext,
        tx: L1Transaction,
        sequencer_address: BitcoinAddress,
    ) -> Result<ExecuteTransactionResponse> {
        self.actor
            .send(ExecuteL1TxMessage {
                ctx,
                tx,
                sequencer_address,
            })
            .await?
    }
}

impl From<ActorRef<PipelineProcessorActor>> for PipelineProcessorProxy {
    fn from(actor: ActorRef<PipelineProcessorActor>) -> Self {
        Self::new(actor)
    }
}
