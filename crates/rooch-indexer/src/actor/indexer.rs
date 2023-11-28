// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::actor::messages::{
    IndexerEventsMessage, IndexerStatesMessage, IndexerTransactionMessage,
    QueryIndexerEventsMessage, QueryIndexerTransactionsMessage,
};
use crate::indexer_reader::IndexerReader;
use crate::store::traits::IndexerStoreTrait;
use crate::types::{IndexedEvent, IndexedGlobalState, IndexedLeafState, IndexedTransaction};
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
use moveos_types::state::State;
use moveos_types::state_resolver::MoveOSResolverProxy;
use rooch_rpc_api::jsonrpc_types::{AnnotatedMoveStructView, AnnotatedMoveValueView};
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

    pub fn resolve_raw_object_value_to_json(&self, raw_object: &RawObject) -> Result<String> {
        let obj_value = MoveValueAnnotator::new(&self.moveos_store)
            .view_resource(&raw_object.value.struct_tag, &raw_object.value.value)?;
        let obj_value_view = AnnotatedMoveStructView::from(obj_value);
        let obj_value_json = serde_json::to_string(&obj_value_view)?;
        Ok(obj_value_json)
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
        let IndexerStatesMessage { state_change_set } = msg;

        // pub struct StateChangeSet {
        //     pub new_tables: BTreeMap<ObjectID, TableTypeInfo>,
        //     pub removed_tables: BTreeSet<ObjectID>,
        //     pub changes: BTreeMap<ObjectID, TableChange>,
        // }

        let mut new_global_states = vec![];
        let mut update_global_states = vec![];
        let mut remove_global_states = vec![];

        let mut new_leaf_states = vec![];
        let mut update_leaf_states = vec![];
        let mut remove_leaf_states = vec![];

        for (table_handle, table_change) in state_change_set.changes {
            // handle global object
            if table_handle == context::GLOBAL_OBJECT_STORAGE_HANDLE {
                for (key, op) in table_change.entries.into_iter() {
                    match op {
                        Op::Modify(value) => {
                            // table object
                            if value.match_struct_type(&ObjectEntity::get_table_object_struct_tag())
                            {
                                let object = value.cast::<ObjectEntity<TableInfo>>()?;
                                let obj_value_json = serde_json::to_string(&object.value)?;

                                //TODO don't need set key_type when update
                                let state = IndexedGlobalState::new_from_table_object(
                                    object,
                                    obj_value_json,
                                    "".to_string(),
                                );
                                update_global_states.push(state);
                            } else if value.is_object() {
                                let raw_object = value.as_raw_object()?;
                                let obj_value_json =
                                    self.resolve_raw_object_value_to_json(&raw_object)?;

                                let state = IndexedGlobalState::new_from_raw_object(
                                    raw_object,
                                    obj_value_json,
                                );
                                update_global_states.push(state);
                            }
                        }
                        Op::Delete => {
                            let table_handle = ObjectID::from_bytes(key.as_slice())?;
                            remove_global_states.push(table_handle);
                        }
                        Op::New(value) => {
                            // table object
                            if value.match_struct_type(&ObjectEntity::get_table_object_struct_tag())
                            {
                                let object = value.cast::<ObjectEntity<TableInfo>>()?;
                                let obj_value_json = serde_json::to_string(&object.value)?;

                                let table_handle = ObjectID::from_bytes(key.as_slice())?;
                                let key_type = state_change_set
                                    .new_tables
                                    .get(&table_handle)
                                    .map_or("".to_string(), |t| t.key_type.to_canonical_string());

                                let state = IndexedGlobalState::new_from_table_object(
                                    object,
                                    obj_value_json,
                                    key_type,
                                );
                                new_global_states.push(state);
                            } else if value.is_object() {
                                let raw_object = value.as_raw_object()?;
                                let obj_value_json =
                                    self.resolve_raw_object_value_to_json(&raw_object)?;

                                let state = IndexedGlobalState::new_from_raw_object(
                                    raw_object,
                                    obj_value_json,
                                );
                                new_global_states.push(state);
                            }
                        }
                    }
                }
            } else {
                // let object = self.moveos_store.0.get_state_store().get_as_object::<TableInfo>(table_handle)?;
                // let new_state_root = table.put_changes(table_change.entries.into_iter())?;
                // object.value.state_root = AccountAddress::new(new_state_root.into());
                // let curr_table_size: i64 = object.value.size as i64;
                // let updated_table_size = curr_table_size + table_change.size_increment;
                // debug_assert!(updated_table_size >= 0);
                // object.value.size = updated_table_size as u64;
                // changed_objects.put(table_handle.to_bytes(), object.into());

                for (key, op) in table_change.entries.into_iter() {
                    match op {
                        Op::Modify(value) => {
                            let key_hash = format!("0x{}", hex::encode(key.as_slice()));
                            let state_json = self.resolve_state_to_json(&value)?;
                            let state = IndexedLeafState::new(
                                table_handle,
                                key_hash,
                                state_json,
                                value.value_type,
                            );
                            update_leaf_states.push(state);
                        }
                        Op::Delete => {
                            let key_hash = format!("0x{}", hex::encode(key.as_slice()));
                            remove_leaf_states.push((table_handle, key_hash));
                        }
                        Op::New(value) => {
                            let key_hash = format!("0x{}", hex::encode(key.as_slice()));
                            let state_json = self.resolve_state_to_json(&value)?;
                            let state = IndexedLeafState::new(
                                table_handle,
                                key_hash,
                                state_json,
                                value.value_type,
                            );
                            new_leaf_states.push(state);
                        }
                    }
                }
            }
        }

        // for table_handle in state_change_set.removed_tables {
        //     changed_objects.remove(table_handle.to_bytes());
        // }
        //

        self.indexer_store
            .persist_global_states(new_global_states)?;
        self.indexer_store
            .update_global_states(update_global_states)?;
        // self.indexer_store.delete_global_states(remove_global_states)?;

        self.indexer_store.persist_leaf_states(new_leaf_states)?;
        self.indexer_store.update_leaf_states(update_leaf_states)?;
        // self.indexer_store.delete_leaf_states(remove_leaf_states)?;
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
