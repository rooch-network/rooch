// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use anyhow::Result;
use coerce::actor::message::Message;
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
use serde::{Deserialize, Serialize};

/// Indexer write Message
#[derive(Debug, Clone)]
pub struct UpdateIndexerMessage {
    pub root: RootObjectEntity,
    pub ledger_transaction: LedgerTransaction,
    pub execution_info: TransactionExecutionInfo,
    pub moveos_tx: VerifiedMoveOSTransaction,
    pub events: Vec<Event>,
    pub state_change_set: StateChangeSet,
}

impl Message for UpdateIndexerMessage {
    type Result = Result<()>;
}

/// Indexer Transaction write Message
#[derive(Debug, Serialize, Deserialize)]
pub struct IndexerTransactionMessage {
    pub ledger_transaction: LedgerTransaction,
    pub execution_info: TransactionExecutionInfo,
    pub move_action: MoveAction,
    pub tx_context: TxContext,
}

impl Message for IndexerTransactionMessage {
    type Result = Result<()>;
}

/// Indexer Event write Message
#[derive(Debug, Serialize, Deserialize)]
pub struct IndexerEventsMessage {
    pub events: Vec<Event>,
    pub ledger_transaction: LedgerTransaction,
    pub tx_context: TxContext,
}

impl Message for IndexerEventsMessage {
    type Result = Result<()>;
}

/// Indexer State write Message
#[derive(Debug, Clone)]
pub struct IndexerStatesMessage {
    pub root: RootObjectEntity,
    pub tx_order: u64,
    pub tx_timestamp: u64,
    pub state_change_set: StateChangeSet,
}

impl Message for IndexerStatesMessage {
    type Result = Result<()>;
}

/// Query Indexer Transactions Message
#[derive(Debug, Serialize, Deserialize)]
pub struct QueryIndexerTransactionsMessage {
    pub filter: TransactionFilter,
    // exclusive cursor if `Some`, otherwise start from the beginning
    pub cursor: Option<u64>,
    pub limit: usize,
    pub descending_order: bool,
}

impl Message for QueryIndexerTransactionsMessage {
    type Result = Result<Vec<IndexerTransaction>>;
}

/// Query Indexer Events Message
#[derive(Debug, Serialize, Deserialize)]
pub struct QueryIndexerEventsMessage {
    pub filter: EventFilter,
    // exclusive cursor if `Some`, otherwise start from the beginning
    pub cursor: Option<IndexerEventID>,
    pub limit: usize,
    pub descending_order: bool,
}

impl Message for QueryIndexerEventsMessage {
    type Result = Result<Vec<IndexerEvent>>;
}

/// Query Indexer Global States Message
#[derive(Debug, Serialize, Deserialize)]
pub struct QueryIndexerObjectStatesMessage {
    pub filter: ObjectStateFilter,
    // exclusive cursor if `Some`, otherwise start from the beginning
    pub cursor: Option<IndexerStateID>,
    pub limit: usize,
    pub descending_order: bool,
}

impl Message for QueryIndexerObjectStatesMessage {
    type Result = Result<Vec<IndexerObjectState>>;
}

/// Query Indexer Table States Message
#[derive(Debug, Serialize, Deserialize)]
pub struct QueryIndexerFieldStatesMessage {
    pub filter: FieldStateFilter,
    // exclusive cursor if `Some`, otherwise start from the beginning
    pub cursor: Option<IndexerStateID>,
    pub limit: usize,
    pub descending_order: bool,
}

impl Message for QueryIndexerFieldStatesMessage {
    type Result = Result<Vec<IndexerFieldState>>;
}
