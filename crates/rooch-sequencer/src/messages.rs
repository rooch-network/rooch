// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use anyhow::Result;
use coerce::actor::message::Message;
use moveos_types::h256::H256;
use rooch_types::sequencer::SequencerOrder;
use rooch_types::transaction::TransactionSequenceInfoMapping;
use rooch_types::transaction::{RoochTransaction, TransactionSequenceInfo};
use serde::{Deserialize, Serialize};

/// Transaction Sequence Message
#[derive(Debug, Serialize, Deserialize)]
pub struct TransactionSequenceMessage {
    pub tx: RoochTransaction,
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
    type Result = Result<Option<RoochTransaction>>;
}

/// Get Transactions By Hash Message
#[derive(Debug, Serialize, Deserialize)]
pub struct GetTransactionsByHashMessage {
    pub tx_hashes: Vec<H256>,
}

impl Message for GetTransactionsByHashMessage {
    type Result = Result<Vec<Option<RoochTransaction>>>;
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GetTxSequenceInfoMappingByOrderMessage {
    pub tx_orders: Vec<u64>,
}

impl Message for GetTxSequenceInfoMappingByOrderMessage {
    type Result = Result<Vec<Option<TransactionSequenceInfoMapping>>>;
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GetTxSequenceInfoMappingByHashMessage {
    pub tx_hashes: Vec<H256>,
}

impl Message for GetTxSequenceInfoMappingByHashMessage {
    type Result = Result<Vec<Option<TransactionSequenceInfoMapping>>>;
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GetTxSequenceInfosMessage {
    pub orders: Vec<u64>,
}

impl Message for GetTxSequenceInfosMessage {
    type Result = Result<Vec<Option<TransactionSequenceInfo>>>;
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GetSequencerOrderMessage {}

impl Message for GetSequencerOrderMessage {
    type Result = Result<Option<SequencerOrder>>;
}
