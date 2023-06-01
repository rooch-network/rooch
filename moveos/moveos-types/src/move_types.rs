// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use anyhow::{anyhow, format_err, Result};
use move_core_types::{
    account_address::AccountAddress, identifier::Identifier, language_storage::ModuleId,
    language_storage::StructTag, language_storage::TypeTag,
};
use serde::{Deserialize, Serialize, Serializer};
use std::fmt;
use std::str::FromStr;

/// Identifier of a module function
/// The FunctionId is of the form <address>::<module>::<function>
#[derive(Clone, Debug, Eq, Ord, PartialOrd, PartialEq, Serialize, Deserialize, Hash)]
pub struct FunctionId {
    pub module_id: ModuleId,
    pub function_name: Identifier,
}

impl FunctionId {
    pub fn new(module_id: ModuleId, function_name: Identifier) -> Self {
        Self {
            module_id,
            function_name,
        }
    }
}

impl std::fmt::Display for FunctionId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}::{}", &self.module_id, &self.function_name)
    }
}

impl FromStr for FunctionId {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (module, function_name) = parse_struct_or_function_id(s)?;
        Ok(Self {
            module_id: module,
            function_name,
        })
    }
}

/// Identifier of a module struct
/// The StructId is of the form <address>::<module>::<struct>
#[derive(Clone, Debug, Eq, Ord, PartialOrd, PartialEq, Serialize, Deserialize, Hash)]
pub struct StructId {
    pub module_id: ModuleId,
    pub struct_name: Identifier,
}

impl StructId {
    pub fn new(module_id: ModuleId, struct_name: Identifier) -> Self {
        Self {
            module_id,
            struct_name,
        }
    }
}

impl std::fmt::Display for StructId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}::{}", &self.module_id, &self.struct_name)
    }
}

impl FromStr for StructId {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (module_id, struct_name) = parse_struct_or_function_id(s)?;
        Ok(Self {
            module_id,
            struct_name,
        })
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

fn parse_struct_or_function_id(function_or_struct_id: &str) -> Result<(ModuleId, Identifier)> {
    let ids: Vec<&str> = function_or_struct_id.split_terminator("::").collect();
    if ids.len() != 3 {
        return Err(anyhow!(
            "StructId is not well formed.  Must be of the form <address>::<module>::<function>"
        ));
    }
    let address = AccountAddress::from_str(ids.first().unwrap())
        .map_err(|err| anyhow!("Module address error: {:?}", err.to_string()))?;
    let module = Identifier::from_str(ids.get(1).unwrap())
        .map_err(|err| anyhow!("Module name error: {:?}", err.to_string()))?;
    let function_or_struct_id = Identifier::from_str(ids.get(2).unwrap())
        .map_err(|err| anyhow!("Function or Struct name error: {:?}", err.to_string()))?;
    let module_id = ModuleId::new(address, module);
    Ok((module_id, function_or_struct_id))
}

/// check the filter TypeTag is match with the Target, if the filter and target both are StructTag, call `struct_tag_match`, otherwise, same as `==`
pub fn type_tag_match(filter: &TypeTag, target: &TypeTag) -> bool {
    if let (TypeTag::Struct(filter), TypeTag::Struct(target)) = (filter, target) {
        struct_tag_match(filter, target)
    } else {
        filter == target
    }
}

/// check the filter StructTag is match with the target.
pub fn struct_tag_match(filter: &StructTag, target: &StructTag) -> bool {
    if filter == target {
        return true;
    }

    if filter.address != target.address
        || filter.module != target.module
        || filter.name != target.name
    {
        return false;
    }

    if filter.type_params.is_empty() {
        return true;
    }

    if filter.type_params.len() != target.type_params.len() {
        return false;
    }

    for (filter_type_tag, target_type_tag) in
        std::iter::zip(filter.type_params.clone(), target.type_params.clone())
    {
        if !type_tag_match(&filter_type_tag, &target_type_tag) {
            return false;
        }
    }
    true
}
