// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::actor::indexer::IndexerActor;
use crate::actor::messages::{
    IndexerEventsMessage, IndexerTransactionMessage, QueryIndexerEventsMessage,
    QueryTransactionsByHashMessage,
};
use anyhow::Result;
use coerce::actor::ActorRef;
use moveos_types::h256::H256;
use moveos_types::moveos_std::event::Event;
use moveos_types::transaction::{TransactionExecutionInfo, VerifiedMoveOSTransaction};
use rooch_types::indexer::event_filter::{EventFilter, IndexerEvent, IndexerEventID};
use rooch_types::transaction::{TransactionSequenceInfo, TransactionWithInfo, TypedTransaction};

#[derive(Clone)]
pub struct IndexerProxy {
    pub actor: ActorRef<IndexerActor>,
}

impl IndexerProxy {
    pub fn new(actor: ActorRef<IndexerActor>) -> Self {
        Self { actor }
    }

    pub async fn indexer_transaction(
        &self,
        transaction: TypedTransaction,
        sequence_info: TransactionSequenceInfo,
        execution_info: TransactionExecutionInfo,
        moveos_tx: VerifiedMoveOSTransaction,
    ) -> Result<()> {
        self.actor
            .send(IndexerTransactionMessage {
                transaction,
                sequence_info,
                execution_info,
                moveos_tx,
            })
            .await?
    }

    pub async fn indexer_events(
        &self,
        events: Vec<Event>,
        transaction: TypedTransaction,
        sequence_info: TransactionSequenceInfo,
        moveos_tx: VerifiedMoveOSTransaction,
    ) -> Result<()> {
        self.actor
            .send(IndexerEventsMessage {
                events,
                transaction,
                sequence_info,
                moveos_tx,
            })
            .await?
    }

    pub async fn query_transactions_by_hash(
        &self,
        tx_hashes: Vec<H256>,
    ) -> Result<Vec<Option<TransactionWithInfo>>> {
        self.actor
            .send(QueryTransactionsByHashMessage { tx_hashes })
            .await?
    }

    pub async fn query_events(
        &self,
        filter: EventFilter,
        // exclusive cursor if `Some`, otherwise start from the beginning
        cursor: Option<IndexerEventID>,
        limit: usize,
        descending_order: bool,
    ) -> Result<Vec<IndexerEvent>> {
        self.actor
            .send(QueryIndexerEventsMessage {
                filter,
                cursor,
                limit,
                descending_order,
            })
            .await?
    }
}
