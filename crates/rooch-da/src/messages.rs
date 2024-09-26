// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use anyhow::Result;
use bitcoin_hashes::sha256t_hash_newtype;
use coerce::actor::message::Message;
use serde::{Deserialize, Serialize};

use moveos_types::h256::H256;

sha256t_hash_newtype! {
    pub struct DABatchTag = hash_str("DABatch");


    #[hash_newtype(forward)]
    pub struct DABatchHash(_);
}

/// A batch is a collection of transactions. It is the unit of data flow in DA Stream
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct DABatch {
    pub meta: DABatchMeta,
    /// encoded tx(LedgerTransaction) list
    pub tx_list_bytes: Vec<u8>,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct DABatchMeta {
    /// The start tx order of the batch
    pub tx_order_start: u64,
    /// The end tx order of the batch
    pub tx_order_end: u64,
    /// How many transactions in the batch
    pub tx_count: u64,
    /// The previous tx accumulator root of the block
    pub prev_tx_accumulator_root: H256,
    /// The tx accumulator root after the last transaction append to the accumulator
    pub tx_accumulator_root: H256,
    /// sha256h of tx_list_bytes
    pub batch_hash: H256,
    /// meta signature, signed by sequencer.
    pub signature: Vec<u8>,
}

impl DABatch {
    pub fn new(
        tx_order_start: u64,
        tx_order_end: u64,
        tx_count: u64,
        prev_tx_accumulator_root: H256,
        tx_accumulator_root: H256,
        batch_hash: H256,
        tx_list_bytes: Vec<u8>,
    ) -> Self {
        assert_eq!(tx_count, (tx_order_end - tx_order_start) + 1);

        Self {
            meta: DABatchMeta {
                tx_order_start,
                tx_order_end,
                tx_count,
                prev_tx_accumulator_root,
                tx_accumulator_root,
                batch_hash,
                signature,
            },
            tx_list_bytes,
        }
    }
}

impl Message for DABatch {
    type Result = Result<()>;
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PutBatchInternalDAMessage {
    pub batch: DABatch,
    // TODO extra info
}

impl Message for PutBatchInternalDAMessage {
    type Result = Result<()>;
}
