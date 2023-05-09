// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use std::str::FromStr;

use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};

pub const ROOCH_COIN_NAME: &str = "ROH";
pub const ROOCH_COIN_ID: u32 = 20130101;

pub static ROOCH_COIN: Lazy<Coin> = Lazy::new(|| Coin {
    id: ROOCH_COIN_ID,
    symbol: ROOCH_COIN_NAME.to_string(),
});

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Coin {
    pub id: u32,
    pub symbol: String,
}

impl TryFrom<u32> for Coin {
    type Error = anyhow::Error;

    fn try_from(value: u32) -> Result<Self, Self::Error> {
        if value == ROOCH_COIN_ID {
            Ok(ROOCH_COIN.clone())
        } else {
            let coin = slip44::Coin::try_from(value)
                .map_err(|e| anyhow::anyhow!("coin id {} is invalid, {}", value, e))?;
            let symbol = slip44::Symbol::try_from(coin)
                .map_err(|e| anyhow::anyhow!("coin id {} is invalid, {}", value, e))?;
            Ok(Self {
                id: coin.id(),
                symbol: symbol.to_string(),
            })
        }
    }
}

impl TryFrom<&str> for Coin {
    type Error = anyhow::Error;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        let value = value.to_uppercase();
        if value == ROOCH_COIN_NAME {
            Ok(ROOCH_COIN.clone())
        } else {
            let symbol = slip44::Symbol::from_str(&value)
                .map_err(|e| anyhow::anyhow!("coin id {} is invalid, {}", value, e))?;
            let coin = slip44::Coin::from(symbol);
            Ok(Self {
                id: coin.id(),
                symbol: symbol.to_string(),
            })
        }
    }
}
