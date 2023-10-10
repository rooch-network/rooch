// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

// Adapted from https://github.com/tomusdrw/rust-web3/blob/master/src/types/log.rs

use crate::jsonrpc_types::{bytes::Bytes, H256View, StrView};
use ethers::types::{H160, U256};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// A log produced by a transaction.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct Log {
    /// H160. the contract that emitted the log
    pub address: StrView<H160>,

    /// topics: Array of 0 to 4 32 Bytes of indexed log arguments.
    /// (In solidity: The first topic is the hash of the signature of the event
    /// (e.g. `Deposit(address,bytes32,uint256)`), except you declared the event
    /// with the anonymous specifier.)
    pub topics: Vec<H256View>,

    /// Data
    pub data: Bytes,

    /// Block Hash
    #[serde(rename = "blockHash")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub block_hash: Option<H256View>,

    /// Block Number
    #[serde(rename = "blockNumber")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub block_number: Option<StrView<u64>>,

    /// Transaction Hash
    #[serde(rename = "transactionHash")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub transaction_hash: Option<H256View>,

    /// Transaction Index
    #[serde(rename = "transactionIndex")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub transaction_index: Option<StrView<u64>>,

    /// Integer of the log index position in the block. None if it's a pending log.
    #[serde(rename = "logIndex")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub log_index: Option<StrView<U256>>,

    /// Integer of the transactions index position log was created from.
    /// None when it's a pending log.
    #[serde(rename = "transactionLogIndex")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub transaction_log_index: Option<StrView<U256>>,

    /// Log Type
    #[serde(rename = "logType")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub log_type: Option<String>,

    /// True when the log was removed, due to a chain reorganization.
    /// false if it's a valid log.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub removed: Option<bool>,
}
