// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::actor::{messages::TransactionSequenceMessage, sequencer::SequencerActor};
use anyhow::Result;
use coerce::actor::ActorRef;
use rooch_types::transaction::{TransactionSequenceInfo, TypedTransaction};

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
}
