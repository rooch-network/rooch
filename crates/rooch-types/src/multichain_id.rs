// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use anyhow::{bail, format_err, Result};
#[cfg(any(test, feature = "fuzzing"))]
use proptest_derive::Arbitrary;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::fmt;
use std::fmt::{Display, Formatter};
use std::str::FromStr;

use crate::chain_id::ChainID;

pub const BITCOIN: u64 = 0;
pub const ETHER: u64 = 60;
pub const SUI: u64 = 784;
pub const NOSTR: u64 = 1237;
pub const ROOCH: u64 = 20230101; // place holder for slip-0044 needs to replace later

#[derive(
    Clone, Copy, Debug, Deserialize, Serialize, Hash, Eq, PartialEq, PartialOrd, Ord, JsonSchema,
)]
#[cfg_attr(any(test, feature = "fuzzing"), derive(Arbitrary))]
pub struct MultiChainID {
    id: u64,
}

impl MultiChainID {
    pub fn new(id: u64) -> Self {
        Self { id }
    }

    pub fn id(self) -> u64 {
        self.id
    }

    pub fn is_ethereum(self) -> bool {
        self.id == ETHER
    }

    pub fn is_sui(self) -> bool {
        self.id == SUI
    }

    pub fn is_bitcoin(self) -> bool {
        self.id == BITCOIN
    }

    pub fn is_nostr(self) -> bool {
        self.id == NOSTR
    }
}

impl Display for MultiChainID {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.id)
    }
}

impl FromStr for MultiChainID {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let id: u64 = s.parse()?;
        Ok(MultiChainID::new(id))
    }
}

impl From<u64> for MultiChainID {
    fn from(id: u64) -> Self {
        Self::new(id)
    }
}

impl From<ChainID> for MultiChainID {
    fn from(chain_id: ChainID) -> Self {
        Self::new(chain_id.id())
    }
}

#[allow(clippy::from_over_into)]
impl Into<u64> for MultiChainID {
    fn into(self) -> u64 {
        self.id
    }
}

// BuiltinMultiChainID is following coin standard: https://github.com/satoshilabs/slips/blob/master/slip-0044.md
#[derive(
    Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize, JsonSchema,
)]
#[repr(u64)]
#[cfg_attr(any(test, feature = "fuzzing"), derive(Arbitrary))]
pub enum RoochMultiChainID {
    Bitcoin = BITCOIN,
    Ether = ETHER,
    Sui = SUI,
    Nostr = NOSTR,
    Rooch = ROOCH,
}

impl Display for RoochMultiChainID {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            RoochMultiChainID::Bitcoin => write!(f, "bitcoin"),
            RoochMultiChainID::Ether => write!(f, "ether"),
            RoochMultiChainID::Sui => write!(f, "sui"),
            RoochMultiChainID::Nostr => write!(f, "nostr"),
            RoochMultiChainID::Rooch => write!(f, "rooch"),
        }
    }
}

impl From<RoochMultiChainID> for u64 {
    fn from(multichain_id: RoochMultiChainID) -> Self {
        multichain_id as u64
    }
}

impl TryFrom<u64> for RoochMultiChainID {
    type Error = anyhow::Error;

    fn try_from(value: u64) -> Result<Self, Self::Error> {
        match value {
            BITCOIN => Ok(RoochMultiChainID::Bitcoin),
            ETHER => Ok(RoochMultiChainID::Ether),
            SUI => Ok(RoochMultiChainID::Sui),
            NOSTR => Ok(RoochMultiChainID::Nostr),
            ROOCH => Ok(RoochMultiChainID::Rooch),
            _ => Err(anyhow::anyhow!("multichain id {} is invalid", value)),
        }
    }
}

impl FromStr for RoochMultiChainID {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "bitcoin" => Ok(RoochMultiChainID::Bitcoin),
            "ether" => Ok(RoochMultiChainID::Ether),
            "sui" => Ok(RoochMultiChainID::Sui),
            "nostr" => Ok(RoochMultiChainID::Nostr),
            "rooch" => Ok(RoochMultiChainID::Rooch),
            s => Err(format_err!("Unknown multichain: {}", s)),
        }
    }
}

impl TryFrom<MultiChainID> for RoochMultiChainID {
    type Error = anyhow::Error;
    fn try_from(multichain_id: MultiChainID) -> Result<Self, Self::Error> {
        Ok(match multichain_id.id() {
            BITCOIN => Self::Bitcoin,
            ETHER => Self::Ether,
            SUI => Self::Sui,
            NOSTR => Self::Nostr,
            ROOCH => Self::Rooch,
            id => bail!("{} is not a builtin multichain id", id),
        })
    }
}

impl RoochMultiChainID {
    pub fn multichain_name(self) -> String {
        self.to_string()
    }

    pub fn multichain_id(self) -> MultiChainID {
        MultiChainID::new(self.into())
    }

    pub fn is_bitcoin(self) -> bool {
        matches!(self, RoochMultiChainID::Bitcoin)
    }

    pub fn is_ethereum(self) -> bool {
        matches!(self, RoochMultiChainID::Ether)
    }

    pub fn is_sui(self) -> bool {
        matches!(self, RoochMultiChainID::Sui)
    }

    pub fn is_nostr(self) -> bool {
        matches!(self, RoochMultiChainID::Nostr)
    }

    pub fn is_rooch(self) -> bool {
        matches!(self, RoochMultiChainID::Rooch)
    }

    pub fn multichain_ids() -> Vec<RoochMultiChainID> {
        vec![
            RoochMultiChainID::Bitcoin,
            RoochMultiChainID::Ether,
            RoochMultiChainID::Sui,
            RoochMultiChainID::Nostr,
            RoochMultiChainID::Rooch,
        ]
    }
}
