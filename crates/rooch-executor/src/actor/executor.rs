// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use super::messages::{
    AnnotatedStatesMessage, ExecuteTransactionMessage, ExecuteTransactionResult,
    ExecuteViewFunctionMessage, GetEventsByEventHandleMessage, GetEventsMessage, StatesMessage,
    ValidateTransactionMessage,
};
use anyhow::bail;
use anyhow::Result;
use async_trait::async_trait;
use coerce::actor::{context::ActorContext, message::Handler, Actor};
use move_resource_viewer::MoveValueAnnotator;
use moveos::moveos::MoveOS;
use moveos_common::accumulator::InMemoryAccumulator;
use moveos_store::MoveOSDB;
use moveos_types::event::AnnotatedMoveOSEvent;
use moveos_types::event::EventHandle;
use moveos_types::function_return_value::AnnotatedFunctionReturnValue;
use moveos_types::module_binding::MoveFunctionCaller;
use moveos_types::move_types::as_struct_tag;
use moveos_types::state::{AnnotatedState, State};
use moveos_types::state_resolver::{AnnotatedStateReader, StateReader};
use moveos_types::transaction::TransactionExecutionInfo;
use moveos_types::transaction::VerifiedMoveOSTransaction;
use moveos_types::tx_context::TxContext;
use rooch_framework::bindings::address_mapping::AddressMapping;
use rooch_framework::bindings::transaction_validator::TransactionValidator;
use rooch_genesis::RoochGenesis;
use rooch_types::transaction::AbstractTransaction;

pub struct ExecutorActor {
    moveos: MoveOS,
}

impl ExecutorActor {
    pub fn new() -> Result<Self> {
        let moveosdb = MoveOSDB::new_with_memory_store();
        let genesis: &RoochGenesis = &rooch_genesis::ROOCH_GENESIS;

        let mut moveos = MoveOS::new(moveosdb, genesis.all_natives(), genesis.config.clone())?;
        if moveos.state().is_genesis() {
            moveos.init_genesis(genesis.genesis_txs.clone())?;
        }
        Ok(Self { moveos })
    }

    pub fn validate<T: AbstractTransaction>(&self, tx: T) -> Result<VerifiedMoveOSTransaction> {
        let sender = tx.sender();

        let resolved_sender = {
            let address_mapping = self.moveos.as_module_bundle::<AddressMapping>();
            address_mapping.resolve(sender.clone())?.ok_or_else(|| {
                anyhow::anyhow!(
                    "the multiaddress sender({}) mapping record is not exists.",
                    sender
                )
            })?
        };

        let tx_context = TxContext::new(resolved_sender, tx.tx_hash());

        let authenticator = tx.authenticator_info();

        let result = {
            let tx_validator = self.moveos.as_module_bundle::<TransactionValidator>();
            tx_validator.validate(&tx_context, authenticator)
        };

        match result {
            Ok(_) => Ok(self
                .moveos
                .verify(tx.construct_moveos_transaction(resolved_sender)?)?),
            Err(e) => {
                //TODO handle the abort error code
                //let status = explain_vm_status(self.db.get_state_store(), e.into_vm_status())?;
                println!("validate failed: {:?}", e);
                // If the error code is EUnsupportedScheme, then we can try to call the sender's validate function
                // This is the Account Abstraction.
                bail!("validate failed: {:?}", e)
            }
        }
    }
}

impl Actor for ExecutorActor {}

#[async_trait]
impl<T> Handler<ValidateTransactionMessage<T>> for ExecutorActor
where
    T: 'static + AbstractTransaction + Send + Sync,
{
    async fn handle(
        &mut self,
        msg: ValidateTransactionMessage<T>,
        _ctx: &mut ActorContext,
    ) -> Result<VerifiedMoveOSTransaction> {
        self.validate(msg.tx)
    }
}

#[async_trait]
impl Handler<ExecuteTransactionMessage> for ExecutorActor {
    async fn handle(
        &mut self,
        msg: ExecuteTransactionMessage,
        _ctx: &mut ActorContext,
    ) -> Result<ExecuteTransactionResult> {
        let tx_hash = msg.tx.ctx.tx_hash();
        let (state_root, output) = self.moveos.execute(msg.tx)?;
        let event_hashes: Vec<_> = output.events.iter().map(|e| e.hash()).collect();
        let event_root = InMemoryAccumulator::from_leaves(event_hashes.as_slice()).root_hash();

        let transaction_info = TransactionExecutionInfo::new(
            tx_hash,
            state_root,
            event_root,
            0,
            output.status.clone(),
        );
        Ok(ExecuteTransactionResult {
            output,
            transaction_info,
        })
    }
}

#[async_trait]
impl Handler<ExecuteViewFunctionMessage> for ExecutorActor {
    async fn handle(
        &mut self,
        msg: ExecuteViewFunctionMessage,
        _ctx: &mut ActorContext,
    ) -> Result<Vec<AnnotatedFunctionReturnValue>, anyhow::Error> {
        let resoler = self.moveos.moveos_resolver();

        self.moveos
            .execute_view_function(msg.call)?
            .into_iter()
            .map(|v| {
                let move_value = resoler.view_value(&v.type_tag, &v.value)?;
                Ok(AnnotatedFunctionReturnValue {
                    value: v,
                    move_value,
                })
            })
            .collect::<Result<Vec<AnnotatedFunctionReturnValue>, anyhow::Error>>()
    }
}

#[async_trait]
impl Handler<StatesMessage> for ExecutorActor {
    async fn handle(
        &mut self,
        msg: StatesMessage,
        _ctx: &mut ActorContext,
    ) -> Result<Vec<Option<State>>, anyhow::Error> {
        let statedb = self.moveos.moveos_resolver();
        statedb.get_states(msg.access_path)
    }
}

#[async_trait]
impl Handler<AnnotatedStatesMessage> for ExecutorActor {
    async fn handle(
        &mut self,
        msg: AnnotatedStatesMessage,
        _ctx: &mut ActorContext,
    ) -> Result<Vec<Option<AnnotatedState>>, anyhow::Error> {
        let statedb = self.moveos.moveos_resolver();
        statedb.get_annotated_states(msg.access_path)
    }
}

#[async_trait]
impl Handler<GetEventsByEventHandleMessage> for ExecutorActor {
    async fn handle(
        &mut self,
        msg: GetEventsByEventHandleMessage,
        _ctx: &mut ActorContext,
    ) -> Result<Vec<Option<AnnotatedMoveOSEvent>>> {
        let GetEventsByEventHandleMessage {
            event_handle_type,
            cursor,
            limit,
        } = msg;
        let event_store = self.moveos.event_store();
        let resolver = self.moveos.moveos_resolver();

        let event_handle_id = EventHandle::derive_event_handle_id(event_handle_type.clone());
        let events = event_store.get_events_by_event_handle_id(&event_handle_id, cursor, limit)?;

        // for ev in events
        let result = events
            .into_iter()
            // .enumerate()
            .map(|event| {
                let state = State::new(event.event_data.clone(), event.type_tag.clone());
                let annotated_event_data = MoveValueAnnotator::new(resolver)
                    .view_resource(&event_handle_type, state.value.as_slice())
                    .unwrap();
                Some(AnnotatedMoveOSEvent::new(
                    event,
                    annotated_event_data,
                    None,
                    None,
                ))
            })
            .collect();
        Ok(result)
    }
}

#[async_trait]
impl Handler<GetEventsMessage> for ExecutorActor {
    async fn handle(
        &mut self,
        msg: GetEventsMessage,
        _ctx: &mut ActorContext,
    ) -> Result<Vec<Option<AnnotatedMoveOSEvent>>> {
        let GetEventsMessage { filter } = msg;
        let event_store = self.moveos.event_store();
        let resolver = self.moveos.moveos_resolver();
        //TODO handle tx hash
        let mut result: Vec<Option<AnnotatedMoveOSEvent>> = Vec::new();
        let events = event_store.get_events_with_filter(filter)?;
        for ev in events
            .into_iter()
            .enumerate()
            .map(|(_i, event)| {
                let state = State::new(event.event_data.clone(), event.type_tag.clone());
                let struct_tag = as_struct_tag(event.type_tag.clone()).unwrap();
                let annotated_event_data = MoveValueAnnotator::new(resolver)
                    .view_resource(&struct_tag, state.value.as_slice())
                    .unwrap();
                AnnotatedMoveOSEvent::new(event, annotated_event_data, None, None)
            })
            .collect::<Vec<_>>()
        {
            result.push(Some(ev));
        }
        Ok(result)
    }
}
