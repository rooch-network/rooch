// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::jsonrpc_types::eth::AccessList;
use crate::jsonrpc_types::{bytes::Bytes, H160View};
use crate::jsonrpc_types::{U256View, U64View};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// Call request
#[derive(Debug, Default, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
#[serde(rename_all = "camelCase")]
pub struct CallRequest {
    /// transaction type. Defaults to legacy type.
    #[serde(rename = "type", skip_serializing_if = "Option::is_none")]
    pub transaction_type: Option<U64View>,
    /// From
    pub from: Option<H160View>,
    /// To
    pub to: Option<H160View>,
    /// Gas Price
    #[serde(skip_serializing_if = "Option::is_none")]
    pub gas_price: Option<U256View>,
    /// Max fee per gas
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_fee_per_gas: Option<U256View>,
    /// Gas
    pub gas: Option<U256View>,
    /// Value
    pub value: Option<U256View>,
    /// Data
    pub data: Option<Bytes>,
    /// Nonce
    pub nonce: Option<U256View>,
    /// Access list
    #[serde(skip_serializing_if = "Option::is_none")]
    pub access_list: Option<AccessList>,
    /// Miner bribe
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_priority_fee_per_gas: Option<U256View>,
}
