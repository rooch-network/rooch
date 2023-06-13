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
