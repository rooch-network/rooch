// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

#[cfg(any(test, feature = "fuzzing"))]
use proptest_derive::Arbitrary;
use serde::{Deserialize, Serialize};

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
