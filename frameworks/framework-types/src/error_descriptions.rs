// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::addresses::*;
use move_core_types::account_address::AccountAddress;
use once_cell::sync::Lazy;
use std::collections::BTreeMap;

const MOVE_STD_ERROR_DESCRIPTIONS: &[u8] =
    include_bytes!("../../move-stdlib/error_description.errmap");

pub fn move_std_error_descriptions() -> &'static [u8] {
    MOVE_STD_ERROR_DESCRIPTIONS
}

const MOVEOS_STD_ERROR_DESCRIPTIONS: &[u8] =
    include_bytes!("../../moveos-stdlib/error_description.errmap");

pub fn moveos_std_error_descriptions() -> &'static [u8] {
    MOVEOS_STD_ERROR_DESCRIPTIONS
}

const ROOCH_FRAMEWORK_ERROR_DESCRIPTIONS: &[u8] =
    include_bytes!("../../rooch-framework/error_description.errmap");

pub fn rooch_framework_error_descriptions() -> &'static [u8] {
    ROOCH_FRAMEWORK_ERROR_DESCRIPTIONS
}

const BITCOIN_MOVE_ERROR_DESCRIPTIONS: &[u8] =
    include_bytes!("../../bitcoin-move/error_description.errmap");

pub fn bitcoin_move_error_descriptions() -> &'static [u8] {
    BITCOIN_MOVE_ERROR_DESCRIPTIONS
}

const ROOCH_NURSERY_ERROR_DESCRIPTIONS: &[u8] =
    include_bytes!("../../rooch-nursery/error_description.errmap");

pub fn rooch_nursery_error_descriptions() -> &'static [u8] {
    ROOCH_NURSERY_ERROR_DESCRIPTIONS
}

pub static ERROR_DESCRIPTIONS: Lazy<BTreeMap<AccountAddress, &'static [u8]>> = Lazy::new(|| {
    let mut error_descriptions = BTreeMap::new();
    error_descriptions.insert(MOVE_STD_ADDRESS, move_std_error_descriptions());
    error_descriptions.insert(MOVEOS_STD_ADDRESS, moveos_std_error_descriptions());
    error_descriptions.insert(
        ROOCH_FRAMEWORK_ADDRESS,
        rooch_framework_error_descriptions(),
    );
    error_descriptions.insert(BITCOIN_MOVE_ADDRESS, bitcoin_move_error_descriptions());
    error_descriptions.insert(ROOCH_NURSERY_ADDRESS, rooch_nursery_error_descriptions());
    error_descriptions
});
