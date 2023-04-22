// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use coerce::actor::ActorRef;

use crate::{
    actor::{
        executor::ServerActor,
        messages::{HelloMessage, SubmitTransactionMessage},
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

    pub async fn submit_txn(&self, payload: Vec<u8>) -> Result<String, Error> {
        self.actor
            .send(SubmitTransactionMessage { payload })
            .await
            .map_err(|e| e.into())
    }
    // TODO other functions
}
