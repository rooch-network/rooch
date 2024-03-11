// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use move_core_types::account_address::AccountAddress;

pub const MOVE_STD_ADDRESS_NAME: &str = "std";
pub const MOVE_STD_ADDRESS_LITERAL: &str = "0x1";
pub const MOVE_STD_ADDRESS: AccountAddress = AccountAddress::ONE;

pub const MOVEOS_STD_ADDRESS_NAME: &str = "moveos_std";
pub const MOVEOS_STD_ADDRESS_LITERAL: &str = "0x2";
pub const MOVEOS_STD_ADDRESS: AccountAddress = AccountAddress::TWO;

pub static MOVEOS_NAMED_ADDRESS_MAPPING: [(&str, &str); 2] = [
    (MOVE_STD_ADDRESS_NAME, MOVE_STD_ADDRESS_LITERAL),
    (MOVEOS_STD_ADDRESS_NAME, MOVEOS_STD_ADDRESS_LITERAL),
];

pub fn is_system_reserved_address(addr: AccountAddress) -> bool {
    let bytes = addr.into_bytes();
    bytes.iter().take(31).all(|u| u == &0u8) && bytes[31] > 0u8 && bytes[31] <= 10u8
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_system_reserved_address() {
        assert_eq!(is_system_reserved_address(AccountAddress::ZERO), false);
        assert_eq!(is_system_reserved_address(AccountAddress::ONE), true);
        assert_eq!(is_system_reserved_address(new_address(11)), false);
        assert_eq!(is_system_reserved_address(AccountAddress::random()), false);
    }

    fn new_address(u: u8) -> AccountAddress {
        let mut addr = [0u8; AccountAddress::LENGTH];
        addr[AccountAddress::LENGTH - 1] = u;
        AccountAddress::new(addr)
    }
}
