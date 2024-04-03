// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use anyhow::Result;
use coerce::actor::message::Message;
use moveos_types::h256::H256;
use rooch_types::sequencer::SequencerOrder;
use rooch_types::transaction::{LedgerTransaction, LedgerTxData};
use serde::{Deserialize, Serialize};

/// Transaction Sequence Message
#[derive(Debug, Serialize, Deserialize)]
pub struct TransactionSequenceMessage {
    pub tx: LedgerTxData,
}

impl Message for TransactionSequenceMessage {
    type Result = Result<LedgerTransaction>;
}

/// Get Transaction By Hash Message
#[derive(Debug, Serialize, Deserialize)]
pub struct GetTransactionByHashMessage {
    pub hash: H256,
}

impl Message for GetTransactionByHashMessage {
    type Result = Result<Option<LedgerTransaction>>;
}

/// Get Transactions By Hash Message
#[derive(Debug, Serialize, Deserialize)]
pub struct GetTransactionsByHashMessage {
    pub tx_hashes: Vec<H256>,
}

impl Message for GetTransactionsByHashMessage {
    type Result = Result<Vec<Option<LedgerTransaction>>>;
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GetTxHashsMessage {
    pub tx_orders: Vec<u64>,
}

impl Message for GetTxHashsMessage {
    type Result = Result<Vec<Option<H256>>>;
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GetSequencerOrderMessage {}

impl Message for GetSequencerOrderMessage {
    type Result = Result<Option<SequencerOrder>>;
}
