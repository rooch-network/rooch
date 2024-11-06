// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use move_core_types::account_address::AccountAddress;
use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;

use crate::multichain_id::RoochMultiChainID;

use super::MultiChainAddress;

/// The Ton address type
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct TonAddress {
    is_nagative: bool,
    //The workchain in TonAddress is i32, but No i32 in Move
    //So we use u32 instead, and use `is_nagative` to represent the sign
    workchain: u32,
    hash_part: AccountAddress,
}

impl TonAddress {
    pub fn workchain(&self) -> i32 {
        if self.is_nagative {
            -(self.workchain as i32)
        } else {
            self.workchain as i32
        }
    }

    pub fn hash_part(&self) -> [u8; 32] {
        self.hash_part.into()
    }

    pub fn from_bytes(bytes: &[u8]) -> Result<Self, anyhow::Error> {
        bcs::from_bytes(bytes).map_err(|e| anyhow::anyhow!("TonAddress deserialize error: {}", e))
    }

    pub fn to_bytes(&self) -> Result<Vec<u8>, anyhow::Error> {
        bcs::to_bytes(self).map_err(|e| anyhow::anyhow!("TonAddress serialize error: {}", e))
    }
}

impl fmt::Display for TonAddress {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let ton_addr: tonlib_core::TonAddress = self.clone().into();
        write!(f, "{}", ton_addr)
    }
}

impl FromStr for TonAddress {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let ton_addr = tonlib_core::TonAddress::from_str(s)?;
        Ok(ton_addr.into())
    }
}

impl From<tonlib_core::TonAddress> for TonAddress {
    fn from(address: tonlib_core::TonAddress) -> Self {
        Self {
            is_nagative: address.workchain < 0,
            workchain: address.workchain.unsigned_abs(),
            hash_part: address.hash_part.into(),
        }
    }
}

impl From<TonAddress> for tonlib_core::TonAddress {
    fn from(address: TonAddress) -> Self {
        tonlib_core::TonAddress {
            workchain: if address.is_nagative {
                -(address.workchain as i32)
            } else {
                address.workchain as i32
            },
            hash_part: address.hash_part.into(),
        }
    }
}

impl TryFrom<MultiChainAddress> for TonAddress {
    type Error = anyhow::Error;

    fn try_from(value: MultiChainAddress) -> Result<Self, Self::Error> {
        if value.multichain_id != RoochMultiChainID::Ton {
            return Err(anyhow::anyhow!(
                "multichain_id type {} is invalid",
                value.multichain_id
            ));
        }
        TonAddress::from_bytes(&value.raw_address)
    }
}

impl From<TonAddress> for MultiChainAddress {
    fn from(address: TonAddress) -> Self {
        MultiChainAddress {
            multichain_id: RoochMultiChainID::Ton,
            raw_address: address
                .to_bytes()
                .expect("TonAddress to_bytes should not fail"),
        }
    }
}
