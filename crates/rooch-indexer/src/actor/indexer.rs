// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::actor::messages::{
    IndexerEventsMessage, IndexerStatesMessage, IndexerTransactionMessage,
};
use crate::store::traits::IndexerStoreTrait;
use crate::types::{
    IndexedEvent, IndexedGlobalState, IndexedTableChangeSet, IndexedTableState, IndexedTransaction,
};
use crate::utils::format_struct_tag;
use crate::IndexerStore;
use anyhow::Result;
use async_trait::async_trait;
use coerce::actor::{context::ActorContext, message::Handler, Actor};
use move_core_types::effects::Op;
use move_core_types::language_storage::TypeTag;
use move_resource_viewer::MoveValueAnnotator;
use moveos_store::MoveOSStore;
use moveos_types::moveos_std::context;
use moveos_types::moveos_std::object::RawObject;
use moveos_types::moveos_std::object_id::ObjectID;
use moveos_types::state::{KeyState, MoveStructType, SplitStateChangeSet, State};
use moveos_types::state_resolver::{MoveOSResolverProxy, StateResolver};
use rooch_rpc_api::jsonrpc_types::{AnnotatedMoveStructView, AnnotatedMoveValueView};
use rooch_types::bitcoin::utxo::UTXO;

pub struct IndexerActor {
    indexer_store: IndexerStore,
    moveos_store: MoveOSResolverProxy<MoveOSStore>,
    data_verify_mode: bool,
}

impl IndexerActor {
    pub fn new(
        indexer_store: IndexerStore,
        moveos_store: MoveOSStore,
        data_verify_mode: bool,
    ) -> Result<Self> {
        Ok(Self {
            indexer_store,
            moveos_store: MoveOSResolverProxy(moveos_store),
            data_verify_mode,
        })
    }

    pub fn resolve_raw_object_value_to_json(&self, raw_object: &RawObject) -> Result<String> {
        let obj_value = MoveValueAnnotator::new(&self.moveos_store)
            .view_resource(&raw_object.value.struct_tag, &raw_object.value.value)?;
        let obj_value_view = AnnotatedMoveStructView::from(obj_value);
        let raw_object_value_json = serde_json::to_string(&obj_value_view)?;
        Ok(raw_object_value_json)
    }

    pub fn resolve_state_to_json(&self, ty_tag: &TypeTag, value: &[u8]) -> Result<String> {
        let annotator_state =
            MoveValueAnnotator::new(&self.moveos_store).view_value(ty_tag, value)?;
        let annotator_state_view = AnnotatedMoveValueView::from(annotator_state);
        let annotator_state_json = serde_json::to_string(&annotator_state_view)?;
        Ok(annotator_state_json)
    }

    pub fn resolve_object_state(&self, table_handle: &ObjectID) -> Result<Option<State>> {
        self.moveos_store.resolve_object_state(table_handle)
    }

    pub fn is_utxo_object(&self, state_opt: Option<State>) -> bool {
        match state_opt {
            Some(state) => state.match_struct_type(&UTXO::struct_tag()),
            None => false,
        }
    }

    pub fn new_global_state_from_raw_object(
        &self,
        value: State,
        tx_order: u64,
        state_index: u64,
    ) -> Result<IndexedGlobalState> {
        let raw_object = value.as_raw_object()?;
        let obj_value_json = self.resolve_raw_object_value_to_json(&raw_object)?;
        let object_type = format_struct_tag(raw_object.value.struct_tag.clone());

        let state = IndexedGlobalState::new_from_raw_object(
            raw_object,
            obj_value_json,
            object_type,
            tx_order,
            state_index,
        );
        Ok(state)
    }

    pub fn new_table_state(
        &self,
        key: KeyState,
        value: State,
        table_handle: ObjectID,
        tx_order: u64,
        state_index: u64,
    ) -> Result<IndexedTableState> {
        let key_hex = key.to_string();
        let key_state_json = self.resolve_state_to_json(&key.key_type, key.key.as_slice())?;
        let state_json = self.resolve_state_to_json(&value.value_type, value.value.as_slice())?;
        let state = IndexedTableState::new(
            table_handle,
            key_hex,
            key_state_json,
            state_json,
            key.key_type,
            value.value_type,
            tx_order,
            state_index,
        );
        Ok(state)
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
                            if value.is_object() {
                                let state = self.new_global_state_from_raw_object(
                                    value,
                                    tx_order,
                                    state_index_generator,
                                )?;
                                //just for data verify mode, don't write utxo object indexer
                                if self.data_verify_mode {
                                    if state.is_utxo_object_state() {
                                        continue;
                                    }
                                }
                                update_global_states.push(state);
                            } else {
                                log::warn!(
                                    "Unexpected state type for op modify, table handle {:?}, value {:?}",
                                    table_handle,
                                    value
                                );
                            }
                        }
                        Op::Delete => {
                            let table_handle = ObjectID::from_bytes(key.key.as_slice())?;
                            remove_global_states.push(table_handle.to_string());
                        }
                        Op::New(value) => {
                            if value.is_object() {
                                let state = self.new_global_state_from_raw_object(
                                    value,
                                    tx_order,
                                    state_index_generator,
                                )?;
                                //just for data verify mode, don't write utxo object indexer
                                if self.data_verify_mode {
                                    if state.is_utxo_object_state() {
                                        continue;
                                    }
                                }
                                new_global_states.push(state);
                            } else {
                                log::warn!(
                                    "Unexpected state type for op new, table handle {:?}, value {:?}",
                                    table_handle,
                                    value
                                );
                            }
                        }
                    }
                    state_index_generator += 1;
                }
            } else {
                // TODO update table size if ObjectID is table hanlde
                for (key, op) in table_change.entries.into_iter() {
                    match op {
                        Op::Modify(value) => {
                            let state = self.new_table_state(
                                key,
                                value,
                                table_handle,
                                tx_order,
                                state_index_generator,
                            )?;

                            //just for data verify mode, don't write utxo object indexer
                            if self.data_verify_mode {
                                let object_state = self.resolve_object_state(&table_handle)?;
                                if self.is_utxo_object(object_state) {
                                    continue;
                                };
                            };

                            update_table_states.push(state);
                        }
                        Op::Delete => {
                            remove_table_states.push((table_handle.to_string(), key.to_string()));
                        }
                        Op::New(value) => {
                            let state = self.new_table_state(
                                key,
                                value,
                                table_handle,
                                tx_order,
                                state_index_generator,
                            )?;

                            //just for data verify mode, don't write utxo object indexer
                            if self.data_verify_mode {
                                let object_state = self.resolve_object_state(&table_handle)?;
                                if self.is_utxo_object(object_state) {
                                    continue;
                                };
                            };

                            new_table_states.push(state);
                        }
                    }
                    state_index_generator += 1;
                }
            }
        }

        for table_handle in state_change_set.removed_tables.clone() {
            remove_global_states.push(table_handle.to_string());
            remove_table_states_by_table_handle.push(table_handle.to_string());
            state_index_generator += 1;
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
        for table_handle in state_change_set.new_tables {
            split_state_change_set.add_new_table(table_handle);
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
