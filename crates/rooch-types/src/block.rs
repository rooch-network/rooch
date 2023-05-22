// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use serde::{Deserialize, Serialize};

use crate::H256;

/// The block in Rooch is constructed by the proposer, representing a batch of transactions
/// How many transactions in the block is determined by the proposer
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct Block {
    /// The index if the block
    //TODO should we use the U256?
    pub block_number: u128,
    /// How many transactions in the block
    pub batch_size: u64,
    /// The previous tx accumulator root of the block
    pub prev_tx_accumulator_root: H256,
    /// The tx accumulator root after the last transaction append to the accumulator
    pub tx_accumulator_root: H256,
    /// The all transaction's state root
    //TODO should we keep all the state root in the block?
    pub state_roots: Vec<H256>,
}

impl Block {
    pub fn new(
        block_number: u128,
        batch_size: u64,
        prev_tx_accumulator_root: H256,
        tx_accumulator_root: H256,
        state_roots: Vec<H256>,
    ) -> Self {
        Self {
            block_number,
            batch_size,
            prev_tx_accumulator_root,
            tx_accumulator_root,
            state_roots,
        }
    }
}
