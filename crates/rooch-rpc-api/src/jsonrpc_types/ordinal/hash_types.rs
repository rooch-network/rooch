// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

// SPDX-License-Identifier: CC0-1.0

//! Bitcoin hash types.
//!
//! This module defines types for hashes used throughout the library. These
//! types are needed in order to avoid mixing data of the same hash format
//! (e.g. `SHA256d`) but of different meaning (such as transaction id, block
//! hash).
//!

use std::str::FromStr;

use crate::jsonrpc_types::StrView;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use bitcoin::hashes::sha256d;

pub type Sha256dHashView = StrView<sha256d::Hash>;

impl FromStr for Sha256dHashView {
    type Err = anyhow::Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(StrView(sha256d::Hash::from_str(s)?))
    }
}

impl From<Sha256dHashView> for sha256d::Hash {
    fn from(value: Sha256dHashView) -> Self {
        value.0
    }
}

impl std::fmt::Display for Sha256dHashView {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self.0)
    }
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq, JsonSchema)]
#[serde(rename_all = "lowercase")]
/// A bitcoin transaction hash/transaction ID.
///
/// For compatibility with the existing Bitcoin infrastructure and historical
/// and current versions of the Bitcoin Core software itself, this and
/// other [`sha256d::Hash`] types, are serialized in reverse
/// byte order when converted to a hex string via [`std::fmt::Display`] trait operations.
/// See [`hashes::Hash::DISPLAY_BACKWARD`] for more details.
pub struct TxidView(pub Sha256dHashView);

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq, JsonSchema)]
#[serde(rename_all = "lowercase")]
/// A bitcoin witness transaction ID.
pub struct WtxidView(pub Sha256dHashView);
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq, JsonSchema)]
#[serde(rename_all = "lowercase")]
/// A bitcoin block hash.
pub struct BlockHashView(pub Sha256dHashView);

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq, JsonSchema)]
#[serde(rename_all = "lowercase")]
/// A hash of the Merkle tree branch or root for transactions
pub struct TxMerkleNodeView(pub Sha256dHashView);
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq, JsonSchema)]
#[serde(rename_all = "lowercase")]
/// A hash corresponding to the Merkle tree root for witness data
pub struct WitnessMerkleNodeView(pub Sha256dHashView);
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq, JsonSchema)]
#[serde(rename_all = "lowercase")]
/// A hash corresponding to the witness structure commitment in the coinbase transaction
pub struct WitnessCommitmentView(pub Sha256dHashView);

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq, JsonSchema)]
#[serde(rename_all = "lowercase")]
/// Filter hash, as defined in BIP-157
pub struct FilterHashView(pub Sha256dHashView);
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq, JsonSchema)]
#[serde(rename_all = "lowercase")]
/// Filter header, as defined in BIP-157
pub struct FilterHeaderView(pub Sha256dHashView);
