// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use move_core_types::account_address::AccountAddress;
use once_cell::sync::Lazy;

/// Define the Rooch address with `0x1`
const MOVEOS_ADDRESS: &str = "0x1";

pub const fn moveos_address() -> &'static str {
    MOVEOS_ADDRESS
}

pub static MOVE_STD_ADDRESS: Lazy<AccountAddress> =
    Lazy::new(|| AccountAddress::from_hex_literal(MOVEOS_ADDRESS).unwrap());
pub static MOVEOS_STD_ADDRESS: Lazy<AccountAddress> = Lazy::new(|| *MOVE_STD_ADDRESS);
pub static ROOCH_FRAMEWORK_ADDRESS: Lazy<AccountAddress> = Lazy::new(|| *MOVE_STD_ADDRESS);
pub const MOVEOS_STD_ADDRESS_NAME: &str = "moveos_std";
pub const ROOCH_FRAMEWORK_ADDRESS_NAME: &str = "rooch_framework";

pub static MOVEOS_NAMED_ADDRESS_MAPPING: [(&str, &str); 2] = [
    (MOVEOS_STD_ADDRESS_NAME, MOVEOS_ADDRESS),
    (ROOCH_FRAMEWORK_ADDRESS_NAME, MOVEOS_ADDRESS),
];
