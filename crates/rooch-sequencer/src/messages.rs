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

/// Get Transaction By Hash Message
#[derive(Debug)]
pub struct GetTransactionByHashMessage {
    pub hash: H256,
}

impl Message for GetTransactionByHashMessage {
    type Result = Result<Option<TypedTransaction>>;
}

/// Get Transactions By Hash Message
#[derive(Debug)]
pub struct GetTransactionsByHashMessage {
    pub tx_hashes: Vec<H256>,
}

impl Message for GetTransactionsByHashMessage {
    type Result = Result<Vec<Option<TypedTransaction>>>;
}
