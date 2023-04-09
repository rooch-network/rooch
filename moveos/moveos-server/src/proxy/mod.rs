// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use coerce::actor::ActorRef;
use moveos::moveos::MoveOS;

use crate::{
    actor::{executor::ServerActor, messages::HelloMessage},
    error::Error,
};

pub struct ServerProxy {
    pub moveos: MoveOS,
    pub actor: ActorRef<ServerActor>,
}

impl ServerProxy {
    pub fn new(moveos: MoveOS, actor: ActorRef<ServerActor>) -> Self {
        Self { moveos, actor }
    }

    pub async fn echo(&self, msg: String) -> Result<String, Error> {
        self.actor
            .send(HelloMessage { msg })
            .await
            .map_err(|e| e.into())
    }

    // TODO other functions
}
