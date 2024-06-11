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
use moveos_types::state_resolver::RootObjectResolver;
use moveos_types::transaction::MoveAction;
use rooch_types::indexer::event::IndexerEvent;
use rooch_types::indexer::state::{handle_object_change, IndexerObjectStateChanges};
use rooch_types::indexer::transaction::IndexerTransaction;

pub struct IndexerActor {
    root: RootObjectEntity,
    indexer_store: IndexerStore,
    moveos_store: MoveOSStore,
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
            moveos_store,
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
        let resolver = RootObjectResolver::new(self.root.clone(), &self.moveos_store);

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

        for (object_id, object_change) in state_change_set.changes {
            state_index_generator = handle_object_change(
                state_index_generator,
                tx_order,
                &mut indexer_object_state_changes,
                object_id,
                object_change,
                &resolver,
            )?;
        }
        self.indexer_store
            .update_object_states(indexer_object_state_changes)?;

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
        let mut state_index_generator = 0u64;
        let mut indexer_object_state_changes = IndexerObjectStateChanges::default();

        let resolver = RootObjectResolver::new(self.root.clone(), &self.moveos_store);
        for (object_id, object_change) in state_change_set.changes {
            state_index_generator = handle_object_change(
                state_index_generator,
                tx_order,
                &mut indexer_object_state_changes,
                object_id,
                object_change,
                &resolver,
            )?;
        }

        self.indexer_store
            .update_object_states(indexer_object_state_changes)?;

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
