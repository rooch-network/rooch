// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use anyhow::format_err;
use moveos_types::h256::H256;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::fmt::Display;
use std::str::FromStr;

/// The block in Rooch is constructed by the proposer, representing a batch of transactions
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct Block {
    /// The index if the block
    pub block_number: u128,
    /// How many transactions in the block
    pub batch_size: u64,
    /// The hash of the block, made by DA
    pub block_hash: H256,
    /// The previous tx accumulator root of the block
    pub prev_tx_accumulator_root: H256,
    /// The tx accumulator root after the last transaction append to the accumulator
    pub tx_accumulator_root: H256,
    /// The last transaction's state root
    pub state_root: H256,
    /// the block generate timestamp
    pub time: u64,
}

impl Block {
    pub fn new(
        block_number: u128,
        batch_size: u64,
        block_hash: H256,
        prev_tx_accumulator_root: H256,
        tx_accumulator_root: H256,
        state_root: H256,
        time: u64,
    ) -> Self {
        Self {
            block_number,
            batch_size,
            block_hash,
            prev_tx_accumulator_root,
            tx_accumulator_root,
            state_root,
            time,
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum BlockType {
    All,
    Finalized,
}

impl Display for BlockType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BlockType::All => write!(f, "all"),
            BlockType::Finalized => write!(f, "finalized"),
        }
    }
}

impl FromStr for BlockType {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> anyhow::Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "all" => Ok(BlockType::All),
            "finalized" => Ok(BlockType::Finalized),
            s => Err(format_err!("Invalid block type str: {}", s)),
        }
    }
}
