// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use super::messages::{
    AnnotatedStatesMessage, ExecuteTransactionMessage, ExecuteTransactionResult,
    ExecuteViewFunctionMessage, GetEventsByEventHandleMessage, GetEventsMessage,
    GetResourceMessage, ObjectMessage, StatesMessage, ValidateTransactionMessage,
};
use anyhow::Result;
use async_trait::async_trait;
use coerce::actor::{context::ActorContext, message::Handler, Actor};
use move_core_types::value::MoveValue;
use move_resource_viewer::{AnnotatedMoveStruct, MoveValueAnnotator};
use moveos::moveos::MoveOS;
use moveos_store::state_store::state_view::{AnnotatedStateReader, StateReader};
use moveos_types::event_filter::MoveOSEvent;
use moveos_types::object::AnnotatedObject;
use moveos_types::state::{AnnotatedState, State};
use moveos_types::transaction::{AuthenticatableTransaction, MoveOSTransaction};
use rooch_types::transaction::TransactionInfo;
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
    ) -> Result<MoveOSTransaction> {
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
        let tx_hash = msg.tx.tx_hash;
        let output = self.moveos.execute(msg.tx)?;
        //TODO calculate event_root
        let event_root = H256::zero();
        let transaction_info = TransactionInfo::new(
            tx_hash,
            output.state_root,
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
    ) -> Result<Vec<MoveValue>, anyhow::Error> {
        let result = self.moveos.execute_view_function(msg.call)?;
        let mut output_values = vec![];
        for v in result.return_values {
            output_values.push(MoveValue::simple_deserialize(&v.0, &v.1)?);
        }

        Ok(output_values)
    }
}

#[async_trait]
impl Handler<GetResourceMessage> for ExecutorActor {
    async fn handle(
        &mut self,
        msg: GetResourceMessage,
        _ctx: &mut ActorContext,
    ) -> Result<Option<AnnotatedMoveStruct>> {
        let GetResourceMessage {
            address,
            resource_type,
        } = msg;
        let storage = self.moveos.state();
        //TODO use annotated state view
        storage
            .get_resource(&address, &resource_type)?
            .map(|data| MoveValueAnnotator::new(storage).view_resource(&resource_type, &data))
            .transpose()
    }
}

#[async_trait]
impl Handler<ObjectMessage> for ExecutorActor {
    async fn handle(
        &mut self,
        msg: ObjectMessage,
        _ctx: &mut ActorContext,
    ) -> Result<Option<AnnotatedObject>, anyhow::Error> {
        let storage = self.moveos.state();
        storage.get_annotated_object(msg.object_id)
    }
}

#[async_trait]
impl Handler<StatesMessage> for ExecutorActor {
    async fn handle(
        &mut self,
        msg: StatesMessage,
        _ctx: &mut ActorContext,
    ) -> Result<Vec<Option<State>>, anyhow::Error> {
        let statedb = self.moveos.state();
        statedb.get_states(&msg.access_path)
    }
}

#[async_trait]
impl Handler<AnnotatedStatesMessage> for ExecutorActor {
    async fn handle(
        &mut self,
        msg: AnnotatedStatesMessage,
        _ctx: &mut ActorContext,
    ) -> Result<Vec<Option<AnnotatedState>>, anyhow::Error> {
        let statedb = self.moveos.state();
        statedb.get_annotated_states(&msg.access_path)
    }
}

#[async_trait]
impl Handler<GetEventsByEventHandleMessage> for ExecutorActor {
    async fn handle(
        &mut self,
        msg: GetEventsByEventHandleMessage,
        _ctx: &mut ActorContext,
    ) -> Result<Option<Vec<MoveOSEvent>>> {
        let GetEventsByEventHandleMessage { event_handle_id } = msg;
        let event_store = self.moveos.event_store();

        let mut result: Vec<MoveOSEvent> = Vec::new();
        let events = event_store.get_events_by_event_handle_id(&event_handle_id)?;

        if Option::is_some(&events) {
            for ev in events
                .unwrap()
                .into_iter()
                .enumerate()
                .map(|(_i, event)| MoveOSEvent::try_from(event, None, None, None))
                .collect::<Vec<_>>()
            {
                result.push(ev.unwrap());
            }
        }

        if !result.is_empty() {
            Ok(Some(result))
        } else {
            Ok(None)
        }
    }
}

#[async_trait]
impl Handler<GetEventsMessage> for ExecutorActor {
    async fn handle(
        &mut self,
        msg: GetEventsMessage,
        _ctx: &mut ActorContext,
    ) -> Result<Option<Vec<MoveOSEvent>>> {
        let GetEventsMessage { filter } = msg;
        let event_store = self.moveos.event_store();
        //TODO handle tx hash
        let mut result: Vec<MoveOSEvent> = Vec::new();
        let events = event_store.get_events_with_filter(filter)?;
        if Option::is_some(&events) {
            for ev in events
                .unwrap()
                .into_iter()
                .enumerate()
                .map(|(_i, event)| MoveOSEvent::try_from(event, None, None, None))
                .collect::<Vec<_>>()
            {
                result.push(ev.unwrap());
            }
        }

        if !result.is_empty() {
            Ok(Some(result))
        } else {
            Ok(None)
        }
    }
}
