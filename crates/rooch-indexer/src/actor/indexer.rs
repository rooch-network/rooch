// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::actor::messages::{
    IndexerEventsMessage, IndexerStatesMessage, IndexerTransactionMessage,
    QueryIndexerEventsMessage, QueryIndexerGlobalStatesMessage, QueryIndexerTableStatesMessage,
    QueryIndexerTransactionsMessage, SyncIndexerStatesMessage,
};
use crate::indexer_reader::IndexerReader;
use crate::store::traits::IndexerStoreTrait;
use crate::types::{
    IndexedEvent, IndexedGlobalState, IndexedTableChangeSet, IndexedTableState, IndexedTransaction,
};
use crate::IndexerStore;
use anyhow::{anyhow, Result};
use async_trait::async_trait;
use coerce::actor::{context::ActorContext, message::Handler, Actor};
use move_core_types::effects::Op;
use move_resource_viewer::MoveValueAnnotator;
use moveos_store::MoveOSStore;
use moveos_types::moveos_std::context;
use moveos_types::moveos_std::object::{ObjectEntity, ObjectID, RawObject};
use moveos_types::moveos_std::raw_table::TableInfo;
use moveos_types::state::{SplitStateChangeSet, State};
use moveos_types::state_resolver::MoveOSResolverProxy;
use rooch_rpc_api::jsonrpc_types::{AnnotatedMoveStructView, AnnotatedMoveValueView};
use rooch_types::indexer::event_filter::IndexerEvent;
use rooch_types::indexer::state::{IndexerGlobalState, IndexerTableChangeSet, IndexerTableState};
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

    pub fn resolve_raw_object_value_to_json(&self, raw_object: &RawObject) -> Result<String> {
        let obj_value = MoveValueAnnotator::new(&self.moveos_store)
            .view_resource(&raw_object.value.struct_tag, &raw_object.value.value)?;
        let obj_value_view = AnnotatedMoveStructView::from(obj_value);
        // let raw_data_view = RawDataView {
        //     struct_tag: raw_object.value.struct_tag.clone().into(),
        //     value: obj_value_view,
        // };
        // let raw_object_view = RawObjectView::new(
        //     raw_object.id,
        //     raw_object.owner,
        //     raw_object.flag,
        //     raw_data_view,
        // );
        let raw_object_value_json = serde_json::to_string(&obj_value_view)?;
        Ok(raw_object_value_json)
    }

    pub fn resolve_state_to_json(&self, state: &State) -> Result<String> {
        let annotator_state = MoveValueAnnotator::new(&self.moveos_store)
            .view_value(&state.value_type, &state.value)?;
        let annotator_state_view = AnnotatedMoveValueView::from(annotator_state);
        let annotator_state_json = serde_json::to_string(&annotator_state_view)?;
        Ok(annotator_state_json)
    }
}

impl Actor for IndexerActor {}

#[async_trait]
impl Handler<IndexerStatesMessage> for IndexerActor {
    async fn handle(&mut self, msg: IndexerStatesMessage, _ctx: &mut ActorContext) -> Result<()> {
        let IndexerStatesMessage {
            tx_order,
            state_change_set,
        } = msg;

        // indexer state index generator
        let mut state_index_generator = 0u64;
        let mut new_global_states = vec![];
        let mut update_global_states = vec![];
        let mut remove_global_states = vec![];

        let mut new_table_states = vec![];
        let mut update_table_states = vec![];
        let mut remove_table_states = vec![];

        // When remove table handle, first delete table handle from global states,
        // then delete all states which belongs to the table_handle from table states
        let mut remove_table_states_by_table_handle = vec![];

        for (table_handle, table_change) in state_change_set.changes.clone() {
            // handle global object
            if table_handle == context::GLOBAL_OBJECT_STORAGE_HANDLE {
                for (key, op) in table_change.entries.into_iter() {
                    match op {
                        Op::Modify(value) => {
                            // table object
                            if value.match_struct_type(&ObjectEntity::get_table_object_struct_tag())
                            {
                                let table_object = value.cast::<ObjectEntity<TableInfo>>()?;
                                // let obj_value_json = serde_json::to_string(&table_object.value)?;

                                let raw_object = value.as_raw_object()?;
                                let obj_value_json =
                                    self.resolve_raw_object_value_to_json(&raw_object)?;
                                let value_type =
                                    raw_object.value.struct_tag.clone().to_canonical_string();

                                let state = IndexedGlobalState::new_from_table_object_update(
                                    table_object,
                                    obj_value_json,
                                    value_type,
                                    tx_order,
                                    state_index_generator,
                                );
                                update_global_states.push(state);
                                // struct object
                            } else if value.is_object() {
                                let raw_object = value.as_raw_object()?;
                                let obj_value_json =
                                    self.resolve_raw_object_value_to_json(&raw_object)?;
                                let value_type =
                                    raw_object.value.struct_tag.clone().to_canonical_string();

                                let state = IndexedGlobalState::new_from_raw_object(
                                    raw_object,
                                    obj_value_json,
                                    value_type,
                                    tx_order,
                                    state_index_generator,
                                );

                                update_global_states.push(state);
                            } else {
                                log::warn!(
                                    "Unexpected state type, table handle {:?}, value {:?}",
                                    table_handle,
                                    value
                                );
                            }
                        }
                        Op::Delete => {
                            let table_handle = ObjectID::from_bytes(key.as_slice())?;
                            remove_global_states.push(table_handle.to_string());
                        }
                        Op::New(value) => {
                            // table object
                            if value.match_struct_type(&ObjectEntity::get_table_object_struct_tag())
                            {
                                let object = value.cast::<ObjectEntity<TableInfo>>()?;
                                // let obj_value_json = serde_json::to_string(&object.value)?;
                                let raw_object = value.as_raw_object()?;
                                let obj_value_json =
                                    self.resolve_raw_object_value_to_json(&raw_object)?;
                                let value_type =
                                    raw_object.value.struct_tag.clone().to_canonical_string();

                                let table_handle = ObjectID::from_bytes(key.as_slice())?;
                                // TODO handle if key type is None
                                let key_type = state_change_set
                                    .new_tables
                                    .get(&table_handle)
                                    .map_or("".to_string(), |t| t.key_type.to_canonical_string());

                                let state = IndexedGlobalState::new_from_table_object(
                                    object,
                                    obj_value_json,
                                    value_type,
                                    key_type,
                                    tx_order,
                                    state_index_generator,
                                );
                                new_global_states.push(state);
                            } else if value.is_object() {
                                let raw_object = value.as_raw_object()?;
                                let obj_value_json =
                                    self.resolve_raw_object_value_to_json(&raw_object)?;
                                let value_type =
                                    raw_object.value.struct_tag.clone().to_canonical_string();

                                let state = IndexedGlobalState::new_from_raw_object(
                                    raw_object,
                                    obj_value_json,
                                    value_type,
                                    tx_order,
                                    state_index_generator,
                                );
                                new_global_states.push(state);
                            }
                        }
                    }
                    state_index_generator = state_index_generator + 1;
                }
            } else {
                // TODO update table size if ObjectID is table hanlde
                // let object = self.moveos_store.0.get_state_store().get_as_object::<TableInfo>(table_handle)?;

                // TODO Since table creation may be lazy, create global table if the table does not exist
                // let (mut object, table) = self.get_as_table_or_create(table_handle)?;

                for (key, op) in table_change.entries.into_iter() {
                    match op {
                        Op::Modify(value) => {
                            let key_hash = format!("0x{}", hex::encode(key.as_slice()));
                            let state_json = self.resolve_state_to_json(&value)?;
                            let state = IndexedTableState::new(
                                table_handle,
                                key_hash,
                                state_json,
                                value.value_type,
                                tx_order,
                                state_index_generator,
                            );
                            update_table_states.push(state);
                        }
                        Op::Delete => {
                            let key_hash = format!("0x{}", hex::encode(key.as_slice()));
                            remove_table_states.push((table_handle.to_string(), key_hash));
                        }
                        Op::New(value) => {
                            let key_hash = format!("0x{}", hex::encode(key.as_slice()));
                            let state_json = self.resolve_state_to_json(&value)?;
                            let state = IndexedTableState::new(
                                table_handle,
                                key_hash,
                                state_json,
                                value.value_type,
                                tx_order,
                                state_index_generator,
                            );
                            new_table_states.push(state);
                        }
                    }
                    state_index_generator = state_index_generator + 1;
                }
            }
        }

        for table_handle in state_change_set.removed_tables.clone() {
            remove_global_states.push(table_handle.to_string());
            remove_table_states_by_table_handle.push(table_handle.to_string());
            state_index_generator = state_index_generator + 1;
        }

        //Merge new global states and update global states
        new_global_states.append(&mut update_global_states);
        self.indexer_store
            .persist_or_update_global_states(new_global_states)?;
        self.indexer_store
            .delete_global_states(remove_global_states)?;

        //Merge new table states and update table states
        new_table_states.append(&mut update_table_states);
        self.indexer_store
            .persist_or_update_table_states(new_table_states)?;
        self.indexer_store
            .delete_table_states(remove_table_states)?;
        self.indexer_store
            .delete_table_states_by_table_handle(remove_table_states_by_table_handle)?;

        // Store table change set for state sync
        let mut split_state_change_set = SplitStateChangeSet::default();
        for (table_handle, table_info) in state_change_set.new_tables {
            split_state_change_set.add_new_table(table_handle, table_info);
        }
        for (table_handle, table_change) in state_change_set.changes.clone() {
            split_state_change_set.add_table_change(table_handle, table_change);
        }
        for table_handle in state_change_set.removed_tables {
            split_state_change_set.add_remove_table(table_handle);
        }

        let mut indexed_table_change_sets = vec![];
        for (index, item) in split_state_change_set
            .table_change_sets
            .into_iter()
            .enumerate()
        {
            let table_change_set =
                IndexedTableChangeSet::new(tx_order, index as u64, item.0, item.1)?;
            indexed_table_change_sets.push(table_change_set);
        }
        self.indexer_store
            .persist_table_change_sets(indexed_table_change_sets)?;
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

#[async_trait]
impl Handler<QueryIndexerGlobalStatesMessage> for IndexerActor {
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
impl Handler<QueryIndexerTableStatesMessage> for IndexerActor {
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
impl Handler<SyncIndexerStatesMessage> for IndexerActor {
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
