// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::actor::messages::{
    QueryIndexerEventsMessage, QueryIndexerFieldStatesMessage, QueryIndexerObjectStatesMessage,
    QueryIndexerTransactionsMessage,
};
use crate::indexer_reader::IndexerReader;
use anyhow::{anyhow, Result};
use async_trait::async_trait;
use coerce::actor::{context::ActorContext, message::Handler, Actor};
use rooch_types::indexer::event::IndexerEvent;
use rooch_types::indexer::state::{IndexerFieldState, IndexerObjectState};
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
impl Handler<QueryIndexerObjectStatesMessage> for IndexerReaderActor {
    async fn handle(
        &mut self,
        msg: QueryIndexerObjectStatesMessage,
        _ctx: &mut ActorContext,
    ) -> Result<Vec<IndexerObjectState>> {
        let QueryIndexerObjectStatesMessage {
            filter,
            cursor,
            limit,
            descending_order,
        } = msg;
        self.indexer_reader
            .query_object_states_with_filter(filter, cursor, limit, descending_order)
            .map_err(|e| anyhow!(format!("Failed to query indexer global states: {:?}", e)))
    }
}

#[async_trait]
impl Handler<QueryIndexerFieldStatesMessage> for IndexerReaderActor {
    async fn handle(
        &mut self,
        msg: QueryIndexerFieldStatesMessage,
        _ctx: &mut ActorContext,
    ) -> Result<Vec<IndexerFieldState>> {
        let QueryIndexerFieldStatesMessage {
            filter,
            cursor,
            limit,
            descending_order,
        } = msg;
        self.indexer_reader
            .query_field_states_with_filter(filter, cursor, limit, descending_order)
            .map_err(|e| anyhow!(format!("Failed to query indexer table states: {:?}", e)))
    }
}
