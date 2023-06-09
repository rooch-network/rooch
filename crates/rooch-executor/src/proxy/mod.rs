// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::actor::{
    executor::ExecutorActor,
    messages::{
        AnnotatedStatesMessage, ExecuteViewFunctionMessage, GetEventsByEventHandleMessage,
        GetEventsMessage, GetResourceMessage, ObjectMessage, StatesMessage,
        ValidateTransactionMessage,
    },
};
use anyhow::Result;
use coerce::actor::ActorRef;
use move_core_types::{account_address::AccountAddress, language_storage::StructTag};
use move_resource_viewer::AnnotatedMoveStruct;
use moveos_types::transaction::TransactionOutput;
use moveos_types::{
    access_path::AccessPath, function_return_value::AnnotatedFunctionReturnValue,
    transaction::VerifiedMoveOSTransaction,
};
use moveos_types::{
    event_filter::{EventFilter},
    event::{AnnotatedMoveOSEvent},
    state::{AnnotatedState, State},
};
use moveos_types::{
    object::{AnnotatedObject, ObjectID},
    transaction::{AuthenticatableTransaction, FunctionCall},
};
use rooch_types::transaction::TransactionExecutionInfo;

#[derive(Clone)]
pub struct ExecutorProxy {
    pub actor: ActorRef<ExecutorActor>,
}

impl ExecutorProxy {
    pub fn new(actor: ActorRef<ExecutorActor>) -> Self {
        Self { actor }
    }

    pub async fn validate_transaction<T>(&self, tx: T) -> Result<VerifiedMoveOSTransaction>
    where
        T: 'static + AuthenticatableTransaction + Send + Sync,
    {
        self.actor.send(ValidateTransactionMessage { tx }).await?
    }

    //TODO ensure the execute result
    pub async fn execute_transaction(
        &self,
        tx: VerifiedMoveOSTransaction,
    ) -> Result<(TransactionOutput, TransactionExecutionInfo)> {
        let result = self
            .actor
            .send(crate::actor::messages::ExecuteTransactionMessage { tx })
            .await??;
        Ok((result.output, result.transaction_info))
    }

    pub async fn execute_view_function(
        &self,
        call: FunctionCall,
    ) -> Result<Vec<AnnotatedFunctionReturnValue>> {
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

    pub async fn get_states(&self, access_path: AccessPath) -> Result<Vec<Option<State>>> {
        self.actor.send(StatesMessage { access_path }).await?
    }

    pub async fn get_annotated_states(
        &self,
        access_path: AccessPath,
    ) -> Result<Vec<Option<AnnotatedState>>> {
        self.actor
            .send(AnnotatedStatesMessage { access_path })
            .await?
    }

    pub async fn get_events_by_event_handle(
        &self,
        event_handle_type: StructTag,
        cursor: u64,
        limit: u64,
    ) -> Result<Vec<Option<AnnotatedMoveOSEvent>>> {
        self.actor
            .send(GetEventsByEventHandleMessage { event_handle_type, cursor, limit })
            .await?
    }

    pub async fn get_events(&self, filter: EventFilter) -> Result<Vec<Option<AnnotatedMoveOSEvent>>> {
        self.actor.send(GetEventsMessage { filter }).await?
    }
}
