// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use anyhow::Result;
use coerce::actor::message::Message;
use rooch_types::transaction::TransactionSequenceMapping;
use rooch_types::{
    transaction::{TransactionSequenceInfo, TypedTransaction},
    H256,
};
use serde::{Deserialize, Serialize};

/// Transaction Sequence Message
#[derive(Debug, Serialize, Deserialize)]
pub struct TransactionSequenceMessage {
    pub tx: TypedTransaction,
}

impl Message for TransactionSequenceMessage {
    type Result = Result<TransactionSequenceInfo>;
}

/// Get Transaction By Hash Message
#[derive(Debug, Serialize, Deserialize)]
pub struct GetTransactionByHashMessage {
    pub hash: H256,
}

impl Message for GetTransactionByHashMessage {
    type Result = Result<Option<TypedTransaction>>;
}

/// Get Transactions By Hash Message
#[derive(Debug, Serialize, Deserialize)]
pub struct GetTransactionsByHashMessage {
    pub tx_hashes: Vec<H256>,
}

impl Message for GetTransactionsByHashMessage {
    type Result = Result<Vec<Option<TypedTransaction>>>;
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GetTxSequenceMappingByOrderMessage {
    pub cursor: Option<u128>,
    pub limit: u64,
}

impl Message for GetTxSequenceMappingByOrderMessage {
    type Result = Result<Vec<TransactionSequenceMapping>>;
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GetTxSequenceInfosMessage {
    pub orders: Vec<u128>,
}

impl Message for GetTxSequenceInfosMessage {
    type Result = Result<Vec<Option<TransactionSequenceInfo>>>;
}
