// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

/// Define Executor of MoveOS Server tasks
/// Step 1. Define a struct and impl the `Actor` for the struct
/// Step 2. Define the communication protocol messages between Actors
/// Step 3. Impl `Handler` with messages  for the Actor struct
use async_trait::async_trait;
use coerce::actor::{context::ActorContext, message::Handler, Actor};

use super::messages::{
    HelloMessage, ObjectMessage, ResourceMessage, SubmitTransactionMessage, ViewFunctionMessage,
};
use move_core_types::{language_storage::StructTag, resolver::ResourceResolver, value::MoveValue};
use move_resource_viewer::MoveValueAnnotator;
use moveos::{
    moveos::MoveOS,
    types::transaction::{SimpleTransaction, ViewPayload},
};
use moveos_types::object::RawObject;

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
    async fn handle(&mut self, msg: SubmitTransactionMessage, _ctx: &mut ActorContext) -> String {
        // deserialize the payload
        let payload = bcs::from_bytes::<SimpleTransaction>(&msg.payload).unwrap();
        println!("sender: {:?}", payload.sender);
        let exec_result = self.moveos.execute(payload);
        match exec_result {
            Ok(_) => "ok".to_string(),
            Err(e) => {
                println!("{:?}", e);
                "error".to_string()
            }
        }
        // "ok".to_string()
    }
}

#[async_trait]
impl Handler<ViewFunctionMessage> for ServerActor {
    async fn handle(
        &mut self,
        msg: ViewFunctionMessage,
        _ctx: &mut ActorContext,
    ) -> Result<Vec<MoveValue>, anyhow::Error> {
        // deserialize the payload
        let payload = bcs::from_bytes::<ViewPayload>(&msg.payload)?;
        let result = self.moveos.execute_view_function(
            &payload.function.module,
            &payload.function.function,
            payload.function.ty_args,
            payload.function.args,
        )?;
        let mut output_values = vec![];
        for v in result.return_values {
            output_values.push(MoveValue::simple_deserialize(&v.0, &v.1)?);
        }

        Ok(output_values)
    }
}

#[async_trait]
impl Handler<ResourceMessage> for ServerActor {
    async fn handle(
        &mut self,
        msg: ResourceMessage,
        _ctx: &mut ActorContext,
    ) -> Result<String, anyhow::Error> {
        let ResourceMessage {
            address,
            module,
            resource,
            type_args,
        } = msg;
        let tag = StructTag {
            address: *module.address(),
            module: module.name().to_owned(),
            name: resource,
            type_params: type_args,
        };
        let storage = self.moveos.state();
        match storage.get_resource(&address, &tag)? {
            None => Ok("[No Resource Exists]".to_owned()),
            Some(data) => {
                let annotated = MoveValueAnnotator::new(storage).view_resource(&tag, &data)?;
                Ok(format!("{}", annotated))

                // MoveValue::try_from();
            }
        }
    }
}

#[async_trait]
impl Handler<ObjectMessage> for ServerActor {
    async fn handle(
        &mut self,
        msg: ObjectMessage,
        _ctx: &mut ActorContext,
    ) -> Result<String, anyhow::Error> {
        let object_id = msg.object_id;
        let object: Option<RawObject> = self.moveos.state().get_as_raw_object(object_id)?;
        let object =
            object.ok_or_else(|| anyhow::anyhow!("Object with id {} not found", object_id))?;

        //TODO print more info about object
        // let annotated = MoveValueAnnotator::new(self.moveos.state())
        //     .view_resource(&move_object.type_, &move_object.contents)?;
        Ok(format!("{:?}", object))
    }
}
