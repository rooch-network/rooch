// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::jsonrpc_types::eth::AccessList;
use crate::jsonrpc_types::BytesView;
use crate::jsonrpc_types::H160View;
use crate::jsonrpc_types::StrView;
use ethers::types::U256;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// Call request
#[derive(Debug, Default, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
#[serde(rename_all = "camelCase")]
pub struct CallRequest {
    /// transaction type. Defaults to legacy type.
    #[serde(rename = "type", skip_serializing_if = "Option::is_none")]
    pub transaction_type: Option<StrView<u64>>,
    /// From
    pub from: Option<H160View>,
    /// To
    pub to: Option<H160View>,
    /// Gas Price
    #[serde(skip_serializing_if = "Option::is_none")]
    pub gas_price: Option<StrView<U256>>,
    /// Max fee per gas
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_fee_per_gas: Option<StrView<U256>>,
    /// Gas
    pub gas: Option<StrView<U256>>,
    /// Value
    pub value: Option<StrView<U256>>,
    /// Data
    pub data: Option<BytesView>,
    /// Nonce
    pub nonce: Option<StrView<U256>>,
    /// Access list
    #[serde(skip_serializing_if = "Option::is_none")]
    pub access_list: Option<AccessList>,
    /// Miner bribe
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_priority_fee_per_gas: Option<StrView<U256>>,
}
