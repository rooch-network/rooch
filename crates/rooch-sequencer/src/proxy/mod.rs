// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::messages::{
    GetSequencerOrderMessage, GetTransactionByHashMessage, GetTransactionsByHashMessage,
    GetTxSequenceInfoMappingByHashMessage, GetTxSequenceInfoMappingByOrderMessage,
    GetTxSequenceInfosMessage,
};
use crate::{actor::sequencer::SequencerActor, messages::TransactionSequenceMessage};
use anyhow::Result;
use coerce::actor::ActorRef;
use rooch_types::sequencer::SequencerOrder;
use rooch_types::transaction::{TransactionSequenceInfoMapping, TypedTransaction};
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

    pub async fn get_transaction_sequence_info_mapping_by_order(
        &self,
        tx_orders: Vec<u128>,
    ) -> Result<Vec<Option<TransactionSequenceInfoMapping>>> {
        self.actor
            .send(GetTxSequenceInfoMappingByOrderMessage { tx_orders })
            .await?
    }

    pub async fn get_transaction_sequence_info_mapping_by_hash(
        &self,
        tx_hashes: Vec<H256>,
    ) -> Result<Vec<Option<TransactionSequenceInfoMapping>>> {
        self.actor
            .send(GetTxSequenceInfoMappingByHashMessage { tx_hashes })
            .await?
    }

    pub async fn get_transaction_sequence_infos(
        &self,
        orders: Vec<u128>,
    ) -> Result<Vec<Option<TransactionSequenceInfo>>> {
        self.actor
            .send(GetTxSequenceInfosMessage { orders })
            .await?
    }

    pub async fn get_sequencer_order(&self) -> Result<Option<SequencerOrder>> {
        self.actor.send(GetSequencerOrderMessage {}).await?
    }
}
