// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use moveos_types::h256::H256;
use serde::{Deserialize, Serialize};

/// The block in Rooch is constructed by the proposer, representing a batch of transactions
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct Block {
    /// The index if the block
    pub block_number: u128,
    /// How many transactions in the block
    pub batch_size: u64,
    /// The hash of the batch, made by DA
    pub batch_hash: H256,
    /// The previous tx accumulator root of the block
    pub prev_tx_accumulator_root: H256,
    /// The tx accumulator root after the last transaction append to the accumulator
    pub tx_accumulator_root: H256,
    /// The last transaction's state root
    pub state_root: H256,
}

impl Block {
    pub fn new(
        block_number: u128,
        batch_size: u64,
        batch_hash: H256,
        prev_tx_accumulator_root: H256,
        tx_accumulator_root: H256,
        state_root: H256,
    ) -> Self {
        Self {
            block_number,
            batch_size,
            batch_hash,
            prev_tx_accumulator_root,
            tx_accumulator_root,
            state_root,
        }
    }
}
