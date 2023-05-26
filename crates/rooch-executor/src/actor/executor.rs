// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use super::messages::{
    ExecuteTransactionMessage, ExecuteTransactionResult, ObjectMessage, ResourceMessage,
    ValidateTransactionMessage, ViewFunctionMessage,
};
use anyhow::Result;
use async_trait::async_trait;
use coerce::actor::{context::ActorContext, message::Handler, Actor};
use move_core_types::language_storage::StructTag;
use move_core_types::resolver::ResourceResolver;
use move_core_types::value::MoveValue;
use move_resource_viewer::MoveValueAnnotator;
use moveos::moveos::MoveOS;
use moveos_types::object::RawObject;
use moveos_types::transaction::{AuthenticatableTransaction, MoveOSTransaction, ViewPayload};
use rooch_types::transaction::TransactionInfo;
use rooch_types::H256;

pub struct ExecutorActor {
    moveos: MoveOS,
}

impl ExecutorActor {
    pub fn new(moveos: MoveOS) -> Self {
        Self { moveos }
    }
}

impl Actor for ExecutorActor {}

#[async_trait]
impl<T> Handler<ValidateTransactionMessage<T>> for ExecutorActor
where
    T: 'static + AuthenticatableTransaction + Send + Sync,
{
    async fn handle(
        &mut self,
        msg: ValidateTransactionMessage<T>,
        _ctx: &mut ActorContext,
    ) -> Result<MoveOSTransaction> {
        self.moveos.validate(msg.tx)
    }
}

#[async_trait]
impl Handler<ExecuteTransactionMessage> for ExecutorActor {
    async fn handle(
        &mut self,
        msg: ExecuteTransactionMessage,
        _ctx: &mut ActorContext,
    ) -> Result<ExecuteTransactionResult> {
        let tx_hash = msg.tx.tx_hash;
        let output = self.moveos.execute(msg.tx)?;
        //TODO calculate event_root
        let event_root = H256::zero();
        let transaction_info = TransactionInfo::new(
            tx_hash,
            output.state_root,
            event_root,
            0,
            output.status.clone(),
        );
        Ok(ExecuteTransactionResult {
            output,
            transaction_info,
        })
    }
}

#[async_trait]
impl Handler<ViewFunctionMessage> for ExecutorActor {
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
impl Handler<ResourceMessage> for ExecutorActor {
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
impl Handler<ObjectMessage> for ExecutorActor {
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
