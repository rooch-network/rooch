// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use super::messages::{
    AnnotatedStatesMessage, ExecuteTransactionMessage, ExecuteTransactionResult,
    ExecuteViewFunctionMessage, GetEventsByEventHandleMessage, GetEventsMessage, StatesMessage,
    ValidateTransactionMessage,
};
use anyhow::Result;
use async_trait::async_trait;
use coerce::actor::{context::ActorContext, message::Handler, Actor};
use move_resource_viewer::MoveValueAnnotator;
use moveos::moveos::MoveOS;
use moveos_types::event::AnnotatedMoveOSEvent;
use moveos_types::event::EventHandle;
use moveos_types::function_return_value::AnnotatedFunctionReturnValue;
use moveos_types::move_types::as_struct_tag;
use moveos_types::state::{AnnotatedState, State};
use moveos_types::state_resolver::{AnnotatedStateReader, StateReader};
use moveos_types::transaction::{AuthenticatableTransaction, VerifiedMoveOSTransaction};
use rooch_types::transaction::TransactionExecutionInfo;
use rooch_types::H256;

pub struct ExecutorActor {
    moveos: MoveOS,
}

impl ExecutorActor {
    pub fn new(moveos: MoveOS) -> Self {
        Self { moveos }
    }
}

impl Actor for ExecutorActor {}

#[async_trait]
impl<T> Handler<ValidateTransactionMessage<T>> for ExecutorActor
where
    T: 'static + AuthenticatableTransaction + Send + Sync,
{
    async fn handle(
        &mut self,
        msg: ValidateTransactionMessage<T>,
        _ctx: &mut ActorContext,
    ) -> Result<VerifiedMoveOSTransaction> {
        self.moveos.validate(msg.tx)
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
        //TODO calculate event_root
        let event_root = H256::zero();
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
