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

/// Transaction By Hash Message
#[derive(Debug)]
pub struct TransactionByHashMessage {
    pub hash: H256,
}

impl Message for TransactionByHashMessage {
    type Result = Result<Option<TypedTransaction>>;
}

/// Transaction By Index Message
#[derive(Debug)]
pub struct GetTransactionsMessage {
    pub tx_hashes: Vec<H256>,
}

impl Message for GetTransactionsMessage {
    type Result = Result<Vec<Option<TypedTransaction>>>;
}
