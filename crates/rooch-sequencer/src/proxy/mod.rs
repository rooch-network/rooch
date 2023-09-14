// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::messages::{GetTransactionByHashMessage, GetTransactionsByHashMessage};
use crate::{actor::sequencer::SequencerActor, messages::TransactionSequenceMessage};
use anyhow::Result;
use coerce::actor::ActorRef;
use rooch_types::transaction::TypedTransaction;
use rooch_types::{transaction::TransactionSequenceInfo, H256};

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
        self.actor
            .send(GetTransactionByHashMessage { hash })
            .await?
    }

    pub async fn get_transactions_by_hash(
        &self,
        tx_hashes: Vec<H256>,
    ) -> Result<Vec<Option<TypedTransaction>>> {
        self.actor
            .send(GetTransactionsByHashMessage { tx_hashes })
            .await?
    }
}
