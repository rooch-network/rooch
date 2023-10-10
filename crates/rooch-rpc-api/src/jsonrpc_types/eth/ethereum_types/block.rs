// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

// Modified from <https://github.com/tomusdrw/rust-web3/blob/master/src/types/block.rs>

#[cfg(not(feature = "celo"))]
use crate::jsonrpc_types::bytes::Bytes;
use crate::jsonrpc_types::{H176View, H256View, H64View, StrView};

use super::{bloom::Bloom, other_fields::OtherFields, withdrawal::Withdrawal};
use move_core_types::u256::U256;
use schemars::JsonSchema;
use serde::{Deserialize, Deserializer, Serialize};
/// The block type returned from RPC calls.
///
/// This is generic over a `TX` type which will be either the hash or the full transaction,
/// i.e. `Block<TxHash>` or `Block<Transaction>`.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct Block<TX> {
    /// Hash of the block
    pub hash: Option<H256View>,
    /// Hash of the parent
    #[serde(default, rename = "parentHash")]
    pub parent_hash: H256View,
    /// Hash of the uncles
    #[cfg(not(feature = "celo"))]
    #[serde(default, rename = "sha3Uncles")]
    pub uncles_hash: H256View,
    /// Miner/author's address. None if pending.
    #[serde(default, rename = "miner")]
    pub author: Option<H176View>,
    /// State root hash
    #[serde(default, rename = "stateRoot")]
    pub state_root: H256View,
    /// Transactions root hash
    #[serde(default, rename = "transactionsRoot")]
    pub transactions_root: H256View,
    /// Transactions receipts root hash
    #[serde(default, rename = "receiptsRoot")]
    pub receipts_root: H256View,
    /// Block number. None if pending.
    pub number: Option<StrView<u64>>,
    /// Gas Used
    #[serde(rename = "gasUsed")]
    pub gas_used: StrView<U256>,
    /// Gas Limit
    #[cfg(not(feature = "celo"))]
    #[serde(rename = "gasLimit")]
    pub gas_limit: StrView<U256>,
    /// Extra data
    #[serde(default, rename = "extraData")]
    pub extra_data: Bytes,
    /// Logs bloom
    #[serde(rename = "logsBloom")]
    pub logs_bloom: Option<Bloom>,
    /// Timestamp
    pub timestamp: StrView<U256>,
    /// Difficulty
    #[cfg(not(feature = "celo"))]
    pub difficulty: StrView<U256>,
    /// Total difficulty
    #[serde(rename = "totalDifficulty")]
    pub total_difficulty: Option<StrView<U256>>,
    /// Seal fields
    #[serde(
        default,
        rename = "sealFields",
        deserialize_with = "deserialize_null_default"
    )]
    pub seal_fields: Vec<Bytes>,
    /// Uncles' hashes
    #[cfg(not(feature = "celo"))]
    #[serde(default)]
    pub uncles: Vec<H256View>,
    /// Transactions
    #[serde(bound = "TX: Serialize + serde::de::DeserializeOwned", default)]
    pub transactions: Vec<TX>,
    /// Size in bytes
    pub size: Option<StrView<U256>>,
    /// Mix Hash
    #[serde(rename = "mixHash")]
    #[cfg(not(feature = "celo"))]
    pub mix_hash: Option<H256View>,
    /// Nonce
    #[cfg(not(feature = "celo"))]
    pub nonce: Option<H64View>,
    /// Base fee per unit of gas (if past London)
    #[serde(rename = "baseFeePerGas")]
    pub base_fee_per_gas: Option<StrView<U256>>,
    /// Withdrawals root hash (if past Shanghai)
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        rename = "withdrawalsRoot"
    )]
    #[cfg(not(feature = "celo"))]
    pub withdrawals_root: Option<H256View>,
    /// Withdrawals (if past Shanghai)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[cfg(not(feature = "celo"))]
    pub withdrawals: Option<Vec<Withdrawal>>,

    #[cfg(feature = "celo")]
    #[cfg_attr(docsrs, doc(cfg(feature = "celo")))]
    /// The block's randomness
    pub randomness: Randomness,

    /// BLS signatures with a SNARK-friendly hash function
    #[cfg(feature = "celo")]
    #[cfg_attr(docsrs, doc(cfg(feature = "celo")))]
    #[serde(rename = "epochSnarkData", default)]
    pub epoch_snark_data: Option<EpochSnarkData>,

    /// Captures unknown fields such as additional fields used by L2s
    #[cfg(not(feature = "celo"))]
    #[serde(flatten)]
    pub other: OtherFields,
}

fn deserialize_null_default<'de, D, T>(deserializer: D) -> Result<T, D::Error>
where
    T: Default + Deserialize<'de>,
    D: Deserializer<'de>,
{
    let opt = Option::deserialize(deserializer)?;
    Ok(opt.unwrap_or_default())
}

/// A block number or tag.
#[derive(Copy, Clone, Debug, Default, PartialEq, Eq, Hash, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub enum BlockNumber {
    /// Latest block
    #[default]
    Latest,
    /// Finalized block accepted as canonical
    Finalized,
    /// Safe head block
    Safe,
    /// Earliest block (genesis)
    Earliest,
    /// Pending block (not yet part of the blockchain)
    Pending,
    /// Block by number from canon chain
    Number(StrView<u64>),
}

impl BlockNumber {
    /// Returns the numeric block number if explicitly set
    pub fn as_number(&self) -> Option<StrView<u64>> {
        match *self {
            BlockNumber::Number(num) => Some(num),
            _ => None,
        }
    }

    /// Returns `true` if a numeric block number is set
    pub fn is_number(&self) -> bool {
        matches!(self, BlockNumber::Number(_))
    }

    /// Returns `true` if it's "latest"
    pub fn is_latest(&self) -> bool {
        matches!(self, BlockNumber::Latest)
    }

    /// Returns `true` if it's "finalized"
    pub fn is_finalized(&self) -> bool {
        matches!(self, BlockNumber::Finalized)
    }

    /// Returns `true` if it's "safe"
    pub fn is_safe(&self) -> bool {
        matches!(self, BlockNumber::Safe)
    }

    /// Returns `true` if it's "pending"
    pub fn is_pending(&self) -> bool {
        matches!(self, BlockNumber::Pending)
    }

    /// Returns `true` if it's "earliest"
    pub fn is_earliest(&self) -> bool {
        matches!(self, BlockNumber::Earliest)
    }
}
