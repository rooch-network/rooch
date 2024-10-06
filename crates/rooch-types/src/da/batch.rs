// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use moveos_types::h256::H256;
use serde::{Deserialize, Serialize};

#[derive(Eq, PartialEq, Hash, Deserialize, Serialize, Clone, Debug)]
/// The tx order range of the block.
pub struct BlockRange {
    /// The Rooch block number for DA, each batch maps to a block
    pub block_number: u128,
    /// The start tx order of the block (inclusive)
    pub tx_order_start: u64,
    /// The end tx order of the block (inclusive)
    pub tx_order_end: u64,
}

#[derive(Eq, PartialEq, Hash, Deserialize, Serialize, Clone, Debug)]
/// The state of the block submission.
pub struct BlockSubmitState {
    /// tx order range of the block
    pub block_range: BlockRange,
    /// submitted or not
    pub done: bool,
}

impl BlockSubmitState {
    /// Create a new BlockSubmitState
    pub fn new(block_number: u128, tx_order_start: u64, tx_order_end: u64) -> Self {
        Self {
            block_range: BlockRange {
                block_number,
                tx_order_start,
                tx_order_end,
            },
            done: false,
        }
    }
    pub fn new_done(block_number: u128, tx_order_start: u64, tx_order_end: u64) -> Self {
        Self {
            block_range: BlockRange {
                block_number,
                tx_order_start,
                tx_order_end,
            },
            done: true,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
/// A batch is a collection of transactions. It is the unit of data flow in DA Stream
pub struct DABatchMeta {
    /// tx order range of the block
    pub block_range: BlockRange,
    /// sha256h of tx_list_bytes
    pub tx_list_hash: H256,
    /// meta signature, signed by sequencer.
    pub signature: Vec<u8>,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
/// A batch is a collection of transactions. It is the unit of data flow in DA Stream
pub struct DABatch {
    /// The metadata of the batch
    pub meta: DABatchMeta,
    /// encoded tx(LedgerTransaction) list
    pub tx_list_bytes: Vec<u8>,
}

impl DABatch {
    pub fn new_no_sign(
        block_number: u128,
        tx_order_start: u64,
        tx_order_end: u64,
        tx_list_bytes: Vec<u8>,
        tx_list_hash: H256,
    ) -> Self {
        Self {
            meta: DABatchMeta {
                block_range: BlockRange {
                    block_number,
                    tx_order_start,
                    tx_order_end,
                },
                tx_list_hash,
                signature: vec![],
            },
            tx_list_bytes,
        }
    }

    pub fn new(
        block_number: u128,
        tx_order_start: u64,
        tx_order_end: u64,
        tx_list_bytes: Vec<u8>,
        tx_list_hash: H256,
        signature: Vec<u8>,
    ) -> Self {
        Self {
            meta: DABatchMeta {
                block_range: BlockRange {
                    block_number,
                    tx_order_start,
                    tx_order_end,
                },
                tx_list_hash,
                signature,
            },
            tx_list_bytes,
        }
    }
}
