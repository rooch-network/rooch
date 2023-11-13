// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::actor::messages::{
    IndexerEventsMessage, IndexerTransactionMessage, QueryIndexerEventsMessage,
    QueryTransactionsByHashMessage,
};
use crate::indexer_reader::IndexerReader;
use crate::store::traits::IndexerStoreTrait;
use crate::types::{IndexedEvent, IndexedTransaction};
use crate::IndexerStore;
use anyhow::{anyhow, Result};
use async_trait::async_trait;
use coerce::actor::{context::ActorContext, message::Handler, Actor};
use rooch_types::indexer::event_filter::IndexerEvent;
use rooch_types::transaction::TransactionWithInfo;

pub struct IndexerActor {
    indexer_store: IndexerStore,
    indexer_reader: IndexerReader,
}

impl IndexerActor {
    pub fn new(indexer_store: IndexerStore, indexer_reader: IndexerReader) -> Result<Self> {
        Ok(Self {
            indexer_store,
            indexer_reader,
        })
    }
}

impl Actor for IndexerActor {}

#[async_trait]
impl Handler<IndexerTransactionMessage> for IndexerActor {
    async fn handle(
        &mut self,
        msg: IndexerTransactionMessage,
        _ctx: &mut ActorContext,
    ) -> Result<()> {
        let IndexerTransactionMessage {
            transaction,
            sequence_info,
            execution_info,
            moveos_tx,
        } = msg;

        let indexed_transaction =
            IndexedTransaction::new(transaction, sequence_info, execution_info, moveos_tx)?;
        let _transactions = vec![indexed_transaction];
        //TODO Open after supporting automatic creation of sqlite schema
        // self.indexer_store.persist_transactions(transactions)?;
        Ok(())
    }
}

#[async_trait]
impl Handler<IndexerEventsMessage> for IndexerActor {
    async fn handle(&mut self, msg: IndexerEventsMessage, _ctx: &mut ActorContext) -> Result<()> {
        let IndexerEventsMessage {
            events,
            transaction,
            sequence_info,
            moveos_tx,
        } = msg;

        let _events: Vec<_> = events
            .into_iter()
            .map(|event| {
                IndexedEvent::new(
                    event,
                    transaction.clone(),
                    sequence_info.clone(),
                    moveos_tx.clone(),
                )
            })
            .collect();
        //TODO Open after supporting automatic creation of sqlite schema
        // self.indexer_store.persist_events(events)?;
        Ok(())
    }
}

#[async_trait]
impl Handler<QueryTransactionsByHashMessage> for IndexerActor {
    async fn handle(
        &mut self,
        msg: QueryTransactionsByHashMessage,
        _ctx: &mut ActorContext,
    ) -> Result<Vec<Option<TransactionWithInfo>>> {
        self.indexer_store
            .query_transactions_by_hash(msg.tx_hashes)
            .map_err(|e| anyhow!(format!("Failed to query transactions by hash: {:?}", e)))
    }
}

#[async_trait]
impl Handler<QueryIndexerEventsMessage> for IndexerActor {
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
