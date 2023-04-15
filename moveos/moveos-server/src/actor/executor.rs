// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

/// Define Executor of MoveOS Server tasks
/// Step 1. Define a struct and impl the `Actor` for the struct
/// Step 2. Define the communication protocol messages between Actors
/// Step 3. Impl `Handler` with messages  for the Actor struct
use async_trait::async_trait;
use coerce::actor::{context::ActorContext, message::Handler, Actor};

use super::messages::{ExecutionFunctionMessage, HelloMessage, PublishPackageMessage};
use moveos::moveos::MoveOS;

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
impl Handler<PublishPackageMessage> for ServerActor {
    async fn handle(&mut self, msg: PublishPackageMessage, ctx: &mut ActorContext) -> String {
        // TODO deserialize module from bytes
        println!("actor(#{:?} received msg: {:?}", ctx.id(), msg);

        let txn = MoveOS::build_publish_txn(msg.module).unwrap();

        // TODO execute should return execute result, not unit
        let _resp = self.moveos.execute(txn);

        // TODO shoud format the response to json format
        "success".to_string()
    }
}

#[async_trait]
impl Handler<ExecutionFunctionMessage> for ServerActor {
    async fn handle(&mut self, msg: ExecutionFunctionMessage, ctx: &mut ActorContext) -> String {
        // TODO shoule use tracing
        println!("actor(#{:?} received msg: {:?}", ctx.id(), msg);
        let txn = MoveOS::build_function_txn(msg.module).unwrap();

        // TODO execute should return execute result, not unit
        let _resp = self.moveos.execute(txn);

        // TODO shoud format the response to json format
        "success".to_string()
    }
}
