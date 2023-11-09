// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use anyhow::{anyhow, Result};
use async_trait::async_trait;
use coerce::actor::{context::ActorContext, message::Handler, Actor};
// use tracing::info;
use crate::actor::messages::{IndexerTransactionMessage, QueryTransactionsByHashMessage};
use crate::store::traits::IndexerStoreTrait;
use crate::types::IndexedTransaction;
use crate::IndexerStore;
use rooch_types::transaction::TransactionWithInfo;

pub struct IndexerActor {
    indexer_store: IndexerStore,
}

impl IndexerActor {
    pub fn new(indexer_store: IndexerStore) -> Result<Self> {
        Ok(Self { indexer_store })
    }
}

impl Actor for IndexerActor {}

#[async_trait]
impl Handler<IndexerTransactionMessage> for IndexerActor {
    async fn handle(
        &mut self,
        msg: IndexerTransactionMessage,
        _ctx: &mut ActorContext,
    ) -> Result<IndexedTransaction> {
        let IndexerTransactionMessage {
            transaction,
            sequence_info,
            execution_info,
            moveos_tx,
        } = msg;

        let indexed_transaction =
            IndexedTransaction::new(transaction, sequence_info, execution_info, moveos_tx)?;
        let _transactions = vec![indexed_transaction.clone()];
        // self.indexer_store.persist_transactions(transactions)?;
        Ok(indexed_transaction)
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
