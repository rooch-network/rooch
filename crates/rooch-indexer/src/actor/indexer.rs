// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::actor::messages::{
    IndexerEventsMessage, IndexerStatesMessage, IndexerTransactionMessage,
};
use crate::store::traits::IndexerStoreTrait;
use crate::types::{IndexedEvent, IndexedGlobalState, IndexedTableState, IndexedTransaction};
use crate::utils::format_struct_tag;
use crate::IndexerStore;
use anyhow::Result;
use async_trait::async_trait;
use coerce::actor::{context::ActorContext, message::Handler, Actor};
use move_core_types::effects::Op;
use move_core_types::language_storage::TypeTag;
use move_resource_viewer::MoveValueAnnotator;
use moveos_store::MoveOSStore;
use moveos_types::moveos_std::object::ObjectID;
use moveos_types::moveos_std::object::RawObject;
use moveos_types::state::{FieldChange, KeyState, MoveStructType, ObjectChange, State};
use moveos_types::state_resolver::{MoveOSResolverProxy, StateResolver};
use rooch_rpc_api::jsonrpc_types::{AnnotatedMoveStructView, AnnotatedMoveValueView};
use rooch_types::bitcoin::utxo::UTXO;

pub struct IndexerActor {
    indexer_store: IndexerStore,
    moveos_store: MoveOSResolverProxy<MoveOSStore>,
}

impl IndexerActor {
    pub fn new(indexer_store: IndexerStore, moveos_store: MoveOSStore) -> Result<Self> {
        Ok(Self {
            indexer_store,
            moveos_store: MoveOSResolverProxy(moveos_store),
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

    //TODO wrap the arguments to a struct
    #[allow(clippy::too_many_arguments)]
    fn handle_object_change(
        &self,
        mut state_index_generator: u64,
        tx_order: u64,
        new_global_states: &mut Vec<IndexedGlobalState>,
        update_global_states: &mut Vec<IndexedGlobalState>,
        remove_global_states: &mut Vec<String>,
        remove_table_states_by_table_handle: &mut Vec<String>,
        new_table_states: &mut Vec<IndexedTableState>,
        update_table_states: &mut Vec<IndexedTableState>,
        remove_table_states: &mut Vec<(String, String)>,
        object_id: ObjectID,
        object_change: ObjectChange,
    ) -> Result<u64> {
        let ObjectChange { op, fields } = object_change;

        if let Some(op) = op {
            match op {
                Op::Modify(value) => {
                    debug_assert!(value.is_object());
                    let state = self.new_global_state_from_raw_object(
                        value,
                        tx_order,
                        state_index_generator,
                    )?;
                    update_global_states.push(state);
                }
                Op::Delete => {
                    remove_global_states.push(object_id.to_string());
                    remove_table_states_by_table_handle.push(object_id.to_string());
                }
                Op::New(value) => {
                    debug_assert!(value.is_object());
                    let state = self.new_global_state_from_raw_object(
                        value,
                        tx_order,
                        state_index_generator,
                    )?;
                    new_global_states.push(state);
                }
            }
        }

        state_index_generator += 1;
        for (key, change) in fields {
            match change {
                FieldChange::Normal(normal_change) => {
                    match normal_change.op {
                        Op::Modify(value) => {
                            let state = self.new_table_state(
                                key,
                                value,
                                object_id.clone(),
                                tx_order,
                                state_index_generator,
                            )?;
                            update_table_states.push(state);
                        }
                        Op::Delete => {
                            remove_table_states.push((object_id.to_string(), key.to_string()));
                        }
                        Op::New(value) => {
                            let state = self.new_table_state(
                                key,
                                value,
                                object_id.clone(),
                                tx_order,
                                state_index_generator,
                            )?;
                            new_table_states.push(state);
                        }
                    }
                    state_index_generator += 1;
                }
                FieldChange::Object(object_change) => {
                    state_index_generator = self.handle_object_change(
                        state_index_generator,
                        tx_order,
                        new_global_states,
                        update_global_states,
                        remove_global_states,
                        remove_table_states_by_table_handle,
                        new_table_states,
                        update_table_states,
                        remove_table_states,
                        key.as_object_id()?,
                        object_change,
                    )?;
                }
            }
        }
        Ok(state_index_generator)
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

        for (object_id, object_change) in state_change_set.changes {
            state_index_generator = self.handle_object_change(
                state_index_generator,
                tx_order,
                &mut new_global_states,
                &mut update_global_states,
                &mut remove_global_states,
                &mut remove_table_states_by_table_handle,
                &mut new_table_states,
                &mut update_table_states,
                &mut remove_table_states,
                object_id,
                object_change,
            )?;
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

        // TODO First temporarily close StateChangeSet Indexer writing and wait for the function to be turned on.
        // Store table change set for state sync
        // let mut split_state_change_set = SplitStateChangeSet::default();

        // for (table_handle, table_change) in state_change_set.changes.clone() {
        //     split_state_change_set.add_table_change(table_handle, table_change);
        // }

        // let mut indexed_table_change_sets = vec![];
        // for (index, item) in split_state_change_set
        //     .table_change_sets
        //     .into_iter()
        //     .enumerate()
        // {
        //     let table_change_set =
        //         IndexedTableChangeSet::new(tx_order, index as u64, item.0, item.1)?;

        //     indexed_table_change_sets.push(table_change_set);
        // }

        // self.indexer_store
        //     .persist_table_change_sets(indexed_table_change_sets)?;
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
            execution_info,
            moveos_tx,
        } = msg;

        let indexed_transaction = IndexedTransaction::new(transaction, execution_info, moveos_tx)?;
        let transactions = vec![indexed_transaction];

        // just for bitcoin block data import, don't write transaction indexer
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
            moveos_tx,
        } = msg;

        let events: Vec<_> = events
            .into_iter()
            .map(|event| IndexedEvent::new(event, transaction.clone(), moveos_tx.clone()))
            .collect();
        self.indexer_store.persist_events(events)?;
        Ok(())
    }
}
