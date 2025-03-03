// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use anyhow::{anyhow, Result};
use bech32::{Bech32m, Hrp};
use move_core_types::account_address::AccountAddress;

pub const MOVE_STD_ADDRESS_NAME: &str = "std";
pub const MOVE_STD_ADDRESS_LITERAL: &str = "0x1";
pub const MOVE_STD_ADDRESS: AccountAddress = AccountAddress::ONE;

pub const MOVEOS_STD_ADDRESS_NAME: &str = "moveos_std";
pub const MOVEOS_STD_ADDRESS_LITERAL: &str = "0x2";
pub const MOVEOS_STD_ADDRESS: AccountAddress = AccountAddress::TWO;

pub const ROOCH_HRP: Hrp = Hrp::parse_unchecked("rooch");

pub static MOVEOS_NAMED_ADDRESS_MAPPING: [(&str, &str); 2] = [
    (MOVE_STD_ADDRESS_NAME, MOVE_STD_ADDRESS_LITERAL),
    (MOVEOS_STD_ADDRESS_NAME, MOVEOS_STD_ADDRESS_LITERAL),
];

pub fn is_system_reserved_address(addr: AccountAddress) -> bool {
    let bytes = addr.into_bytes();
    bytes.iter().take(31).all(|u| u == &0u8) && bytes[31] > 0u8 && bytes[31] <= 10u8
}

pub fn is_vm_or_system_reserved_address(addr: AccountAddress) -> bool {
    //zero is vm address
    addr == AccountAddress::ZERO || is_system_reserved_address(addr)
}

pub fn to_bech32(addr: &AccountAddress) -> Result<String> {
    bech32::encode::<Bech32m>(ROOCH_HRP, addr.into_bytes().as_slice())
        .map_err(|e| anyhow!(format!("bech32 encode error: {}", e.to_string())))
}

pub fn from_bech32(bech32: &str) -> Result<AccountAddress> {
    let (hrp, data) = bech32::decode(bech32)?;
    anyhow::ensure!(hrp == ROOCH_HRP, "invalid account address hrp");
    anyhow::ensure!(
        data.len() == AccountAddress::LENGTH,
        "invalid account address length"
    );
    AccountAddress::from_bytes(data.as_slice()).map_err(|e| {
        anyhow!(format!(
            "account address from bytes error: {}",
            e.to_string()
        ))
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_system_reserved_address() {
        assert!(!is_system_reserved_address(AccountAddress::ZERO));
        assert!(is_system_reserved_address(AccountAddress::ONE));
        assert!(!is_system_reserved_address(new_address(11)));
        assert!(!is_system_reserved_address(AccountAddress::random()));
    }

    fn new_address(u: u8) -> AccountAddress {
        let mut addr = [0u8; AccountAddress::LENGTH];
        addr[AccountAddress::LENGTH - 1] = u;
        AccountAddress::new(addr)
    }
}
