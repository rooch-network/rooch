// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use std::fmt::Display;

#[cfg(any(test, feature = "fuzzing"))]
use proptest_derive::Arbitrary;
use serde::{Deserialize, Serialize};

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[cfg_attr(any(test, feature = "fuzzing"), derive(Arbitrary))]
#[repr(u64)]
pub enum ChainID {
    Dev = 20230103,
    Test = 20230102,
    Main = 20230101,
}

impl Display for ChainID {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ChainID::Dev => write!(f, "Dev"),
            ChainID::Test => write!(f, "Test"),
            ChainID::Main => write!(f, "Main"),
        }
    }
}

impl From<ChainID> for u64 {
    fn from(chain_id: ChainID) -> Self {
        chain_id as u64
    }
}

impl TryFrom<u64> for ChainID {
    type Error = anyhow::Error;

    fn try_from(value: u64) -> Result<Self, Self::Error> {
        match value {
            20230103 => Ok(ChainID::Dev),
            20230102 => Ok(ChainID::Test),
            20230101 => Ok(ChainID::Main),
            _ => Err(anyhow::anyhow!("chain id {} is invalid", value)),
        }
    }
}
