// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use std::fmt::{self, Display};

use bitcoin::hashes::Hash;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use super::{hash_types::BlockHashView, network::NetworkView};

/// The uniquely identifying hash of the target blockchain.
#[derive(
    Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize, JsonSchema,
)]
pub struct ChainHashView([u8; 32]);

impl ChainHashView {
    // Mainnet value can be verified at https://github.com/lightning/bolts/blob/master/00-introduction.md
    /// `ChainHash` for mainnet bitcoin.
    pub const BITCOIN: Self = Self([
        111, 226, 140, 10, 182, 241, 179, 114, 193, 166, 162, 70, 174, 99, 247, 79, 147, 30, 131,
        101, 225, 90, 8, 156, 104, 214, 25, 0, 0, 0, 0, 0,
    ]);
    /// `ChainHash` for testnet bitcoin.
    pub const TESTNET: Self = Self([
        67, 73, 127, 215, 248, 38, 149, 113, 8, 244, 163, 15, 217, 206, 195, 174, 186, 121, 151,
        32, 132, 233, 14, 173, 1, 234, 51, 9, 0, 0, 0, 0,
    ]);
    /// `ChainHash` for signet bitcoin.
    pub const SIGNET: Self = Self([
        246, 30, 238, 59, 99, 163, 128, 164, 119, 160, 99, 175, 50, 178, 187, 201, 124, 159, 249,
        240, 31, 44, 66, 37, 233, 115, 152, 129, 8, 0, 0, 0,
    ]);
    /// `ChainHash` for regtest bitcoin.
    pub const REGTEST: Self = Self([
        6, 34, 110, 70, 17, 26, 11, 89, 202, 175, 18, 96, 67, 235, 91, 191, 40, 195, 79, 58, 94,
        51, 42, 31, 199, 178, 183, 60, 241, 136, 145, 15,
    ]);

    /// Returns the hash of the `network` genesis block for use as a chain hash.
    ///
    /// See [BOLT 0](https://github.com/lightning/bolts/blob/ffeece3dab1c52efdb9b53ae476539320fa44938/00-introduction.md#chain_hash)
    /// for specification.
    pub const fn using_genesis_block(network: NetworkView) -> Self {
        let hashes = [Self::BITCOIN, Self::TESTNET, Self::SIGNET, Self::REGTEST];
        hashes[network as usize]
    }

    /// Converts genesis block hash into `ChainHash`.
    pub fn from_genesis_block_hash(block_hash: BlockHashView) -> Self {
        ChainHashView(block_hash.0 .0.to_byte_array())
    }
}

/// Error in parsing network from chain hash.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub struct UnknownChainHashErrorView(pub ChainHashView);

impl Display for UnknownChainHashErrorView {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "unknown chain hash: {:?}", self.0)
    }
}

impl std::error::Error for UnknownChainHashErrorView {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        None
    }
}
