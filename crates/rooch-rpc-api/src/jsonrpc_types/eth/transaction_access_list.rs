// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::vec::Vec;

use crate::jsonrpc_types::{H176View, H256View};

pub type AccessList = Vec<AccessListItem>;

#[derive(Debug, Clone, Default, Eq, PartialEq, Hash, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct AccessListItem {
    address: H176View,
    storage_keys: Vec<H256View>,
}

impl AccessListItem {
    pub fn new(address: H176View, storage_keys: Vec<H256View>) -> Self {
        Self {
            address,
            storage_keys,
        }
    }
}
