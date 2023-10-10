// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::jsonrpc_types::StrView;
use ethers::types::H160;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// ENS name or Ethereum Address. Not RLP encoded/serialized if it's a name.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub enum NameOrAddress {
    /// An ENS Name (format does not get checked)
    Name(String),
    /// An Ethereum Address
    Address(StrView<H160>),
}
