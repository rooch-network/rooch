// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::actor::{
    executor::ExecutorActor,
    messages::{ObjectMessage, ResourceMessage, ValidateTransactionMessage, ViewFunctionMessage},
};
use anyhow::Result;
use coerce::actor::ActorRef;
use move_core_types::{
    account_address::AccountAddress,
    identifier::Identifier,
    language_storage::{ModuleId, TypeTag},
    value::MoveValue,
};
use moveos::moveos::TransactionOutput;
use moveos_types::{
    object::ObjectID,
    transaction::{AuthenticatableTransaction, MoveOSTransaction},
};
use rooch_types::transaction::TransactionInfo;

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

    pub async fn view(&self, payload: Vec<u8>) -> Result<Vec<MoveValue>> {
        self.actor.send(ViewFunctionMessage { payload }).await?
    }

    pub async fn resource(
        &self,
        address: AccountAddress,
        module: &ModuleId,
        resource: &Identifier,
        type_args: Vec<TypeTag>,
    ) -> Result<Option<String>> {
        self.actor
            .send(ResourceMessage {
                address,
                module: module.clone(),
                resource: resource.clone(),
                type_args,
            })
            .await?
    }

    pub async fn object(&self, object_id: ObjectID) -> Result<Option<String>> {
        self.actor.send(ObjectMessage { object_id }).await?
    }
}
