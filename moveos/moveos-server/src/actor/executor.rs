// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

/// Define Executor of MoveOS Server tasks
/// Step 1. Define a struct and impl the `Actor` for the struct
/// Step 2. Define the communication protocol messages between Actors
/// Step 3. Impl `Handler` with messages  for the Actor struct
use async_trait::async_trait;
use coerce::actor::{context::ActorContext, message::Handler, Actor};

use super::messages::{HelloMessage, SubmitTransactionMessage, ViewFunctionMessage};
use moveos::{
    moveos::MoveOS,
    types::transaction::{SimpleTransaction, ViewPayload},
};

pub struct ServerActor {
    moveos: MoveOS,
}

impl ServerActor {
    pub fn new(moveos: MoveOS) -> Self {
        Self { moveos }
    }
}

impl Actor for ServerActor {}

#[async_trait]
impl Handler<HelloMessage> for ServerActor {
    async fn handle(&mut self, msg: HelloMessage, ctx: &mut ActorContext) -> String {
        let actor_id = ctx.id();
        // Do something
        format!("response {}, {}", msg.msg, actor_id)
    }
}

#[async_trait]
impl Handler<SubmitTransactionMessage> for ServerActor {
    async fn handle(&mut self, msg: SubmitTransactionMessage, ctx: &mut ActorContext) -> String {
        // deserialize the payload
        let payload = bcs::from_bytes::<SimpleTransaction>(&msg.payload).unwrap();
        println!("sender: {:?}", payload.sender);
        let exec_result = self.moveos.execute(payload);
        // TODO: handle moveos execute result
        "ok".to_string()
    }
}

#[async_trait]
impl Handler<ViewFunctionMessage> for ServerActor {
    async fn handle(&mut self, msg: ViewFunctionMessage, ctx: &mut ActorContext) -> String {
        // deserialize the payload
        let payload = bcs::from_bytes::<ViewPayload>(&msg.payload).unwrap();
        let result = self
            .moveos
            .execute_view_function(
                &payload.function.module,
                &payload.function.function,
                payload.function.ty_args,
                payload.function.args,
            )
            .unwrap();
        println!("result: {:?}", result.return_values[0].0);
        "ok".to_string()
    }
}
