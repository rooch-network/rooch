// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::actor::messages::{
    QueryIndexerEventsMessage, QueryIndexerGlobalStatesMessage, QueryIndexerTableStatesMessage,
    QueryIndexerTransactionsMessage, SyncIndexerStatesMessage,
};
use crate::indexer_reader::IndexerReader;
use anyhow::{anyhow, Result};
use async_trait::async_trait;
use coerce::actor::{context::ActorContext, message::Handler, Actor};
use rooch_types::indexer::event_filter::IndexerEvent;
use rooch_types::indexer::state::{IndexerGlobalState, IndexerTableChangeSet, IndexerTableState};
use rooch_types::transaction::TransactionWithInfo;

pub struct IndexerReaderActor {
    indexer_reader: IndexerReader,
}

impl IndexerReaderActor {
    pub fn new(indexer_reader: IndexerReader) -> Result<Self> {
        Ok(Self { indexer_reader })
    }
}

impl Actor for IndexerReaderActor {}

#[async_trait]
impl Handler<QueryIndexerTransactionsMessage> for IndexerReaderActor {
    async fn handle(
        &mut self,
        msg: QueryIndexerTransactionsMessage,
        _ctx: &mut ActorContext,
    ) -> Result<Vec<TransactionWithInfo>> {
        let QueryIndexerTransactionsMessage {
            filter,
            cursor,
            limit,
            descending_order,
        } = msg;
        self.indexer_reader
            .query_transactions_with_filter(filter, cursor, limit, descending_order)
            .map_err(|e| anyhow!(format!("Failed to query indexer transactions: {:?}", e)))
    }
}

#[async_trait]
impl Handler<QueryIndexerEventsMessage> for IndexerReaderActor {
    async fn handle(
        &mut self,
        msg: QueryIndexerEventsMessage,
        _ctx: &mut ActorContext,
    ) -> Result<Vec<IndexerEvent>> {
        let QueryIndexerEventsMessage {
            filter,
            cursor,
            limit,
            descending_order,
        } = msg;
        self.indexer_reader
            .query_events_with_filter(filter, cursor, limit, descending_order)
            .map_err(|e| anyhow!(format!("Failed to query indexer events: {:?}", e)))
    }
}

#[async_trait]
impl Handler<QueryIndexerGlobalStatesMessage> for IndexerReaderActor {
    async fn handle(
        &mut self,
        msg: QueryIndexerGlobalStatesMessage,
        _ctx: &mut ActorContext,
    ) -> Result<Vec<IndexerGlobalState>> {
        let QueryIndexerGlobalStatesMessage {
            filter,
            cursor,
            limit,
            descending_order,
        } = msg;
        self.indexer_reader
            .query_global_states_with_filter(filter, cursor, limit, descending_order)
            .map_err(|e| anyhow!(format!("Failed to query indexer global states: {:?}", e)))
    }
}

#[async_trait]
impl Handler<QueryIndexerTableStatesMessage> for IndexerReaderActor {
    async fn handle(
        &mut self,
        msg: QueryIndexerTableStatesMessage,
        _ctx: &mut ActorContext,
    ) -> Result<Vec<IndexerTableState>> {
        let QueryIndexerTableStatesMessage {
            filter,
            cursor,
            limit,
            descending_order,
        } = msg;
        self.indexer_reader
            .query_table_states_with_filter(filter, cursor, limit, descending_order)
            .map_err(|e| anyhow!(format!("Failed to query indexer table states: {:?}", e)))
    }
}

#[async_trait]
impl Handler<SyncIndexerStatesMessage> for IndexerReaderActor {
    async fn handle(
        &mut self,
        msg: SyncIndexerStatesMessage,
        _ctx: &mut ActorContext,
    ) -> Result<Vec<IndexerTableChangeSet>> {
        let SyncIndexerStatesMessage {
            filter,
            cursor,
            limit,
            descending_order,
        } = msg;
        self.indexer_reader
            .sync_states(filter, cursor, limit, descending_order)
            .map_err(|e| {
                anyhow!(format!(
                    "Failed to query indexer state change sets: {:?}",
                    e
                ))
            })
    }
}
