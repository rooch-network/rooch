// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::crypto::{RoochKeyPair, Signature};
use crate::transaction::LedgerTransaction;
use moveos_types::h256::{sha2_256_of, H256};
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
/// Meta of DA batch
pub struct DABatchMeta {
    /// tx order range of the block
    pub block_range: BlockRange,
    /// sha256h of encoded tx_list
    pub tx_list_hash: H256,
}

impl DABatchMeta {
    pub fn new(
        block_number: u128,
        tx_order_start: u64,
        tx_order_end: u64,
        tx_list_hash: H256,
    ) -> Self {
        Self {
            block_range: BlockRange {
                block_number,
                tx_order_start,
                tx_order_end,
            },
            tx_list_hash,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct SignedDABatchMeta {
    pub meta: DABatchMeta,
    pub signature: Vec<u8>,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
/// A batch is a collection of transactions. It is the unit of data flow in DA Stream
pub struct DABatch {
    /// The metadata of the batch
    pub meta: DABatchMeta,
    /// meta signature, signed by sequencer.
    pub meta_signature: Vec<u8>,
    /// encoded Vec<LedgerTransaction>
    pub tx_list_bytes: Vec<u8>,
}

impl DABatch {
    pub fn new(
        block_number: u128,
        tx_order_start: u64,
        tx_order_end: u64,
        tx_list: &Vec<LedgerTransaction>,
        sequencer_key: RoochKeyPair,
    ) -> Self {
        let tx_list_bytes = bcs::to_bytes(tx_list).expect("encode tx_list should success");
        let tx_list_hash = sha2_256_of(&tx_list_bytes);
        let batch_meta = DABatchMeta::new(block_number, tx_order_start, tx_order_end, tx_list_hash);
        let meta_bytes = bcs::to_bytes(&batch_meta).expect("encode batch_meta should success");
        let meta_hash = sha2_256_of(&meta_bytes);
        let meta_signature = Signature::sign(&meta_hash.0, &sequencer_key)
            .as_ref()
            .to_vec();

        Self {
            meta: batch_meta,
            meta_signature,
            tx_list_bytes,
        }
    }
}
