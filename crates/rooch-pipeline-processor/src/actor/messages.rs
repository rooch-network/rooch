// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use anyhow::Result;
use coerce::actor::message::Message;
use rooch_types::transaction::{
    ExecuteTransactionResponse, L1BlockWithBody, L1Transaction, RoochTransaction,
};

#[derive(Clone)]
pub struct ExecuteL2TxMessage {
    pub tx: RoochTransaction,
}

impl Message for ExecuteL2TxMessage {
    type Result = Result<ExecuteTransactionResponse>;
}

#[derive(Clone)]
pub struct ExecuteL1BlockMessage {
    pub tx: L1BlockWithBody,
}

impl Message for ExecuteL1BlockMessage {
    type Result = Result<ExecuteTransactionResponse>;
}

#[derive(Clone)]
pub struct ExecuteL1TxMessage {
    pub tx: L1Transaction,
}

impl Message for ExecuteL1TxMessage {
    type Result = Result<ExecuteTransactionResponse>;
}
