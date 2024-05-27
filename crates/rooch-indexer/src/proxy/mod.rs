// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::actor::indexer::IndexerActor;
use crate::actor::messages::{
    IndexerEventsMessage, IndexerStatesMessage, IndexerTransactionMessage,
    QueryIndexerEventsMessage, QueryIndexerFieldStatesMessage, QueryIndexerObjectStatesMessage,
    QueryIndexerTransactionsMessage, UpdateIndexerMessage,
};
use crate::actor::reader_indexer::IndexerReaderActor;
use anyhow::Result;
use coerce::actor::ActorRef;
use moveos_types::moveos_std::event::Event;
use moveos_types::moveos_std::object::RootObjectEntity;
use moveos_types::state::StateChangeSet;
use moveos_types::transaction::{TransactionExecutionInfo, VerifiedMoveOSTransaction};
use rooch_types::indexer::event::{EventFilter, IndexerEvent, IndexerEventID};
use rooch_types::indexer::state::{
    FieldStateFilter, IndexerFieldState, IndexerObjectState, IndexerStateID, ObjectStateFilter,
};
use rooch_types::indexer::transaction::TransactionFilter;
use rooch_types::transaction::LedgerTransaction;
use rooch_types::transaction::TransactionWithInfo;

#[derive(Clone)]
pub struct IndexerProxy {
    pub actor: ActorRef<IndexerActor>,
    pub reader_actor: ActorRef<IndexerReaderActor>,
}

impl IndexerProxy {
    pub fn new(actor: ActorRef<IndexerActor>, reader_actor: ActorRef<IndexerReaderActor>) -> Self {
        Self {
            actor,
            reader_actor,
        }
    }

    pub async fn update_indexer(
        &self,
        root: RootObjectEntity,
        transaction: LedgerTransaction,
        execution_info: TransactionExecutionInfo,
        moveos_tx: VerifiedMoveOSTransaction,
        events: Vec<Event>,
        state_change_set: StateChangeSet,
    ) -> Result<()> {
        self.actor
            .send(UpdateIndexerMessage {
                root,
                transaction,
                execution_info,
                moveos_tx,
                events,
                state_change_set,
            })
            .await?
    }

    pub async fn indexer_states(
        &self,
        root: RootObjectEntity,
        tx_order: u64,
        state_change_set: StateChangeSet,
    ) -> Result<()> {
        self.actor
            .send(IndexerStatesMessage {
                root,
                tx_order,
                state_change_set,
            })
            .await?
    }

    pub async fn indexer_transaction(
        &self,
        transaction: LedgerTransaction,
        execution_info: TransactionExecutionInfo,
        moveos_tx: VerifiedMoveOSTransaction,
    ) -> Result<()> {
        self.actor
            .send(IndexerTransactionMessage {
                transaction,
                execution_info,
                moveos_tx,
            })
            .await?
    }

    pub async fn indexer_events(
        &self,
        events: Vec<Event>,
        transaction: LedgerTransaction,
        moveos_tx: VerifiedMoveOSTransaction,
    ) -> Result<()> {
        self.actor
            .send(IndexerEventsMessage {
                events,
                transaction,
                moveos_tx,
            })
            .await?
    }

    pub async fn query_transactions(
        &self,
        filter: TransactionFilter,
        // exclusive cursor if `Some`, otherwise start from the beginning
        cursor: Option<u64>,
        limit: usize,
        descending_order: bool,
    ) -> Result<Vec<TransactionWithInfo>> {
        self.reader_actor
            .send(QueryIndexerTransactionsMessage {
                filter,
                cursor,
                limit,
                descending_order,
            })
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
        self.reader_actor
            .send(QueryIndexerEventsMessage {
                filter,
                cursor,
                limit,
                descending_order,
            })
            .await?
    }

    pub async fn query_object_states(
        &self,
        filter: ObjectStateFilter,
        // exclusive cursor if `Some`, otherwise start from the beginning
        cursor: Option<IndexerStateID>,
        limit: usize,
        descending_order: bool,
    ) -> Result<Vec<IndexerObjectState>> {
        self.reader_actor
            .send(QueryIndexerObjectStatesMessage {
                filter,
                cursor,
                limit,
                descending_order,
            })
            .await?
    }

    pub async fn query_field_states(
        &self,
        filter: FieldStateFilter,
        // exclusive cursor if `Some`, otherwise start from the beginning
        cursor: Option<IndexerStateID>,
        limit: usize,
        descending_order: bool,
    ) -> Result<Vec<IndexerFieldState>> {
        self.reader_actor
            .send(QueryIndexerFieldStatesMessage {
                filter,
                cursor,
                limit,
                descending_order,
            })
            .await?
    }
}
