// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use framework_types::addresses::ROOCH_FRAMEWORK_ADDRESS;
use move_core_types::account_address::AccountAddress;
use move_core_types::identifier::IdentStr;
use move_core_types::language_storage::TypeTag;
use move_core_types::value::{MoveStructLayout, MoveTypeLayout};
use moveos_types::state::{MoveStructState, MoveStructType};
use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;
use crate::multichain_id::RoochMultiChainID;
use super::MultiChainAddress;

pub const MODULE_NAME: &IdentStr = IdentStr::new("ton_address").unwrap();

/// The Ton address type
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct TonAddress {
    workchain: i32,
    hash_part: AccountAddress,
}

impl TonAddress {
    pub fn workchain(&self) -> i32 {
        self.workchain
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

impl MoveStructType for TonAddress {
    const ADDRESS: AccountAddress = ROOCH_FRAMEWORK_ADDRESS;
    const MODULE_NAME: &'static IdentStr = MODULE_NAME;
    const STRUCT_NAME: &'static IdentStr = IdentStr::new("TonAddress").unwrap();
}

impl MoveStructState for TonAddress {
    fn struct_layout() -> MoveStructLayout {
        MoveStructLayout::new(vec![
            MoveTypeLayout::U32,
            MoveTypeLayout::Address,
        ])
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
            workchain: address.workchain,
            hash_part: address.hash_part.into(),
        }
    }
}

impl From<TonAddress> for tonlib_core::TonAddress {
    fn from(address: TonAddress) -> Self {
        tonlib_core::TonAddress {
            workchain: address.workchain,
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ton_address() {
        let addr = TonAddress::from_str("-1:e4d954ef9f4e1250a26b5bbad76a1cdd17cfd08babad6f4c23e372270aef6f76").unwrap();
        //println!("addr: {}", addr);
        let bytes = addr.to_bytes().unwrap();
        println!("bytes: {:?}", hex::encode(bytes));
    }
}