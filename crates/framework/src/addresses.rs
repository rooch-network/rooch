// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use move_core_types::account_address::AccountAddress;
use once_cell::sync::Lazy;

/// Define the Rooch address with `0x1`
const MOS_ADDRESS: &str = "0x1";

pub const fn mos_address() -> &'static str {
    MOS_ADDRESS
}

pub static MOVE_STD_ADDRESS: Lazy<AccountAddress> =
    Lazy::new(|| AccountAddress::from_hex_literal(MOS_ADDRESS).unwrap());
pub static MOS_STD_ADDRESS: Lazy<AccountAddress> = Lazy::new(|| *MOVE_STD_ADDRESS);
pub static MOS_FRAMEWORK_ADDRESS: Lazy<AccountAddress> = Lazy::new(|| *MOVE_STD_ADDRESS);
pub const MOS_STD_ADDRESS_NAME: &str = "mos_std";
pub const MOS_FRAMEWORK_ADDRESS_NAME: &str = "mos_framework";
