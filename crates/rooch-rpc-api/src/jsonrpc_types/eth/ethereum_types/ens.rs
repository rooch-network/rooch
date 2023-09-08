// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use ethers::utils::rlp::{self, Decodable, Encodable, RlpStream};
use schemars::JsonSchema;
use serde::{ser::Error as SerializationError, Deserialize, Deserializer, Serialize, Serializer};
use std::{cmp::Ordering, str::FromStr};

use crate::jsonrpc_types::H160View;

/// ENS name or Ethereum Address. Not RLP encoded/serialized if it's a name.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub enum NameOrAddress {
    /// An ENS Name (format does not get checked)
    Name(String),
    /// An Ethereum Address
    Address(H160View),
}
