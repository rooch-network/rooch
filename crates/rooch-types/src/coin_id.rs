// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

#[cfg(any(test, feature = "fuzzing"))]
use proptest_derive::Arbitrary;
use serde::{Deserialize, Serialize};
use std::fmt::Display;

/// The coin id standard is defined in [slip-0044](https://github.com/satoshilabs/slips/blob/master/slip-0044.md)
/// The rooch's ID do not add to the slip-0044 yet.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[cfg_attr(any(test, feature = "fuzzing"), derive(Arbitrary))]
#[repr(u32)]
pub enum CoinID {
    BTC = 0,
    ETH = 60,
    ROH = 20230101,
}

impl TryFrom<u32> for CoinID {
    type Error = anyhow::Error;

    fn try_from(value: u32) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(CoinID::BTC),
            60 => Ok(CoinID::ETH),
            20130101 => Ok(CoinID::ROH),
            _ => Err(anyhow::anyhow!("coin id {} is invalid", value)),
        }
    }
}

impl TryFrom<&str> for CoinID {
    type Error = anyhow::Error;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        let value = value.to_uppercase();
        match value.as_str() {
            "BTC" => Ok(CoinID::BTC),
            "ETH" => Ok(CoinID::ETH),
            "ROH" => Ok(CoinID::ROH),
            _ => Err(anyhow::anyhow!("coin id {} is invalid", value)),
        }
    }
}

impl Display for CoinID {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CoinID::BTC => write!(f, "BTC"),
            CoinID::ETH => write!(f, "ETH"),
            CoinID::ROH => write!(f, "ROH"),
        }
    }
}
