// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::jsonrpc_types::{H256View, StrView};
use ethers::types::H160;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::vec::Vec;

pub type AccessList = Vec<AccessListItem>;

#[derive(Debug, Clone, Default, Eq, PartialEq, Hash, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct AccessListItem {
    address: StrView<H160>,
    storage_keys: Vec<H256View>,
}

impl AccessListItem {
    pub fn new(address: StrView<H160>, storage_keys: Vec<H256View>) -> Self {
        Self {
            address,
            storage_keys,
        }
    }
}
