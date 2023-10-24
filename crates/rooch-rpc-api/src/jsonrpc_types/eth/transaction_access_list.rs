// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::jsonrpc_types::{H160View, H256View};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::vec::Vec;

pub type AccessList = Vec<AccessListItem>;

#[derive(Debug, Clone, Default, Eq, PartialEq, Hash, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct AccessListItem {
    address: H160View,
    storage_keys: Vec<H256View>,
}

impl AccessListItem {
    pub fn new(address: H160View, storage_keys: Vec<H256View>) -> Self {
        Self {
            address,
            storage_keys,
        }
    }
}
