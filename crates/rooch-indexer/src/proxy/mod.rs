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
use moveos_types::moveos_std::tx_context::TxContext;
use moveos_types::state::StateChangeSet;
use moveos_types::transaction::{MoveAction, TransactionExecutionInfo, VerifiedMoveOSTransaction};
use rooch_types::indexer::event::{EventFilter, IndexerEvent, IndexerEventID};
use rooch_types::indexer::state::{
    FieldStateFilter, IndexerFieldState, IndexerObjectState, IndexerStateID, ObjectStateFilter,
};
use rooch_types::indexer::transaction::{IndexerTransaction, TransactionFilter};
use rooch_types::transaction::LedgerTransaction;

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
        ledger_transaction: LedgerTransaction,
        execution_info: TransactionExecutionInfo,
        moveos_tx: VerifiedMoveOSTransaction,
        events: Vec<Event>,
        state_change_set: StateChangeSet,
    ) -> Result<()> {
        self.actor
            .send(UpdateIndexerMessage {
                root,
                ledger_transaction,
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
        tx_timestamp: u64,
        state_change_set: StateChangeSet,
    ) -> Result<()> {
        self.actor
            .send(IndexerStatesMessage {
                root,
                tx_order,
                tx_timestamp,
                state_change_set,
            })
            .await?
    }

    pub async fn indexer_transaction(
        &self,
        ledger_transaction: LedgerTransaction,
        execution_info: TransactionExecutionInfo,
        move_action: MoveAction,
        tx_context: TxContext,
    ) -> Result<()> {
        self.actor
            .send(IndexerTransactionMessage {
                ledger_transaction,
                execution_info,
                move_action,
                tx_context,
            })
            .await?
    }

    pub async fn indexer_events(
        &self,
        events: Vec<Event>,
        ledger_transaction: LedgerTransaction,
        tx_context: TxContext,
    ) -> Result<()> {
        self.actor
            .send(IndexerEventsMessage {
                events,
                ledger_transaction,
                tx_context,
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
    ) -> Result<Vec<IndexerTransaction>> {
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
