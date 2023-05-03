// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use coerce::actor::ActorRef;
use move_core_types::value::MoveValue;

use crate::actor::{
    executor::ServerActor,
    messages::{
        HelloMessage, ObjectMessage, ResourceMessage, SubmitTransactionMessage, ViewFunctionMessage,
    },
};
use anyhow::{Error, Result};
pub struct ServerProxy {
    pub actor: ActorRef<ServerActor>,
}
use move_core_types::{
    account_address::AccountAddress,
    identifier::Identifier,
    language_storage::{ModuleId, TypeTag},
};
use moveos_types::object::ObjectID;

impl ServerProxy {
    pub fn new(actor: ActorRef<ServerActor>) -> Self {
        Self { actor }
    }

    pub async fn echo(&self, msg: String) -> Result<String, Error> {
        self.actor
            .send(HelloMessage { msg })
            .await
            .map_err(|e| e.into())
    }

    pub async fn submit_txn(&self, payload: Vec<u8>) -> Result<String, Error> {
        self.actor
            .send(SubmitTransactionMessage { payload })
            .await
            .map_err(|e| e.into())
    }

    pub async fn view(&self, payload: Vec<u8>) -> Result<Vec<MoveValue>, Error> {
        self.actor.send(ViewFunctionMessage { payload }).await?
    }

    pub async fn resource(
        &self,
        address: AccountAddress,
        module: &ModuleId,
        resource: &Identifier,
        type_args: Vec<TypeTag>,
    ) -> Result<String, Error> {
        self.actor
            .send(ResourceMessage {
                address,
                module: module.clone(),
                resource: resource.clone(),
                type_args,
            })
            .await?
    }

    pub async fn object(&self, object_id: ObjectID) -> Result<String, Error> {
        self.actor.send(ObjectMessage { object_id }).await?
    }
    // TODO other functions
}
