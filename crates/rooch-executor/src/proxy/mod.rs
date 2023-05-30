// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::actor::{
    executor::ExecutorActor,
    messages::{
        ExecuteViewFunctionMessage, GetEventsByTxHashMessage, GetEventsMessage, GetResourceMessage,
        ObjectMessage, ValidateTransactionMessage,
    },
};
use anyhow::Result;
use coerce::actor::ActorRef;
use move_core_types::{
    account_address::AccountAddress, language_storage::StructTag, value::MoveValue,
};
use move_resource_viewer::AnnotatedMoveStruct;
use moveos::moveos::TransactionOutput;
use moveos_types::event_filter::{EventFilter, MoveOSEvent};
use moveos_types::{
    object::{AnnotatedObject, ObjectID},
    transaction::{AuthenticatableTransaction, FunctionCall, MoveOSTransaction},
};
use rooch_types::{transaction::TransactionInfo, H256};

#[derive(Clone)]
pub struct ExecutorProxy {
    pub actor: ActorRef<ExecutorActor>,
}

impl ExecutorProxy {
    pub fn new(actor: ActorRef<ExecutorActor>) -> Self {
        Self { actor }
    }

    pub async fn validate_transaction<T>(&self, tx: T) -> Result<MoveOSTransaction>
    where
        T: 'static + AuthenticatableTransaction + Send + Sync,
    {
        self.actor.send(ValidateTransactionMessage { tx }).await?
    }

    //TODO ensure the execute result
    pub async fn execute_transaction(
        &self,
        tx: MoveOSTransaction,
    ) -> Result<(TransactionOutput, TransactionInfo)> {
        let result = self
            .actor
            .send(crate::actor::messages::ExecuteTransactionMessage { tx })
            .await??;
        Ok((result.output, result.transaction_info))
    }

    pub async fn execute_view_function(&self, call: FunctionCall) -> Result<Vec<MoveValue>> {
        self.actor.send(ExecuteViewFunctionMessage { call }).await?
    }

    pub async fn get_resource(
        &self,
        address: AccountAddress,
        resource_type: StructTag,
    ) -> Result<Option<AnnotatedMoveStruct>> {
        self.actor
            .send(GetResourceMessage {
                address,
                resource_type,
            })
            .await?
    }

    pub async fn get_object(&self, object_id: ObjectID) -> Result<Option<AnnotatedObject>> {
        self.actor.send(ObjectMessage { object_id }).await?
    }

    pub async fn get_events_by_tx_hash(&self, tx_hash: H256) -> Result<Option<Vec<MoveOSEvent>>> {
        self.actor
            .send(GetEventsByTxHashMessage { tx_hash })
            .await?
    }

    pub async fn get_events(&self, filter: EventFilter) -> Result<Option<Vec<MoveOSEvent>>> {
        self.actor.send(GetEventsMessage { filter }).await?
    }
}
