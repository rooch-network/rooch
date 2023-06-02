// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use anyhow::Result;
use coerce::actor::message::Message;
use move_core_types::{
    account_address::AccountAddress, language_storage::StructTag, value::MoveValue,
};
use move_resource_viewer::AnnotatedMoveStruct;
use moveos::moveos::TransactionOutput;
use moveos_types::event_filter::{EventFilter, MoveOSEvent};
use moveos_types::{
    object::{AnnotatedObject, ObjectID},
    transaction::{AuthenticatableTransaction, FunctionCall, MoveOSTransaction},
};
use rooch_types::transaction::TransactionInfo;
use serde::{Deserialize, Serialize};

#[derive(Debug)]
pub struct ValidateTransactionMessage<T> {
    pub tx: T,
}

impl<T> Message for ValidateTransactionMessage<T>
where
    T: 'static + AuthenticatableTransaction + Send + Sync,
{
    type Result = Result<MoveOSTransaction>;
}

#[derive(Debug)]
pub struct ExecuteTransactionMessage {
    pub tx: MoveOSTransaction,
}

pub struct ExecuteTransactionResult {
    pub output: TransactionOutput,
    pub transaction_info: TransactionInfo,
}

impl Message for ExecuteTransactionMessage {
    type Result = Result<ExecuteTransactionResult>;
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ExecuteViewFunctionMessage {
    pub call: FunctionCall,
}

impl Message for ExecuteViewFunctionMessage {
    type Result = Result<Vec<MoveValue>, anyhow::Error>;
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GetResourceMessage {
    pub address: AccountAddress,
    pub resource_type: StructTag,
}

impl Message for GetResourceMessage {
    type Result = Result<Option<AnnotatedMoveStruct>>;
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ObjectMessage {
    pub object_id: ObjectID,
}

impl Message for ObjectMessage {
    type Result = Result<Option<AnnotatedObject>>;
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GetEventsByEventHandleMessage {
    pub event_handle_id: ObjectID,
}

impl Message for GetEventsByEventHandleMessage {
    type Result = Result<Option<Vec<MoveOSEvent>>>;
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GetEventsMessage {
    pub filter: EventFilter,
}

impl Message for GetEventsMessage {
    type Result = Result<Option<Vec<MoveOSEvent>>>;
}
