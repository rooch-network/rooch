// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use super::{constants::ChainHashView, p2p::MagicView};

/// An error in parsing network string.
#[derive(Debug, Clone, PartialEq, Eq, JsonSchema)]
#[non_exhaustive]
pub struct ParseNetworkErrorView(String);

/// The cryptocurrency network to act on.
#[derive(
    Copy, PartialEq, Eq, PartialOrd, Ord, Clone, Hash, Debug, Serialize, Deserialize, JsonSchema,
)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "serde", serde(crate = "actual_serde"))]
#[cfg_attr(feature = "serde", serde(rename_all = "lowercase"))]
#[cfg_attr(
    feature = "serde",
    serde(rename_all = "deserialize_with = deserialize_bip70_network")
)]
#[non_exhaustive]
pub enum NetworkView {
    /// Mainnet Bitcoin.
    Bitcoin,
    /// Bitcoin's testnet network.
    Testnet,
    /// Bitcoin's signet network.
    Signet,
    /// Bitcoin's regtest network.
    Regtest,
}

impl NetworkView {
    /// Creates a `Network` from the magic bytes.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use bitcoin::p2p::Magic;
    /// use bitcoin::Network;
    /// use std::convert::TryFrom;
    ///
    /// assert_eq!(Ok(Network::Bitcoin), Network::try_from(Magic::from_bytes([0xF9, 0xBE, 0xB4, 0xD9])));
    /// assert_eq!(None, Network::from_magic(Magic::from_bytes([0xFF, 0xFF, 0xFF, 0xFF])));
    /// ```
    pub fn from_magic(magic: MagicView) -> Option<NetworkView> {
        NetworkView::try_from(magic).ok()
    }

    /// Return the network magic bytes, which should be encoded little-endian
    /// at the start of every message
    ///
    /// # Examples
    ///
    /// ```rust
    /// use bitcoin::p2p::Magic;
    /// use bitcoin::Network;
    ///
    /// let network = Network::Bitcoin;
    /// assert_eq!(network.magic(), Magic::from_bytes([0xF9, 0xBE, 0xB4, 0xD9]));
    /// ```
    pub fn magic(self) -> MagicView {
        MagicView::from(self)
    }

    /// Converts a `Network` to its equivalent `bitcoind -chain` argument name.
    ///
    /// ```bash
    /// $ bitcoin-23.0/bin/bitcoind --help | grep -C 3 '\-chain=<chain>'
    /// Chain selection options:
    ///
    /// -chain=<chain>
    /// Use the chain <chain> (default: main). Allowed values: main, test, signet, regtest
    /// ```
    pub fn to_core_arg(self) -> &'static str {
        match self {
            NetworkView::Bitcoin => "main",
            NetworkView::Testnet => "test",
            NetworkView::Signet => "signet",
            NetworkView::Regtest => "regtest",
        }
    }

    /// Converts a `bitcoind -chain` argument name to its equivalent `Network`.
    ///
    /// ```bash
    /// $ bitcoin-23.0/bin/bitcoind --help | grep -C 3 '\-chain=<chain>'
    /// Chain selection options:
    ///
    /// -chain=<chain>
    /// Use the chain <chain> (default: main). Allowed values: main, test, signet, regtest
    /// ```
    pub fn from_core_arg(core_arg: &str) -> Result<Self, ParseNetworkErrorView> {
        use NetworkView::*;

        let network = match core_arg {
            "main" => Bitcoin,
            "test" => Testnet,
            "signet" => Signet,
            "regtest" => Regtest,
            _ => return Err(ParseNetworkErrorView(core_arg.to_owned())),
        };
        Ok(network)
    }

    /// Return the network's chain hash (genesis block hash).
    ///
    /// # Examples
    ///
    /// ```rust
    /// use bitcoin::Network;
    /// use bitcoin::blockdata::constants::ChainHash;
    ///
    /// let network = Network::Bitcoin;
    /// assert_eq!(network.chain_hash(), ChainHash::BITCOIN);
    /// ```
    pub fn chain_hash(self) -> ChainHashView {
        ChainHashView::using_genesis_block(self)
    }

    /// Creates a `Network` from the chain hash (genesis block hash).
    ///
    /// # Examples
    ///
    /// ```rust
    /// use bitcoin::Network;
    /// use bitcoin::blockdata::constants::ChainHash;
    /// use std::convert::TryFrom;
    ///
    /// assert_eq!(Ok(Network::Bitcoin), Network::try_from(ChainHash::BITCOIN));
    /// ```
    pub fn from_chain_hash(chain_hash: ChainHashView) -> Option<NetworkView> {
        NetworkView::try_from(chain_hash).ok()
    }
}
