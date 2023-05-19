// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use anyhow::Result;
use coerce::actor::message::Message;
use move_core_types::{
    account_address::AccountAddress,
    identifier::Identifier,
    language_storage::{ModuleId, TypeTag},
    value::MoveValue,
};
use moveos::moveos::TransactionOutput;
use moveos_types::{
    object::ObjectID,
    transaction::{AuthenticatableTransaction, MoveOSTransaction},
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
pub struct ViewFunctionMessage {
    pub payload: Vec<u8>,
}

impl Message for ViewFunctionMessage {
    type Result = Result<Vec<MoveValue>, anyhow::Error>;
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ResourceMessage {
    pub address: AccountAddress,
    pub module: ModuleId,
    pub resource: Identifier,
    pub type_args: Vec<TypeTag>,
}

impl Message for ResourceMessage {
    type Result = Result<String, anyhow::Error>;
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ObjectMessage {
    pub object_id: ObjectID,
}

impl Message for ObjectMessage {
    type Result = Result<String, anyhow::Error>;
}
