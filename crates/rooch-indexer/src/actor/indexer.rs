// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::actor::messages::{
    IndexerEventsMessage, IndexerStatesMessage, IndexerTransactionMessage,
};
use crate::actor::{new_field_state, new_object_state_from_raw_object};
use crate::store::traits::IndexerStoreTrait;
use crate::types::{IndexedEvent, IndexedFieldState, IndexedObjectState, IndexedTransaction};
use crate::IndexerStore;
use anyhow::Result;
use async_trait::async_trait;
use coerce::actor::{context::ActorContext, message::Handler, Actor};
use move_core_types::effects::Op;
use moveos_store::MoveOSStore;
use moveos_types::moveos_std::object::{ObjectID, RootObjectEntity};
use moveos_types::state::{FieldChange, ObjectChange};

pub struct IndexerActor {
    root: RootObjectEntity,
    indexer_store: IndexerStore,
    _moveos_store: MoveOSStore,
}

impl IndexerActor {
    pub fn new(
        root: RootObjectEntity,
        indexer_store: IndexerStore,
        moveos_store: MoveOSStore,
    ) -> Result<Self> {
        Ok(Self {
            root,
            indexer_store,
            _moveos_store: moveos_store,
        })
    }

    //TODO wrap the arguments to a struct
    #[allow(clippy::too_many_arguments)]
    fn handle_object_change(
        mut state_index_generator: u64,
        tx_order: u64,
        new_object_states: &mut Vec<IndexedObjectState>,
        update_object_states: &mut Vec<IndexedObjectState>,
        remove_object_states: &mut Vec<String>,
        remove_field_states_by_object_id: &mut Vec<String>,
        new_field_states: &mut Vec<IndexedFieldState>,
        update_field_states: &mut Vec<IndexedFieldState>,
        remove_field_states: &mut Vec<(String, String)>,
        object_id: ObjectID,
        object_change: ObjectChange,
    ) -> Result<u64> {
        let ObjectChange { op, fields } = object_change;

        if let Some(op) = op {
            match op {
                Op::Modify(value) => {
                    debug_assert!(value.is_object());
                    let state =
                        new_object_state_from_raw_object(value, tx_order, state_index_generator)?;
                    update_object_states.push(state);
                }
                Op::Delete => {
                    remove_object_states.push(object_id.to_string());
                    remove_field_states_by_object_id.push(object_id.to_string());
                }
                Op::New(value) => {
                    debug_assert!(value.is_object());
                    let state =
                        new_object_state_from_raw_object(value, tx_order, state_index_generator)?;
                    new_object_states.push(state);
                }
            }
        }

        state_index_generator += 1;
        for (key, change) in fields {
            match change {
                FieldChange::Normal(normal_change) => {
                    match normal_change.op {
                        Op::Modify(value) => {
                            let state = new_field_state(
                                key,
                                value,
                                object_id.clone(),
                                tx_order,
                                state_index_generator,
                            )?;
                            update_field_states.push(state);
                        }
                        Op::Delete => {
                            remove_field_states.push((object_id.to_string(), key.to_string()));
                        }
                        Op::New(value) => {
                            let state = new_field_state(
                                key,
                                value,
                                object_id.clone(),
                                tx_order,
                                state_index_generator,
                            )?;
                            new_field_states.push(state);
                        }
                    }
                    state_index_generator += 1;
                }
                FieldChange::Object(object_change) => {
                    state_index_generator = Self::handle_object_change(
                        state_index_generator,
                        tx_order,
                        new_object_states,
                        update_object_states,
                        remove_object_states,
                        remove_field_states_by_object_id,
                        new_field_states,
                        update_field_states,
                        remove_field_states,
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
            root,
            tx_order,
            state_change_set,
        } = msg;

        self.root = root;

        // indexer state index generator
        let mut state_index_generator = 0u64;
        let mut new_object_states = vec![];
        let mut update_object_states = vec![];
        let mut remove_object_states = vec![];

        let mut new_field_states = vec![];
        let mut update_field_states = vec![];
        let mut remove_field_states = vec![];

        // When remove table handle, first delete table handle from global states,
        // then delete all states which belongs to the object_id from table states
        let mut remove_field_states_by_object_id = vec![];

        for (object_id, object_change) in state_change_set.changes {
            state_index_generator = Self::handle_object_change(
                state_index_generator,
                tx_order,
                &mut new_object_states,
                &mut update_object_states,
                &mut remove_object_states,
                &mut remove_field_states_by_object_id,
                &mut new_field_states,
                &mut update_field_states,
                &mut remove_field_states,
                object_id,
                object_change,
            )?;
        }

        //Merge new object states and update object states
        new_object_states.append(&mut update_object_states);
        self.indexer_store
            .persist_or_update_object_states(new_object_states)?;
        self.indexer_store
            .delete_object_states(remove_object_states)?;

        //Merge new field states and update field states
        new_field_states.append(&mut update_field_states);
        self.indexer_store
            .persist_or_update_field_states(new_field_states)?;
        self.indexer_store
            .delete_field_states(remove_field_states)?;
        self.indexer_store
            .delete_field_states_by_object_id(remove_field_states_by_object_id)?;

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
