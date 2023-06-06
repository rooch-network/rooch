// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::actor::{
    messages::{TransactionByHashMessage, TransactionByIndexMessage, TransactionSequenceMessage},
    sequencer::SequencerActor,
};
use anyhow::Result;
use coerce::actor::ActorRef;
use rooch_types::{
    transaction::{TransactionSequenceInfo, TypedTransaction},
    H256,
};

#[derive(Clone)]
pub struct SequencerProxy {
    pub actor: ActorRef<SequencerActor>,
}

impl SequencerProxy {
    pub fn new(actor: ActorRef<SequencerActor>) -> Self {
        Self { actor }
    }

    pub async fn sequence_transaction(
        &self,
        tx: TypedTransaction,
    ) -> Result<TransactionSequenceInfo> {
        self.actor.send(TransactionSequenceMessage { tx }).await?
    }

    pub async fn get_transaction_by_hash(&self, hash: H256) -> Result<Option<TypedTransaction>> {
        self.actor.send(TransactionByHashMessage { hash }).await?
    }

    pub async fn get_transaction_by_index(
        &self,
        start: u64,
        limit: u64,
    ) -> Result<Option<Vec<TypedTransaction>>> {
        self.actor
            .send(TransactionByIndexMessage { start, limit })
            .await?
    }
}
