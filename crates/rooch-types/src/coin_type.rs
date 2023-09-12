// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use clap::ArgEnum;
#[cfg(any(test, feature = "fuzzing"))]
use proptest_derive::Arbitrary;
use serde::{Deserialize, Serialize};
use strum_macros::{Display, EnumString};

/// The symbol standard is defined in [slip-0044](https://github.com/satoshilabs/slips/blob/master/slip-0044.md)
/// The rooch's symbol is not added to the slip-0044 yet.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[cfg_attr(any(test, feature = "fuzzing"), derive(Arbitrary))]
#[repr(u64)]
pub enum Symbol {
    BTC = 0,
    ETH = 60,
    ROH = 20230101,
}

impl TryFrom<u64> for Symbol {
    type Error = anyhow::Error;

    fn try_from(value: u64) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Symbol::BTC),
            60 => Ok(Symbol::ETH),
            20230101 => Ok(Symbol::ROH),
            _ => Err(anyhow::anyhow!("coin id {} is invalid", value)),
        }
    }
}

impl TryFrom<&str> for Symbol {
    type Error = anyhow::Error;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        let value = value.to_uppercase();
        match value.as_str() {
            "BTC" => Ok(Symbol::BTC),
            "ETH" => Ok(Symbol::ETH),
            "ROH" => Ok(Symbol::ROH),
            _ => Err(anyhow::anyhow!("coin id {} is invalid", value)),
        }
    }
}

impl std::fmt::Display for Symbol {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Symbol::BTC => write!(f, "BTC"),
            Symbol::ETH => write!(f, "ETH"),
            Symbol::ROH => write!(f, "ROH"),
        }
    }
}

/// The coin standard is defined in [slip-0044](https://github.com/satoshilabs/slips/blob/master/slip-0044.md)
/// The rooch's coin is not added to the slip-0044 yet.
#[derive(
    Debug,
    Copy,
    Clone,
    PartialEq,
    Eq,
    Hash,
    Serialize,
    Deserialize,
    Display,
    EnumString,
    ArgEnum,
    Ord,
    PartialOrd,
)]
#[cfg_attr(any(test, feature = "fuzzing"), derive(Arbitrary))]
#[repr(u64)]
#[strum(serialize_all = "lowercase")]
pub enum CoinID {
    Rooch = 0,
    Bitcoin = 1,
    Ether = 2,
    Nostr = 3,
}

impl TryFrom<u64> for CoinID {
    type Error = anyhow::Error;

    fn try_from(value: u64) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(CoinID::Rooch),
            1 => Ok(CoinID::Bitcoin),
            2 => Ok(CoinID::Ether),
            3 => Ok(CoinID::Nostr),
            _ => Err(anyhow::anyhow!("coin id {} is invalid", value)),
        }
    }
}
