// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use ethers::types::{H160, U256, U64};
use crate::jsonrpc_types::bytes::Bytes;
use crate::jsonrpc_types::eth::AccessList;
use serde::{Deserialize};

/// Call request
#[derive(Debug, Default, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
#[serde(rename_all = "camelCase")]
pub struct CallRequest {
    /// transaction type. Defaults to legacy type.
    #[serde(rename = "type", skip_serializing_if = "Option::is_none")]
    pub transaction_type: Option<U64>,
    /// From
    pub from: Option<H160>,
    /// To
    pub to: Option<H160>,
    /// Gas Price
    #[serde(skip_serializing_if = "Option::is_none")]
    pub gas_price: Option<U256>,
    /// Max fee per gas
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_fee_per_gas: Option<U256>,
    /// Gas
    pub gas: Option<U256>,
    /// Value
    pub value: Option<U256>,
    /// Data
    pub data: Option<Bytes>,
    /// Nonce
    pub nonce: Option<U256>,
    /// Access list
    #[serde(skip_serializing_if = "Option::is_none")]
    pub access_list: Option<AccessList>,
    /// Miner bribe
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_priority_fee_per_gas: Option<U256>,
}