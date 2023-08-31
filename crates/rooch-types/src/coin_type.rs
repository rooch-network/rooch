// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use clap::ArgEnum;
#[cfg(any(test, feature = "fuzzing"))]
use proptest_derive::Arbitrary;
use serde::{Deserialize, Serialize};
use std::fmt::Display;

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

impl Display for Symbol {
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
    Debug, Copy, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, ArgEnum, Ord, PartialOrd,
)]
#[cfg_attr(any(test, feature = "fuzzing"), derive(Arbitrary))]
#[repr(u64)]
pub enum Coin {
    Bitcoin = 0,
    Ether = 60,
    Nostr = 1237,
    Rooch = 20230101,
}

impl TryFrom<u64> for Coin {
    type Error = anyhow::Error;

    fn try_from(value: u64) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Coin::Bitcoin),
            60 => Ok(Coin::Ether),
            1237 => Ok(Coin::Nostr),
            20230101 => Ok(Coin::Rooch),
            _ => Err(anyhow::anyhow!("coin id {} is invalid", value)),
        }
    }
}

impl TryFrom<&str> for Coin {
    type Error = anyhow::Error;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        let value = value.to_lowercase();
        match value.as_str() {
            "bitcoin" => Ok(Coin::Bitcoin),
            "ether" => Ok(Coin::Ether),
            "nostr" => Ok(Coin::Nostr),
            "rooch" => Ok(Coin::Rooch),
            _ => Err(anyhow::anyhow!("coin id {} is invalid", value)),
        }
    }
}

impl Display for Coin {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Coin::Bitcoin => write!(f, "Bitcoin"),
            Coin::Ether => write!(f, "Ether"),
            Coin::Nostr => write!(f, "Nostr"),
            Coin::Rooch => write!(f, "Rooch"),
        }
    }
}
