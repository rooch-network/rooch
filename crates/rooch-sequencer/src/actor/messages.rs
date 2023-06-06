// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use anyhow::Result;
use coerce::actor::message::Message;
use rooch_types::{
    transaction::{TransactionSequenceInfo, TypedTransaction},
    H256,
};

/// Transaction Sequence Message
#[derive(Debug)]
pub struct TransactionSequenceMessage {
    pub tx: TypedTransaction,
}

impl Message for TransactionSequenceMessage {
    type Result = Result<TransactionSequenceInfo>;
}

pub struct TransactionByHashMessage {
    pub hash: H256,
}

impl Message for TransactionByHashMessage {
    type Result = Result<Option<TypedTransaction>>;
}

pub struct TransactionByIndexMessage {
    pub start: u64,
    pub limit: u64,
}

impl Message for TransactionByIndexMessage {
    type Result = Result<Option<Vec<TypedTransaction>>>;
}
