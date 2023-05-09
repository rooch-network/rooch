// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use anyhow::{anyhow, format_err, Result};
use move_core_types::{
    account_address::AccountAddress, identifier::Identifier, language_storage::ModuleId,
};
use serde::{Deserialize, Serialize, Serializer};
use std::fmt;
use std::str::FromStr;

/// Identifier of a module function
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StructId {
    pub module_id: ModuleId,
    pub struct_id: Identifier,
}

fn parse_function_id(function_id: &str) -> Result<StructId> {
    let ids: Vec<&str> = function_id.split_terminator("::").collect();
    if ids.len() != 3 {
        return Err(anyhow!(
            "StructId is not well formed.  Must be of the form <address>::<module>::<function>"
        ));
    }
    let address = AccountAddress::from_str(ids.first().unwrap())
        .map_err(|err| anyhow!("Module address error: {:?}", err.to_string()))?;
    let module = Identifier::from_str(ids.get(1).unwrap())
        .map_err(|err| anyhow!("Module name error: {:?}", err.to_string()))?;
    let struct_id = Identifier::from_str(ids.get(2).unwrap())
        .map_err(|err| anyhow!("Function name error: {:?}", err.to_string()))?;
    let module_id = ModuleId::new(address, module);
    Ok(StructId {
        module_id,
        struct_id,
    })
}

impl FromStr for StructId {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        parse_function_id(s)
    }
}

/// Hex encoded bytes to allow for having bytes represented in JSON
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct HexEncodedBytes(pub Vec<u8>);

impl HexEncodedBytes {
    pub fn json(&self) -> anyhow::Result<serde_json::Value> {
        Ok(serde_json::to_value(self)?)
    }
}

impl FromStr for HexEncodedBytes {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> anyhow::Result<Self, anyhow::Error> {
        let hex_str = if let Some(hex) = s.strip_prefix("0x") {
            hex
        } else {
            s
        };
        Ok(Self(hex::decode(hex_str).map_err(|e| {
            format_err!(
                "decode hex-encoded string({:?}) failed, caused by error: {}",
                s,
                e
            )
        })?))
    }
}

impl fmt::Display for HexEncodedBytes {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "0x{}", hex::encode(&self.0))?;
        Ok(())
    }
}

impl Serialize for HexEncodedBytes {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        self.to_string().serialize(serializer)
    }
}

impl From<Vec<u8>> for HexEncodedBytes {
    fn from(bytes: Vec<u8>) -> Self {
        Self(bytes)
    }
}
