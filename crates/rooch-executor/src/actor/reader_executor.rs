// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use super::messages::{
    AnnotatedStatesMessage, CheckStateChangeSetsMessage, ExecuteViewFunctionMessage,
    GetAnnotatedEventsByEventHandleMessage, GetAnnotatedEventsByEventIDsMessage,
    GetEventsByEventHandleMessage, GetStateChangeSetsMessage, RefreshStateMessage, StatesMessage,
};
use crate::actor::messages::{
    GetEventsByEventIDsMessage, GetTxExecutionInfosByHashMessage, ListAnnotatedStatesMessage,
    ListStatesMessage,
};
use anyhow::Result;
use async_trait::async_trait;
use coerce::actor::{context::ActorContext, message::Handler, Actor, LocalActorRef};
use move_resource_viewer::MoveValueAnnotator;
use moveos::moveos::{MoveOS, MoveOSCacheManager};
use moveos_eventbus::bus::EventData;
use moveos_store::transaction_store::TransactionStore;
use moveos_store::MoveOSStore;
use moveos_types::function_return_value::AnnotatedFunctionResult;
use moveos_types::function_return_value::AnnotatedFunctionReturnValue;
use moveos_types::moveos_std::event::EventHandle;
use moveos_types::moveos_std::event::{AnnotatedEvent, Event};
use moveos_types::moveos_std::object::ObjectMeta;
use moveos_types::state::{AnnotatedState, ObjectState, StateChangeSetExt};
use moveos_types::state_resolver::RootObjectResolver;
use moveos_types::state_resolver::{AnnotatedStateKV, AnnotatedStateReader, StateKV, StateReader};
use moveos_types::transaction::TransactionExecutionInfo;
use rooch_genesis::FrameworksGasParameters;
use rooch_notify::actor::NotifyActor;
use rooch_notify::event::GasUpgradeEvent;
use rooch_notify::messages::NotifyActorSubscribeMessage;
use rooch_store::RoochStore;
use rooch_types::framework::{system_post_execute_functions, system_pre_execute_functions};

pub struct ReaderExecutorActor {
    root: ObjectMeta,
    moveos: MoveOS,
    moveos_store: MoveOSStore,
    rooch_store: RoochStore,
    notify_actor: Option<LocalActorRef<NotifyActor>>,
    global_cache_manager: MoveOSCacheManager,
}

impl ReaderExecutorActor {
    pub fn new(
        root: ObjectMeta,
        moveos_store: MoveOSStore,
        rooch_store: RoochStore,
        notify_actor: Option<LocalActorRef<NotifyActor>>,
        global_cache_manager: MoveOSCacheManager,
    ) -> Result<Self> {
        let moveos = MoveOS::new(
            moveos_store.clone(),
            system_pre_execute_functions(),
            system_post_execute_functions(),
            global_cache_manager.clone(),
        )?;

        Ok(Self {
            root,
            moveos,
            moveos_store,
            rooch_store,
            notify_actor,
            global_cache_manager,
        })
    }

    pub async fn subscribe_event(
        &self,
        notify_actor_ref: LocalActorRef<NotifyActor>,
        executor_actor_ref: LocalActorRef<ReaderExecutorActor>,
    ) {
        let gas_upgrade_event = GasUpgradeEvent::default();
        let actor_subscribe_message = NotifyActorSubscribeMessage::new(
            gas_upgrade_event,
            "read-executor".to_string(),
            Box::new(executor_actor_ref),
        );
        let _ = notify_actor_ref.send(actor_subscribe_message).await;
    }

    pub fn get_rooch_store(&self) -> RoochStore {
        self.rooch_store.clone()
    }

    pub fn moveos(&self) -> &MoveOS {
        &self.moveos
    }

    pub fn refresh_state(&mut self, root: ObjectMeta, is_upgrade: bool) -> Result<()> {
        self.root = root;
        self.moveos.flush_module_cache(is_upgrade)
    }
}

#[async_trait]
impl Actor for ReaderExecutorActor {
    async fn started(&mut self, ctx: &mut ActorContext) {
        let local_actor_ref: LocalActorRef<Self> = ctx.actor_ref();
        if let Some(notify_actor) = self.notify_actor.clone() {
            let _ = self.subscribe_event(notify_actor, local_actor_ref).await;
        }
    }
}

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
    ) -> Result<Vec<Option<ObjectState>>, anyhow::Error> {
        let resolver = if let Some(state_root) = msg.state_root {
            let root_object_meta = ObjectMeta::root_metadata(state_root, 0);
            RootObjectResolver::new(root_object_meta, &self.moveos_store)
        } else {
            RootObjectResolver::new(self.root.clone(), &self.moveos_store)
        };
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
        let resolver = if let Some(state_root) = msg.state_root {
            let root_object_meta = ObjectMeta::root_metadata(state_root, 0);
            RootObjectResolver::new(root_object_meta, &self.moveos_store)
        } else {
            RootObjectResolver::new(self.root.clone(), &self.moveos_store)
        };
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
        let resolver = if let Some(state_root) = msg.state_root {
            let root_object_meta = ObjectMeta::root_metadata(state_root, 0);
            RootObjectResolver::new(root_object_meta, &self.moveos_store)
        } else {
            RootObjectResolver::new(self.root.clone(), &self.moveos_store)
        };
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
        let resolver = if let Some(state_root) = msg.state_root {
            let root_object_meta = ObjectMeta::root_metadata(state_root, 0);
            RootObjectResolver::new(root_object_meta, &self.moveos_store)
        } else {
            RootObjectResolver::new(self.root.clone(), &self.moveos_store)
        };
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
impl Handler<GetAnnotatedEventsByEventIDsMessage> for ReaderExecutorActor {
    async fn handle(
        &mut self,
        msg: GetAnnotatedEventsByEventIDsMessage,
        _ctx: &mut ActorContext,
    ) -> Result<Vec<Option<AnnotatedEvent>>> {
        let GetAnnotatedEventsByEventIDsMessage { event_ids } = msg;
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
impl Handler<GetEventsByEventIDsMessage> for ReaderExecutorActor {
    async fn handle(
        &mut self,
        msg: GetEventsByEventIDsMessage,
        _ctx: &mut ActorContext,
    ) -> Result<Vec<Option<Event>>> {
        let GetEventsByEventIDsMessage { event_ids } = msg;
        let event_store = self.moveos().event_store();
        event_store.multi_get_events(event_ids)
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
impl Handler<RefreshStateMessage> for ReaderExecutorActor {
    async fn handle(&mut self, msg: RefreshStateMessage, _ctx: &mut ActorContext) -> Result<()> {
        let RefreshStateMessage { root, is_upgrade } = msg;
        self.refresh_state(root, is_upgrade)
    }
}

#[async_trait]
impl Handler<EventData> for ReaderExecutorActor {
    async fn handle(&mut self, message: EventData, _ctx: &mut ActorContext) -> Result<()> {
        if let Ok(_gas_upgrade_msg) = message.data.downcast::<GasUpgradeEvent>() {
            tracing::info!("ReadExecutorActor: Reload the MoveOS instance...");

            self.moveos = MoveOS::new(
                self.moveos_store.clone(),
                system_pre_execute_functions(),
                system_post_execute_functions(),
                self.global_cache_manager.clone(),
            )?;
        }
        Ok(())
    }
}

#[async_trait]
impl Handler<GetStateChangeSetsMessage> for ReaderExecutorActor {
    async fn handle(
        &mut self,
        msg: GetStateChangeSetsMessage,
        _ctx: &mut ActorContext,
    ) -> Result<Vec<Option<StateChangeSetExt>>> {
        let GetStateChangeSetsMessage { tx_orders } = msg;
        self.rooch_store
            .state_store
            .multi_get_state_change_set(tx_orders)
    }
}

#[async_trait]
impl Handler<CheckStateChangeSetsMessage> for ReaderExecutorActor {
    async fn handle(
        &mut self,
        msg: CheckStateChangeSetsMessage,
        _ctx: &mut ActorContext,
    ) -> Result<Vec<u64>> {
        let CheckStateChangeSetsMessage { tx_orders } = msg;
        self.rooch_store
            .state_store
            .check_state_change_set(tx_orders)
    }
}
