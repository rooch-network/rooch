// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use move_core_types::account_address::AccountAddress;
use std::{collections::BTreeMap, str::FromStr};

pub const ROOCH_FRAMEWORK_ADDRESS_NAME: &str = "rooch_framework";
pub const ROOCH_FRAMEWORK_ADDRESS_LITERAL: &str = "0x3";
pub const ROOCH_FRAMEWORK_ADDRESS: AccountAddress = {
    let mut addr = [0u8; AccountAddress::LENGTH];
    addr[AccountAddress::LENGTH - 1] = 3u8;
    AccountAddress::new(addr)
};

pub const BITCOIN_MOVE_ADDRESS_NAME: &str = "bitcoin_move";
pub const BITCOIN_MOVE_ADDRESS_LITERAL: &str = "0x4";
pub const BITCOIN_MOVE_ADDRESS: AccountAddress = {
    let mut addr = [0u8; AccountAddress::LENGTH];
    addr[AccountAddress::LENGTH - 1] = 4u8;
    AccountAddress::new(addr)
};

pub const ROOCH_NURSERY_ADDRESS_NAME: &str = "rooch_nursery";
pub const ROOCH_NURSERY_ADDRESS_LITERAL: &str = "0xa";
pub const ROOCH_NURSERY_ADDRESS: AccountAddress = {
    let mut addr = [0u8; AccountAddress::LENGTH];
    addr[AccountAddress::LENGTH - 1] = 10u8;
    AccountAddress::new(addr)
};

pub static ROOCH_NAMED_ADDRESS_MAPPING: [(&str, &str); 3] = [
    (
        ROOCH_FRAMEWORK_ADDRESS_NAME,
        ROOCH_FRAMEWORK_ADDRESS_LITERAL,
    ),
    (BITCOIN_MOVE_ADDRESS_NAME, BITCOIN_MOVE_ADDRESS_LITERAL),
    (ROOCH_NURSERY_ADDRESS_NAME, ROOCH_NURSERY_ADDRESS_LITERAL),
];

pub fn rooch_framework_named_addresses() -> BTreeMap<String, AccountAddress> {
    let mut address_mapping = moveos_stdlib::moveos_stdlib_named_addresses();
    address_mapping.extend(
        ROOCH_NAMED_ADDRESS_MAPPING
            .iter()
            .map(|(name, addr)| (name.to_string(), AccountAddress::from_str(addr).unwrap())),
    );
    address_mapping
}
