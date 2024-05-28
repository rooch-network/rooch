// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::actor::messages::{
    IndexerEventsMessage, IndexerStatesMessage, IndexerTransactionMessage, UpdateIndexerMessage,
};
use crate::store::traits::IndexerStoreTrait;
use crate::IndexerStore;
use anyhow::Result;
use async_trait::async_trait;
use coerce::actor::{context::ActorContext, message::Handler, Actor};
use moveos_store::MoveOSStore;
use moveos_types::moveos_std::object::RootObjectEntity;
use moveos_types::transaction::MoveAction;
use rooch_types::indexer::event::IndexerEvent;
use rooch_types::indexer::state::{
    handle_object_change, IndexerFieldStateChanges, IndexerObjectStateChanges,
};
use rooch_types::indexer::transaction::IndexerTransaction;

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
}

impl Actor for IndexerActor {}

#[async_trait]
impl Handler<UpdateIndexerMessage> for IndexerActor {
    async fn handle(&mut self, msg: UpdateIndexerMessage, _ctx: &mut ActorContext) -> Result<()> {
        let UpdateIndexerMessage {
            root,
            ledger_transaction,
            execution_info,
            moveos_tx,
            events,
            state_change_set,
        } = msg;

        self.root = root;
        let tx_order = ledger_transaction.sequence_info.tx_order;

        // 1. update indexer transaction
        let move_action = MoveAction::from(moveos_tx.action);
        let indexer_transaction = IndexerTransaction::new(
            ledger_transaction.clone(),
            execution_info.clone(),
            move_action,
            moveos_tx.ctx.clone(),
        )?;
        let transactions = vec![indexer_transaction];
        self.indexer_store.persist_transactions(transactions)?;

        // 2. update indexer state
        let events: Vec<_> = events
            .into_iter()
            .map(|event| {
                IndexerEvent::new(
                    event.clone(),
                    ledger_transaction.clone(),
                    moveos_tx.ctx.clone(),
                )
            })
            .collect();
        self.indexer_store.persist_events(events)?;

        // 3. update indexer state
        // indexer state index generator
        let mut state_index_generator = 0u64;
        let mut indexer_object_state_changes = IndexerObjectStateChanges::default();
        let mut indexer_field_state_changes = IndexerFieldStateChanges::default();

        for (object_id, object_change) in state_change_set.changes {
            state_index_generator = handle_object_change(
                state_index_generator,
                tx_order,
                &mut indexer_object_state_changes,
                &mut indexer_field_state_changes,
                object_id,
                object_change,
            )?;
        }
        self.indexer_store
            .update_object_states(indexer_object_state_changes)?;
        self.indexer_store
            .update_field_states(indexer_field_state_changes)?;

        Ok(())
    }
}

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
        let mut indexer_object_state_changes = IndexerObjectStateChanges::default();
        let mut indexer_field_state_changes = IndexerFieldStateChanges::default();

        for (object_id, object_change) in state_change_set.changes {
            state_index_generator = handle_object_change(
                state_index_generator,
                tx_order,
                &mut indexer_object_state_changes,
                &mut indexer_field_state_changes,
                object_id,
                object_change,
            )?;
        }

        //Merge new object states and update object states
        let mut object_states_new_and_update = indexer_object_state_changes.new_object_states;
        object_states_new_and_update.append(&mut indexer_object_state_changes.update_object_states);
        self.indexer_store
            .persist_or_update_object_states(object_states_new_and_update)?;
        self.indexer_store
            .delete_object_states(indexer_object_state_changes.remove_object_states)?;

        //Merge new field states and update field states
        let mut fiels_states_new_and_update = indexer_field_state_changes.new_field_states;
        fiels_states_new_and_update.append(&mut indexer_field_state_changes.update_field_states);
        self.indexer_store
            .persist_or_update_field_states(fiels_states_new_and_update)?;
        self.indexer_store
            .delete_field_states(indexer_field_state_changes.remove_field_states)?;
        self.indexer_store.delete_field_states_by_object_id(
            indexer_field_state_changes.remove_field_states_by_object_id,
        )?;

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
            ledger_transaction,
            execution_info,
            move_action,
            tx_context,
        } = msg;

        let indexer_transaction =
            IndexerTransaction::new(ledger_transaction, execution_info, move_action, tx_context)?;
        let transactions = vec![indexer_transaction];

        self.indexer_store.persist_transactions(transactions)?;
        Ok(())
    }
}

#[async_trait]
impl Handler<IndexerEventsMessage> for IndexerActor {
    async fn handle(&mut self, msg: IndexerEventsMessage, _ctx: &mut ActorContext) -> Result<()> {
        let IndexerEventsMessage {
            events,
            ledger_transaction,
            tx_context,
        } = msg;

        let events: Vec<_> = events
            .into_iter()
            .map(|event| IndexerEvent::new(event, ledger_transaction.clone(), tx_context.clone()))
            .collect();
        self.indexer_store.persist_events(events)?;
        Ok(())
    }
}
