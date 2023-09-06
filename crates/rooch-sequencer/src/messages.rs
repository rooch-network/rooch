// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use anyhow::Result;
use coerce::actor::message::Message;
use moveos_types::h256::H256;
use rooch_types::{
    transaction::{TransactionSequenceInfo, TypedTransaction},
};

/// Transaction Sequence Message
#[derive(Debug)]
pub struct TransactionSequenceMessage {
    pub tx: TypedTransaction,
}

impl Message for TransactionSequenceMessage {
    type Result = Result<TransactionSequenceInfo>;
}

/// Transaction By Hash Message
#[derive(Debug)]
pub struct TransactionByHashMessage {
    pub hash: H256,
}

impl Message for TransactionByHashMessage {
    type Result = Result<Option<TypedTransaction>>;
}

/// Transaction By Hash And Index Message
#[derive(Debug)]
pub struct TransactionByHashAndIndexMessage {
    pub hash: H256,
    pub index: u64,
}

impl Message for TransactionByHashAndIndexMessage {
    type Result = Result<TypedTransaction>;
}

#[derive(Debug)]
pub struct TransactionByIndicesMessage {
    pub start: u64,
    pub limit: u64,
}

impl Message for TransactionByIndicesMessage {
    type Result = Result<Vec<TypedTransaction>>;
}
