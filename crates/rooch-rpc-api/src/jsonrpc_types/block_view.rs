// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::jsonrpc_types::{H256View, StrView};
use rooch_types::block::Block;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct BlockView {
    /// The index if the block
    pub block_number: StrView<u128>,
    /// How many transactions in the block
    // pub batch_size: StrView<u64>,
    /// The hash of the block, made by DA
    pub block_hash: H256View,
    /// The previous tx accumulator root of the block
    // pub prev_tx_accumulator_root: H256View,
    /// The tx accumulator root after the last transaction append to the accumulator
    // pub tx_accumulator_root: H256View,
    /// The last transaction's state root
    // pub state_root: H256View,
    /// the block generate timestamp
    pub time: StrView<u64>,
}

impl From<Block> for BlockView {
    fn from(block: Block) -> Self {
        Self {
            block_number: block.block_number.into(),
            block_hash: block.block_hash.into(),
            time: block.time.into(),
        }
    }
}
