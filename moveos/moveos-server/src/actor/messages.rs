// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use coerce::actor::message::Message;
use move_core_types::{
    account_address::AccountAddress,
    identifier::Identifier,
    language_storage::{ModuleId, TypeTag},
    value::MoveValue,
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct HelloMessage {
    pub msg: String,
}

impl Message for HelloMessage {
    type Result = String;
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SubmitTransactionMessage {
    pub payload: Vec<u8>,
}

impl Message for SubmitTransactionMessage {
    type Result = String;
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
