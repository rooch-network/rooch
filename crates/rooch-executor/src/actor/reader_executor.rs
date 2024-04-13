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
use moveos_types::moveos_std::object::RootObjectEntity;
use moveos_types::state::{AnnotatedState, State};
use moveos_types::state_resolver::RootObjectResolver;
use moveos_types::state_resolver::{AnnotatedStateKV, AnnotatedStateReader, StateKV, StateReader};
use moveos_types::transaction::TransactionExecutionInfo;
use rooch_genesis::RoochGenesis;
use rooch_store::RoochStore;
use rooch_types::framework::{system_post_execute_functions, system_pre_execute_functions};

pub struct ReaderExecutorActor {
    root: RootObjectEntity,
    moveos: MoveOS,
    moveos_store: MoveOSStore,
    rooch_store: RoochStore,
}

impl ReaderExecutorActor {
    pub fn new(
        root: RootObjectEntity,
        genesis: RoochGenesis,
        moveos_store: MoveOSStore,
        rooch_store: RoochStore,
    ) -> Result<Self> {
        let moveos = MoveOS::new(
            moveos_store.clone(),
            genesis.all_natives(),
            genesis.config.clone(),
            system_pre_execute_functions(),
            system_post_execute_functions(),
        )?;

        Ok(Self {
            root,
            moveos,
            moveos_store,
            rooch_store,
        })
    }

    pub fn get_rooch_store(&self) -> RoochStore {
        self.rooch_store.clone()
    }

    pub fn moveos(&self) -> &MoveOS {
        &self.moveos
    }

    pub fn refresh_state(&mut self, root: RootObjectEntity, is_upgrade: bool) -> Result<()> {
        self.root = root;
        self.moveos.flush_module_cache(is_upgrade)
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
        let resolver = RootObjectResolver::new(self.root.clone(), &self.moveos_store);
        let function_result = self
            .moveos()
            .execute_view_function(self.root.clone(), msg.call);
        Ok(AnnotatedFunctionResult {
            vm_status: function_result.vm_status,
            return_values: match function_result.return_values {
                Some(values) => Some(
                    values
                        .into_iter()
                        .map(|v| {
                            let decoded_value = resolver.view_value(&v.type_tag, &v.value)?;
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
        let resolver = RootObjectResolver::new(self.root.clone(), &self.moveos_store);
        resolver.get_states(msg.access_path)
    }
}

#[async_trait]
impl Handler<AnnotatedStatesMessage> for ReaderExecutorActor {
    async fn handle(
        &mut self,
        msg: AnnotatedStatesMessage,
        _ctx: &mut ActorContext,
    ) -> Result<Vec<Option<AnnotatedState>>, anyhow::Error> {
        let resolver = RootObjectResolver::new(self.root.clone(), &self.moveos_store);
        resolver.get_annotated_states(msg.access_path)
    }
}

#[async_trait]
impl Handler<ListStatesMessage> for ReaderExecutorActor {
    async fn handle(
        &mut self,
        msg: ListStatesMessage,
        _ctx: &mut ActorContext,
    ) -> Result<Vec<StateKV>, anyhow::Error> {
        let resolver = RootObjectResolver::new(self.root.clone(), &self.moveos_store);
        resolver.list_states(msg.access_path, msg.cursor, msg.limit)
    }
}

#[async_trait]
impl Handler<ListAnnotatedStatesMessage> for ReaderExecutorActor {
    async fn handle(
        &mut self,
        msg: ListAnnotatedStatesMessage,
        _ctx: &mut ActorContext,
    ) -> Result<Vec<AnnotatedStateKV>, anyhow::Error> {
        let resolver = RootObjectResolver::new(self.root.clone(), &self.moveos_store);
        resolver.list_annotated_states(msg.access_path, msg.cursor, msg.limit)
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
            descending_order,
        } = msg;
        let event_store = self.moveos().event_store();

        let resolver = RootObjectResolver::new(self.root.clone(), &self.moveos_store);

        let event_handle_id = EventHandle::derive_event_handle_id(&event_handle_type);
        let events = event_store.get_events_by_event_handle_id(
            &event_handle_id,
            cursor,
            limit,
            descending_order,
        )?;

        events
            .into_iter()
            .map(|event| {
                let event_move_value = MoveValueAnnotator::new(&resolver)
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
            descending_order,
        } = msg;
        let event_store = self.moveos().event_store();

        let event_handle_id = EventHandle::derive_event_handle_id(&event_handle_type);
        event_store.get_events_by_event_handle_id(&event_handle_id, cursor, limit, descending_order)
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
        let resolver = RootObjectResolver::new(self.root.clone(), &self.moveos_store);
        event_store
            .multi_get_events(event_ids)?
            .into_iter()
            .map(|v| match v {
                Some(event) => {
                    let event_move_value = MoveValueAnnotator::new(&resolver)
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
        let resolver = RootObjectResolver::new(self.root.clone(), &self.moveos_store);

        states
            .into_iter()
            .map(|state| {
                let annotate_state = MoveValueAnnotator::new(&resolver)
                    .view_value(&state.value_type, &state.value)?;
                Ok(AnnotatedState::new(state, annotate_state))
            })
            .collect::<Result<Vec<_>>>()
    }
}

#[async_trait]
impl Handler<RefreshStateMessage> for ReaderExecutorActor {
    async fn handle(&mut self, msg: RefreshStateMessage, _ctx: &mut ActorContext) -> Result<()> {
        let RefreshStateMessage { root, is_upgrade } = msg;
        self.refresh_state(root, is_upgrade)
    }
}
