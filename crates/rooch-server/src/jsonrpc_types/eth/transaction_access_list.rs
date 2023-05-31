// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use ethers::types::{H160, H256};
use serde::{Deserialize, Serialize};
use std::vec::Vec;

pub type AccessList = Vec<AccessListItem>;

#[derive(Debug, Clone, Default, Eq, PartialEq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AccessListItem {
    address: H160,
    storage_keys: Vec<H256>,
}

impl AccessListItem {
    pub fn new(address: H160, storage_keys: Vec<H256>) -> Self {
        Self {
            address,
            storage_keys,
        }
    }
}
