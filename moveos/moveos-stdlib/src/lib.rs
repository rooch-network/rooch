// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use move_command_line_common::address::NumericalAddress;
use moveos_types::addresses::MOVEOS_NAMED_ADDRESS_MAPPING;
use std::collections::BTreeMap;

pub mod natives;

const MOVE_STD_ERROR_DESCRIPTIONS: &[u8] =
    include_bytes!("../move-stdlib/error_description.errmap");

pub fn move_std_error_descriptions() -> &'static [u8] {
    MOVE_STD_ERROR_DESCRIPTIONS
}

const MOVEOS_STD_ERROR_DESCRIPTIONS: &[u8] =
    include_bytes!("../moveos-stdlib/error_description.errmap");

pub fn moveos_std_error_descriptions() -> &'static [u8] {
    MOVEOS_STD_ERROR_DESCRIPTIONS
}

pub fn moveos_stdlib_named_addresses() -> BTreeMap<String, NumericalAddress> {
    let mut address_mapping = move_stdlib::move_stdlib_named_addresses();
    address_mapping.extend(
        MOVEOS_NAMED_ADDRESS_MAPPING
            .iter()
            .map(|(name, addr)| (name.to_string(), NumericalAddress::parse_str(addr).unwrap())),
    );
    address_mapping
}
