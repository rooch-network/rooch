// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use super::messages::{
    ExecuteTransactionMessage, ExecuteTransactionResult, ExecuteViewFunctionMessage,
    GetEventsByTxHashMessage, GetEventsMessage, GetResourceMessage, ObjectMessage,
    ValidateTransactionMessage,
};
use anyhow::Result;
use async_trait::async_trait;
use coerce::actor::{context::ActorContext, message::Handler, Actor};
use move_core_types::resolver::ResourceResolver;
use move_core_types::value::MoveValue;
use move_resource_viewer::{AnnotatedMoveStruct, MoveValueAnnotator};
use moveos::moveos::MoveOS;
use moveos_types::event_filter::MoveOSEvent;
use moveos_types::object::AnnotatedObject;
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
        //TODO implement a resolver cache for MoveValueAnnotator
        let annotator = MoveValueAnnotator::new(storage);
        let object_id = msg.object_id;
        storage
            .get(object_id)?
            .map(|state| state.as_annotated_object(&annotator))
            .transpose()
    }
}

#[async_trait]
impl Handler<GetEventsByTxHashMessage> for ExecutorActor {
    async fn handle(
        &mut self,
        msg: GetEventsByTxHashMessage,
        _ctx: &mut ActorContext,
    ) -> Result<Option<Vec<MoveOSEvent>>> {
        let GetEventsByTxHashMessage { tx_hash } = msg;
        let event_store = self.moveos.event_store();

        let mut result: Vec<MoveOSEvent> = Vec::new();
        for ev in event_store
            .get_events_by_tx_hash(&tx_hash)?
            .unwrap()
            .into_iter()
            .enumerate()
            .map(|(_i, event)| MoveOSEvent::try_from(event, tx_hash, None, None))
            .collect::<Vec<_>>()
        {
            result.push(ev.unwrap());
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
        for ev in event_store
            .get_events_with_filter(filter)?
            .unwrap()
            .into_iter()
            .enumerate()
            .map(|(_i, event)| MoveOSEvent::try_from(event, H256::zero(), None, None))
            .collect::<Vec<_>>()
        {
            result.push(ev.unwrap());
        }

        if !result.is_empty() {
            Ok(Some(result))
        } else {
            Ok(None)
        }
    }
}
