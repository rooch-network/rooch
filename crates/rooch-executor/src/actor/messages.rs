// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use anyhow::Result;
use coerce::actor::message::Message;
use moveos_types::access_path::AccessPath;
use moveos_types::event_filter::{EventFilter, MoveOSEvent};
use moveos_types::function_return_value::AnnotatedFunctionReturnValue;
use moveos_types::state::{AnnotatedState, State};
use moveos_types::transaction::TransactionOutput;
use moveos_types::transaction::VerifiedMoveOSTransaction;
use moveos_types::{
    object::ObjectID,
    transaction::{AuthenticatableTransaction, FunctionCall},
};
use rooch_types::transaction::TransactionExecutionInfo;
use serde::{Deserialize, Serialize};

#[derive(Debug)]
pub struct ValidateTransactionMessage<T> {
    pub tx: T,
}

impl<T> Message for ValidateTransactionMessage<T>
where
    T: 'static + AuthenticatableTransaction + Send + Sync,
{
    type Result = Result<VerifiedMoveOSTransaction>;
}

#[derive(Debug)]
pub struct ExecuteTransactionMessage {
    pub tx: VerifiedMoveOSTransaction,
}

pub struct ExecuteTransactionResult {
    pub output: TransactionOutput,
    pub transaction_info: TransactionExecutionInfo,
}

impl Message for ExecuteTransactionMessage {
    type Result = Result<ExecuteTransactionResult>;
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ExecuteViewFunctionMessage {
    pub call: FunctionCall,
}

impl Message for ExecuteViewFunctionMessage {
    type Result = Result<Vec<AnnotatedFunctionReturnValue>, anyhow::Error>;
}

#[derive(Debug, Serialize, Deserialize)]
pub struct StatesMessage {
    pub access_path: AccessPath,
}

impl Message for StatesMessage {
    type Result = Result<Vec<Option<State>>>;
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AnnotatedStatesMessage {
    pub access_path: AccessPath,
}

impl Message for AnnotatedStatesMessage {
    type Result = Result<Vec<Option<AnnotatedState>>>;
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
