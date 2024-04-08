// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use move_core_types::account_address::AccountAddress;
use moveos_types::addresses::MOVEOS_NAMED_ADDRESS_MAPPING;
use std::{collections::BTreeMap, str::FromStr};

pub mod natives;

pub fn moveos_stdlib_named_addresses() -> BTreeMap<String, AccountAddress> {
    let mut address_mapping = BTreeMap::new();
    address_mapping.extend(
        move_stdlib::move_stdlib_named_addresses()
            .into_iter()
            .map(|(k, v)| (k, v.into_inner())),
    );

    address_mapping.extend(
        MOVEOS_NAMED_ADDRESS_MAPPING
            .iter()
            .map(|(name, addr)| (name.to_string(), AccountAddress::from_str(addr).unwrap())),
    );
    address_mapping
}
