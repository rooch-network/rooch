// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::actor::messages::{
    IndexerApplyObjectStatesMessage, IndexerDeleteAnyObjectStatesMessage, IndexerEventsMessage,
    IndexerPersistOrUpdateAnyObjectStatesMessage, IndexerRevertStatesMessage, IndexerStatesMessage,
    IndexerTransactionMessage, UpdateIndexerMessage,
};
use crate::store::traits::IndexerStoreTrait;
use crate::IndexerStore;
use anyhow::Result;
use async_trait::async_trait;
use coerce::actor::{context::ActorContext, message::Handler, Actor};
use moveos_types::moveos_std::object::ObjectMeta;
use moveos_types::transaction::MoveAction;
use rooch_types::indexer::event::IndexerEvent;
use rooch_types::indexer::state::{handle_object_change, handle_revert_object_change, IndexerObjectStateChangeSet, IndexerObjectStatesIndexGenerator, ObjectStateType};
use rooch_types::indexer::transaction::IndexerTransaction;

pub struct IndexerActor {
    root: ObjectMeta,
    indexer_store: IndexerStore,
}

impl IndexerActor {
    pub fn new(root: ObjectMeta, indexer_store: IndexerStore) -> Result<Self> {
        Ok(Self {
            root,
            indexer_store,
        })
    }
}

impl Actor for IndexerActor {}

#[async_trait]
impl Handler<UpdateIndexerMessage> for IndexerActor {
    async fn handle(&mut self, msg: UpdateIndexerMessage, _ctx: &mut ActorContext) -> Result<()> {
        let UpdateIndexerMessage {
            ledger_transaction,
            execution_info,
            moveos_tx,
            events,
            state_change_set,
        } = msg;

        self.root = state_change_set.root_metadata();
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

        // 2. update indexer event
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

        // 3. update indexer full object state, including object_states, utxos and inscriptions
        // indexer object state index generator
        let mut state_index_generator = IndexerObjectStatesIndexGenerator::default();
        let mut indexer_object_state_change_set = IndexerObjectStateChangeSet::default();

        for (_feild_key, object_change) in state_change_set.changes {
            let _ = handle_object_change(
                &mut state_index_generator,
                tx_order,
                &mut indexer_object_state_change_set,
                object_change,
            )?;
        }
        self.indexer_store
            .apply_object_states(indexer_object_state_change_set)?;

        Ok(())
    }
}

#[async_trait]
impl Handler<IndexerStatesMessage> for IndexerActor {
    async fn handle(&mut self, msg: IndexerStatesMessage, _ctx: &mut ActorContext) -> Result<()> {
        let IndexerStatesMessage {
            root,
            tx_order,
            tx_timestamp: _,
            state_change_set,
        } = msg;

        self.root = root;

        // indexer state index generator
        let mut state_index_generator = IndexerObjectStatesIndexGenerator::default();
        let mut indexer_object_state_change_set = IndexerObjectStateChangeSet::default();

        for (_field_key, object_change) in state_change_set.changes {
            handle_object_change(
                &mut state_index_generator,
                tx_order,
                &mut indexer_object_state_change_set,
                object_change,
            )?;
        }

        self.indexer_store
            .apply_object_states(indexer_object_state_change_set)?;

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

#[async_trait]
impl Handler<IndexerPersistOrUpdateAnyObjectStatesMessage> for IndexerActor {
    async fn handle(
        &mut self,
        msg: IndexerPersistOrUpdateAnyObjectStatesMessage,
        _ctx: &mut ActorContext,
    ) -> Result<()> {
        let IndexerPersistOrUpdateAnyObjectStatesMessage { states, state_type } = msg;

        match state_type {
            ObjectStateType::ObjectState => {
                self.indexer_store.persist_or_update_object_states(states)?
            }
            ObjectStateType::UTXO => self
                .indexer_store
                .persist_or_update_object_state_utxos(states)?,
            ObjectStateType::Inscription => self
                .indexer_store
                .persist_or_update_object_state_inscriptions(states)?,
        }
        Ok(())
    }
}

#[async_trait]
impl Handler<IndexerDeleteAnyObjectStatesMessage> for IndexerActor {
    async fn handle(
        &mut self,
        msg: IndexerDeleteAnyObjectStatesMessage,
        _ctx: &mut ActorContext,
    ) -> Result<()> {
        let IndexerDeleteAnyObjectStatesMessage {
            object_ids,
            state_type,
        } = msg;

        let state_pks = object_ids.into_iter().map(|v| v.to_string()).collect();
        match state_type {
            ObjectStateType::ObjectState => self.indexer_store.delete_object_states(state_pks)?,
            ObjectStateType::UTXO => self.indexer_store.delete_object_state_utxos(state_pks)?,
            ObjectStateType::Inscription => self
                .indexer_store
                .delete_object_state_inscriptions(state_pks)?,
        }
        Ok(())
    }
}

#[async_trait]
impl Handler<IndexerApplyObjectStatesMessage> for IndexerActor {
    async fn handle(
        &mut self,
        msg: IndexerApplyObjectStatesMessage,
        _ctx: &mut ActorContext,
    ) -> Result<()> {
        let IndexerApplyObjectStatesMessage {
            object_state_change_set,
        } = msg;

        self.indexer_store
            .apply_object_states(object_state_change_set)?;
        Ok(())
    }
}

#[async_trait]
impl Handler<IndexerRevertStatesMessage> for IndexerActor {
    async fn handle(
        &mut self,
        msg: IndexerApplyObjectStatesMessage,
        _ctx: &mut ActorContext,
    ) -> Result<()> {
        let IndexerRevertStatesMessage {
            revert_tx_order,
            revert_ledger_tx,
            revert_execution_info,
            revert_state_change_set,
            root,
        } = msg;

        self.root = root;
        let tx_order = ledger_transaction.sequence_info.tx_order;

        // 1. revert indexer transaction
        self.indexer_store
            .delete_transactions(vec![revert_tx_order])?;

        // 2. revert indexer event
        self.indexer_store.delete_events(vec![revert_tx_order])?;

        // 3. revert indexer full object state, including object_states, utxos and inscriptions
        // indexer object state index generator
        let mut state_index_generator = IndexerObjectStatesIndexGenerator::default();
        let mut indexer_object_state_change_set = IndexerObjectStateChangeSet::default();

        // set genesis tx_order and state_index_generator for new indexer revert
        let tx_order: u64 = 0;
        let last_state_index = self
            .query_last_state_index_by_tx_order(tx_order, state_type.clone())
            .await?;
        let mut state_index_generator = last_state_index.map_or(0, |x| x + 1);

        for (_feild_key, object_change) in revert_state_change_set.state_change_set.changes {
            let _ = handle_revert_object_change(
                &mut state_index_generator,
                tx_order,
                &mut indexer_object_state_change_set,
                object_change,
            )?;
        }
        self.indexer_store
            .apply_object_states(indexer_object_state_change_set)?;

        self.indexer_store.revert_states(object_state_change_set)?;
        Ok(())
    }
}
