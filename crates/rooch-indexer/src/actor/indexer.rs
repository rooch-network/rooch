// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::actor::messages::{
    IndexerEventsMessage, IndexerStatesMessage, IndexerTransactionMessage,
    QueryIndexerEventsMessage, QueryIndexerTransactionsMessage,
};
use crate::indexer_reader::IndexerReader;
use crate::store::traits::IndexerStoreTrait;
use crate::types::{IndexedEvent, IndexedTransaction};
use crate::IndexerStore;
use anyhow::{anyhow, Result};
use async_trait::async_trait;
use coerce::actor::{context::ActorContext, message::Handler, Actor};
use moveos_store::MoveOSStore;
// use moveos_types::moveos_std::context;
use moveos_types::state_resolver::MoveOSResolverProxy;
use rooch_types::indexer::event_filter::IndexerEvent;
use rooch_types::transaction::TransactionWithInfo;

pub struct IndexerActor {
    indexer_store: IndexerStore,
    indexer_reader: IndexerReader,
    moveos_store: MoveOSResolverProxy<MoveOSStore>,
}

impl IndexerActor {
    pub fn new(
        indexer_store: IndexerStore,
        indexer_reader: IndexerReader,
        moveos_store: MoveOSStore,
    ) -> Result<Self> {
        Ok(Self {
            indexer_store,
            indexer_reader,
            moveos_store: MoveOSResolverProxy(moveos_store),
        })
    }
}

impl Actor for IndexerActor {}

#[async_trait]
impl Handler<IndexerStatesMessage> for IndexerActor {
    async fn handle(&mut self, msg: IndexerStatesMessage, _ctx: &mut ActorContext) -> Result<()> {
        let IndexerStatesMessage { state_change_set } = msg;

        // let indexed_transaction =
        //     IndexedTransaction::new(transaction, sequence_info, execution_info, moveos_tx)?;
        //
        // updates: BTreeMap<K, Option<V>>,

        // #[derive(Default, Clone, Debug)]
        // pub struct StateChangeSet {
        //     pub new_tables: BTreeMap<ObjectID, TableTypeInfo>,
        //     pub removed_tables: BTreeSet<ObjectID>,
        //     pub changes: BTreeMap<ObjectID, TableChange>,
        // }

        // for (table_handle, table_change) in state_change_set.changes {
        //     // handle global object
        //     if table_handle == context::GLOBAL_OBJECT_STORAGE_HANDLE {
        //         self.global_table
        //             .put_changes(table_change.entries.into_iter())?;
        //         // TODO: do we need to update the size of global table?
        //     } else {
        //         let (mut object, table) = self.get_as_table_or_create(table_handle)?;
        //         let new_state_root = table.put_changes(table_change.entries.into_iter())?;
        //         object.value.state_root = AccountAddress::new(new_state_root.into());
        //         let curr_table_size: i64 = object.value.size as i64;
        //         let updated_table_size = curr_table_size + table_change.size_increment;
        //         debug_assert!(updated_table_size >= 0);
        //         object.value.size = updated_table_size as u64;
        //         changed_objects.put(table_handle.to_bytes(), object.into());
        //     }
        // }

        // for (table_handle, table_change) in state_change_set.changes {
        //     // handle global object
        //     if table_handle == context::GLOBAL_OBJECT_STORAGE_HANDLE {
        //         self.global_table
        //             .put_changes(table_change.entries.into_iter())?;
        //         // TODO: do we need to update the size of global table?
        //     } else {
        //         let (mut object, table) = self.get_as_table_or_create(table_handle)?;
        //         let new_state_root = table.put_changes(table_change.entries.into_iter())?;
        //         object.value.state_root = AccountAddress::new(new_state_root.into());
        //         let curr_table_size: i64 = object.value.size as i64;
        //         let updated_table_size = curr_table_size + table_change.size_increment;
        //         debug_assert!(updated_table_size >= 0);
        //         object.value.size = updated_table_size as u64;
        //         changed_objects.put(table_handle.to_bytes(), object.into());
        //     }
        // }

        // for table_handle in state_change_set.removed_tables {
        //     changed_objects.remove(table_handle.to_bytes());
        // }
        //
        // self.global_table.puts(changed_objects)
        //
        // pub fn put_resources(&self, modules: BTreeMap<StructTag, Op<Vec<u8>>>) -> Result<H256> {
        //     self.put_changes(modules.into_iter().map(|(k, v)| {
        //         (
        //             resource_tag_to_key(&k),
        //             v.map(|v| State::new(v, TypeTag::Struct(Box::new(k)))),
        //         )
        //     }))
        //
        //     let mut update_set = UpdateSet::new();
        //     for (key, op) in changes {
        //         match op {
        //             Op::Modify(value) => {
        //                 update_set.put(key, value);
        //             }
        //             Op::Delete => {
        //                 update_set.remove(key);
        //             }
        //             Op::New(value) => {
        //                 update_set.put(key, value);
        //             }
        //         }
        //     }
        //     self.puts(update_set)

        // let transactions = vec![indexed_transaction];
        // self.indexer_store.persist_global_states(global_states)?;
        Ok(())
    }
}

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
        let transactions = vec![indexed_transaction];
        self.indexer_store.persist_transactions(transactions)?;
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

        let events: Vec<_> = events
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
        self.indexer_store.persist_events(events)?;
        Ok(())
    }
}

#[async_trait]
impl Handler<QueryIndexerTransactionsMessage> for IndexerActor {
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
