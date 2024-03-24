// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use std::{fmt, str::FromStr};

use bitcoin::hex::{write_err, FromHex, HexToArrayError};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use super::network::NetworkView;

/// Network magic bytes to identify the cryptocurrency network the message was intended for.
#[derive(
    Debug, Copy, PartialEq, Eq, PartialOrd, Ord, Clone, Hash, Deserialize, Serialize, JsonSchema,
)]
pub struct MagicView([u8; 4]);

impl MagicView {
    /// Bitcoin mainnet network magic bytes.
    pub const BITCOIN: Self = Self([0xF9, 0xBE, 0xB4, 0xD9]);
    /// Bitcoin testnet network magic bytes.
    pub const TESTNET: Self = Self([0x0B, 0x11, 0x09, 0x07]);
    /// Bitcoin signet network magic bytes.
    pub const SIGNET: Self = Self([0x0A, 0x03, 0xCF, 0x40]);
    /// Bitcoin regtest network magic bytes.
    pub const REGTEST: Self = Self([0xFA, 0xBF, 0xB5, 0xDA]);

    /// Create network magic from bytes.
    pub fn from_bytes(bytes: [u8; 4]) -> MagicView {
        MagicView(bytes)
    }

    /// Get network magic bytes.
    pub fn to_bytes(self) -> [u8; 4] {
        self.0
    }
}

impl FromStr for MagicView {
    type Err = ParseMagicErrorView;

    fn from_str(s: &str) -> Result<MagicView, Self::Err> {
        match <[u8; 4]>::from_hex(s) {
            Ok(magic) => Ok(MagicView::from_bytes(magic)),
            Err(e) => Err(ParseMagicErrorView {
                error: e,
                magic: s.to_owned(),
            }),
        }
    }
}

impl From<NetworkView> for MagicView {
    fn from(network: NetworkView) -> MagicView {
        match network {
            // Note: new network entries must explicitly be matched in `try_from` below.
            NetworkView::Bitcoin => MagicView::BITCOIN,
            NetworkView::Testnet => MagicView::TESTNET,
            NetworkView::Signet => MagicView::SIGNET,
            NetworkView::Regtest => MagicView::REGTEST,
        }
    }
}

/// An error in parsing magic bytes.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub struct ParseMagicErrorView {
    /// The error that occurred when parsing the string.
    error: HexToArrayError,
    /// The byte string that failed to parse.
    magic: String,
}

impl fmt::Display for ParseMagicErrorView {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        write_err!(f, "failed to parse {} as network magic", self.magic; self.error)
    }
}

impl std::error::Error for ParseMagicErrorView {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        Some(&self.error)
    }
}

/// Error in creating a Network from Magic bytes.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize, JsonSchema)]
#[non_exhaustive]
pub struct UnknownMagicErrorView(pub MagicView);

impl fmt::Display for UnknownMagicErrorView {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        write!(f, "unknown network magic {:?}", self.0)
    }
}
