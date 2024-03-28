// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::jsonrpc_types::StrView;

use super::{
    amount::AmountView,
    constants::{ChainHashView, UnknownChainHashErrorView},
    p2p::{MagicView, UnknownMagicErrorView},
};

/// An error in parsing network string.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize, JsonSchema)]
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

impl TryFrom<MagicView> for NetworkView {
    type Error = UnknownMagicErrorView;

    fn try_from(magic: MagicView) -> Result<Self, Self::Error> {
        match magic {
            // Note: any new network entries must be matched against here.
            MagicView::BITCOIN => Ok(NetworkView::Bitcoin),
            MagicView::TESTNET => Ok(NetworkView::Testnet),
            MagicView::SIGNET => Ok(NetworkView::Signet),
            MagicView::REGTEST => Ok(NetworkView::Regtest),
            _ => Err(UnknownMagicErrorView(magic)),
        }
    }
}

impl TryFrom<ChainHashView> for NetworkView {
    type Error = UnknownChainHashErrorView;

    fn try_from(chain_hash: ChainHashView) -> Result<Self, Self::Error> {
        match chain_hash {
            // Note: any new network entries must be matched against here.
            ChainHashView::BITCOIN => Ok(NetworkView::Bitcoin),
            ChainHashView::TESTNET => Ok(NetworkView::Testnet),
            ChainHashView::SIGNET => Ok(NetworkView::Signet),
            ChainHashView::REGTEST => Ok(NetworkView::Regtest),
            _ => Err(UnknownChainHashErrorView(chain_hash)),
        }
    }
}

#[derive(Clone, PartialEq, Eq, Debug, Deserialize, Serialize, JsonSchema)]
pub struct GetNetworkInfoResultNetworkView {
    pub name: String,
    pub limited: bool,
    pub reachable: bool,
    pub proxy: String,
    pub proxy_randomize_credentials: bool,
}

#[derive(Clone, PartialEq, Eq, Debug, Deserialize, Serialize, JsonSchema)]
pub struct GetNetworkInfoResultAddressView {
    pub address: String,
    pub port: StrView<usize>,
    pub score: StrView<usize>,
}

#[derive(Clone, PartialEq, Eq, Debug, Deserialize, Serialize, JsonSchema)]
pub struct GetNetworkInfoResultView {
    pub version: StrView<usize>,
    pub subversion: String,
    #[serde(rename = "protocolversion")]
    pub protocol_version: StrView<usize>,
    #[serde(rename = "localservices")]
    pub local_services: String,
    #[serde(rename = "localrelay")]
    pub local_relay: bool,
    #[serde(rename = "timeoffset")]
    pub time_offset: isize,
    pub connections: StrView<usize>,
    /// The number of inbound connections
    /// Added in Bitcoin Core v0.21
    pub connections_in: Option<StrView<usize>>,
    /// The number of outbound connections
    /// Added in Bitcoin Core v0.21
    pub connections_out: Option<StrView<usize>>,
    #[serde(rename = "networkactive")]
    pub network_active: bool,
    pub networks: Vec<GetNetworkInfoResultNetworkView>,
    #[serde(rename = "relayfee")]
    pub relay_fee: AmountView,
    #[serde(rename = "incrementalfee")]
    pub incremental_fee: AmountView,
    #[serde(rename = "localaddresses")]
    pub local_addresses: Vec<GetNetworkInfoResultAddressView>,
    pub warnings: String,
}

mod sealed {
    pub trait NetworkValidationView {}
    impl NetworkValidationView for super::NetworkCheckedView {}
    impl NetworkValidationView for super::NetworkUncheckedView {}
}

/// Marker of status of address's network validation. See section [*Parsing addresses*](Address#parsing-addresses)
/// on [`Address`] for details.
pub trait NetworkValidationView:
    sealed::NetworkValidationView + Sync + Send + Sized + Unpin
{
    /// Indicates whether this `NetworkValidation` is `NetworkChecked` or not.
    const IS_CHECKED: bool;
}

/// Marker that address's network has been successfully validated. See section [*Parsing addresses*](Address#parsing-addresses)
/// on [`Address`] for details.
#[derive(
    Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Deserialize, Serialize, JsonSchema,
)]
pub enum NetworkCheckedView {}

/// Marker that address's network has not yet been validated. See section [*Parsing addresses*](Address#parsing-addresses)
/// on [`Address`] for details.
#[derive(
    Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Deserialize, Serialize, JsonSchema,
)]
pub enum NetworkUncheckedView {}

impl NetworkValidationView for NetworkCheckedView {
    const IS_CHECKED: bool = true;
}
impl NetworkValidationView for NetworkUncheckedView {
    const IS_CHECKED: bool = false;
}
