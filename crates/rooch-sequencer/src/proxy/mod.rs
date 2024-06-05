// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::messages::{
    GetSequencerOrderMessage, GetTransactionByHashMessage, GetTransactionsByHashMessage,
    GetTxHashsMessage,
};
use crate::{actor::sequencer::SequencerActor, messages::TransactionSequenceMessage};
use anyhow::Result;
use coerce::actor::ActorRef;
use moveos_types::h256::H256;
use rooch_types::transaction::{LedgerTransaction, LedgerTxData};

#[derive(Clone)]
pub struct SequencerProxy {
    pub actor: ActorRef<SequencerActor>,
}

impl SequencerProxy {
    pub fn new(actor: ActorRef<SequencerActor>) -> Self {
        Self { actor }
    }

    pub async fn sequence_transaction(&self, tx: LedgerTxData) -> Result<LedgerTransaction> {
        self.actor.send(TransactionSequenceMessage { tx }).await?
    }

    pub async fn get_transaction_by_hash(&self, hash: H256) -> Result<Option<LedgerTransaction>> {
        self.actor
            .send(GetTransactionByHashMessage { hash })
            .await?
    }

    pub async fn get_transactions_by_hash(
        &self,
        tx_hashes: Vec<H256>,
    ) -> Result<Vec<Option<LedgerTransaction>>> {
        self.actor
            .send(GetTransactionsByHashMessage { tx_hashes })
            .await?
    }

    pub async fn get_tx_hashs(&self, tx_orders: Vec<u64>) -> Result<Vec<Option<H256>>> {
        self.actor.send(GetTxHashsMessage { tx_orders }).await?
    }

    pub async fn get_sequencer_order(&self) -> Result<u64> {
        self.actor.send(GetSequencerOrderMessage {}).await?
    }
}
