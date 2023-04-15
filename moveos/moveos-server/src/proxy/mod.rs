// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use coerce::actor::ActorRef;

use crate::{
    actor::{
        executor::ServerActor,
        messages::{ExecutionFunctionMessage, HelloMessage, PublishPackageMessage},
    },
    error::Error,
};

pub struct ServerProxy {
    pub actor: ActorRef<ServerActor>,
}

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

    pub async fn publish(&self, module: Vec<u8>) -> Result<String, Error> {
        self.actor
            .send(PublishPackageMessage { module })
            .await
            .map_err(|e| e.into())
    }

    pub async fn execute_function(&self, module: Vec<u8>) -> Result<String, Error> {
        self.actor
            .send(ExecutionFunctionMessage { module })
            .await
            .map_err(|e| e.into())
    }

    // TODO other functions
}
