// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use super::messages::{
    AnnotatedStatesMessage, ExecuteViewFunctionMessage, GetAnnotatedEventsByEventHandleMessage,
    GetAnnotatedStatesByStateMessage, GetEventsByEventHandleMessage, RefreshStateMessage,
    StatesMessage,
};
use crate::actor::messages::{
    GetEventsByEventIDsMessage, GetTxExecutionInfosByHashMessage, ListAnnotatedStatesMessage,
    ListStatesMessage,
};
use anyhow::Result;
use async_trait::async_trait;
use coerce::actor::{context::ActorContext, message::Handler, Actor};
use move_resource_viewer::MoveValueAnnotator;
use moveos::moveos::MoveOS;
use moveos_store::transaction_store::TransactionStore;
use moveos_store::MoveOSStore;
use moveos_types::function_return_value::AnnotatedFunctionResult;
use moveos_types::function_return_value::AnnotatedFunctionReturnValue;
use moveos_types::moveos_std::event::EventHandle;
use moveos_types::moveos_std::event::{AnnotatedEvent, Event};
use moveos_types::state::{AnnotatedState, State};
use moveos_types::state_resolver::{AnnotatedStateReader, StateReader};
use moveos_types::transaction::TransactionExecutionInfo;
use rooch_genesis::RoochGenesis;
use rooch_store::RoochStore;
use rooch_types::framework::{system_post_execute_functions, system_pre_execute_functions};

pub struct ReaderExecutorActor {
    moveos: MoveOS,
    rooch_store: RoochStore,
}

impl ReaderExecutorActor {
    pub fn new(
        genesis: RoochGenesis,
        moveos_store: MoveOSStore,
        rooch_store: RoochStore,
    ) -> Result<Self> {
        let moveos = MoveOS::new(
            moveos_store,
            genesis.all_natives(),
            genesis.config.clone(),
            system_pre_execute_functions(),
            system_post_execute_functions(),
        )?;

        Ok(Self {
            moveos,
            rooch_store,
        })
    }

    pub fn get_rooch_store(&self) -> RoochStore {
        self.rooch_store.clone()
    }

    pub fn moveos(&self) -> &MoveOS {
        &self.moveos
    }
}

impl Actor for ReaderExecutorActor {}

#[async_trait]
impl Handler<ExecuteViewFunctionMessage> for ReaderExecutorActor {
    async fn handle(
        &mut self,
        msg: ExecuteViewFunctionMessage,
        _ctx: &mut ActorContext,
    ) -> Result<AnnotatedFunctionResult, anyhow::Error> {
        let resoler = self.moveos().moveos_resolver();

        let function_result = self.moveos().execute_view_function(msg.call);
        Ok(AnnotatedFunctionResult {
            vm_status: function_result.vm_status,
            return_values: match function_result.return_values {
                Some(values) => Some(
                    values
                        .into_iter()
                        .map(|v| {
                            let decoded_value = resoler.view_value(&v.type_tag, &v.value)?;
                            Ok(AnnotatedFunctionReturnValue {
                                value: v,
                                decoded_value,
                            })
                        })
                        .collect::<Result<Vec<AnnotatedFunctionReturnValue>, anyhow::Error>>()?,
                ),
                None => None,
            },
        })
    }
}

#[async_trait]
impl Handler<StatesMessage> for ReaderExecutorActor {
    async fn handle(
        &mut self,
        msg: StatesMessage,
        _ctx: &mut ActorContext,
    ) -> Result<Vec<Option<State>>, anyhow::Error> {
        let statedb = self.moveos().moveos_resolver();
        statedb.get_states(msg.access_path)
    }
}

#[async_trait]
impl Handler<AnnotatedStatesMessage> for ReaderExecutorActor {
    async fn handle(
        &mut self,
        msg: AnnotatedStatesMessage,
        _ctx: &mut ActorContext,
    ) -> Result<Vec<Option<AnnotatedState>>, anyhow::Error> {
        let statedb = self.moveos().moveos_resolver();
        statedb.get_annotated_states(msg.access_path)
    }
}

#[async_trait]
impl Handler<ListStatesMessage> for ReaderExecutorActor {
    async fn handle(
        &mut self,
        msg: ListStatesMessage,
        _ctx: &mut ActorContext,
    ) -> Result<Vec<(Vec<u8>, State)>, anyhow::Error> {
        let statedb = self.moveos().moveos_resolver();
        statedb.list_states(msg.access_path, msg.cursor, msg.limit)
    }
}

#[async_trait]
impl Handler<ListAnnotatedStatesMessage> for ReaderExecutorActor {
    async fn handle(
        &mut self,
        msg: ListAnnotatedStatesMessage,
        _ctx: &mut ActorContext,
    ) -> Result<Vec<(Vec<u8>, AnnotatedState)>, anyhow::Error> {
        let statedb = self.moveos().moveos_resolver();
        statedb.list_annotated_states(msg.access_path, msg.cursor, msg.limit)
    }
}

#[async_trait]
impl Handler<GetAnnotatedEventsByEventHandleMessage> for ReaderExecutorActor {
    async fn handle(
        &mut self,
        msg: GetAnnotatedEventsByEventHandleMessage,
        _ctx: &mut ActorContext,
    ) -> Result<Vec<AnnotatedEvent>> {
        let GetAnnotatedEventsByEventHandleMessage {
            event_handle_type,
            cursor,
            limit,
        } = msg;
        let event_store = self.moveos().event_store();
        let resolver = self.moveos().moveos_resolver();

        let event_handle_id = EventHandle::derive_event_handle_id(&event_handle_type);
        let events = event_store.get_events_by_event_handle_id(&event_handle_id, cursor, limit)?;

        events
            .into_iter()
            .map(|event| {
                let event_move_value = MoveValueAnnotator::new(resolver)
                    .view_resource(&event_handle_type, event.event_data())?;
                Ok(AnnotatedEvent::new(event, event_move_value))
            })
            .collect::<Result<Vec<_>>>()
    }
}

#[async_trait]
impl Handler<GetEventsByEventHandleMessage> for ReaderExecutorActor {
    async fn handle(
        &mut self,
        msg: GetEventsByEventHandleMessage,
        _ctx: &mut ActorContext,
    ) -> Result<Vec<Event>> {
        let GetEventsByEventHandleMessage {
            event_handle_type,
            cursor,
            limit,
        } = msg;
        let event_store = self.moveos().event_store();

        let event_handle_id = EventHandle::derive_event_handle_id(&event_handle_type);
        event_store.get_events_by_event_handle_id(&event_handle_id, cursor, limit)
    }
}

#[async_trait]
impl Handler<GetEventsByEventIDsMessage> for ReaderExecutorActor {
    async fn handle(
        &mut self,
        msg: GetEventsByEventIDsMessage,
        _ctx: &mut ActorContext,
    ) -> Result<Vec<Option<AnnotatedEvent>>> {
        let GetEventsByEventIDsMessage { event_ids } = msg;
        let event_store = self.moveos().event_store();
        let resolver = self.moveos().moveos_resolver();

        event_store
            .multi_get_events(event_ids)?
            .into_iter()
            .map(|v| match v {
                Some(event) => {
                    let event_move_value = MoveValueAnnotator::new(resolver)
                        .view_resource(event.event_type(), event.event_data())?;
                    Ok(Some(AnnotatedEvent::new(event, event_move_value)))
                }
                None => Ok(None),
            })
            .collect::<Result<Vec<_>>>()
    }
}

#[async_trait]
impl Handler<GetTxExecutionInfosByHashMessage> for ReaderExecutorActor {
    async fn handle(
        &mut self,
        msg: GetTxExecutionInfosByHashMessage,
        _ctx: &mut ActorContext,
    ) -> Result<Vec<Option<TransactionExecutionInfo>>> {
        let GetTxExecutionInfosByHashMessage { tx_hashes } = msg;
        self.moveos
            .transaction_store()
            .multi_get_tx_execution_infos(tx_hashes)
    }
}

#[async_trait]
impl Handler<GetAnnotatedStatesByStateMessage> for ReaderExecutorActor {
    async fn handle(
        &mut self,
        msg: GetAnnotatedStatesByStateMessage,
        _ctx: &mut ActorContext,
    ) -> Result<Vec<AnnotatedState>> {
        let GetAnnotatedStatesByStateMessage { states } = msg;
        let resolver = self.moveos().moveos_resolver();

        states
            .into_iter()
            .map(|state| {
                let annotate_state = MoveValueAnnotator::new(resolver)
                    .view_value(&state.value_type, &state.value)?;
                Ok(AnnotatedState::new(state, annotate_state))
            })
            .collect::<Result<Vec<_>>>()
    }
}

#[async_trait]
impl Handler<RefreshStateMessage> for ReaderExecutorActor {
    async fn handle(&mut self, msg: RefreshStateMessage, _ctx: &mut ActorContext) -> Result<()> {
        let RefreshStateMessage {
            new_state_root,
            is_upgrade,
        } = msg;
        self.moveos.refresh_state(new_state_root, is_upgrade)
    }
}
