// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use anyhow::{anyhow, bail, format_err, Result};
use move_core_types::{
    account_address::AccountAddress, identifier::Identifier, language_storage::ModuleId,
    language_storage::StructTag, language_storage::TypeTag,
};
use serde::{Deserialize, Serialize, Serializer};
use std::fmt;
use std::str::FromStr;

use crate::addresses::MOVEOS_STD_ADDRESS;
use crate::object;
#[cfg(any(test, feature = "fuzzing"))]
use proptest::prelude::*;

/// Identifier of a module function
/// The FunctionId is of the form <address>::<module>::<function>
#[derive(Clone, Debug, Eq, Ord, PartialOrd, PartialEq, Serialize, Deserialize, Hash)]
pub struct FunctionId {
    pub module_id: ModuleId,
    pub function_name: Identifier,
}

impl FunctionId {
    pub const fn new(module_id: ModuleId, function_name: Identifier) -> Self {
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

pub fn parse_module_id(s: &str) -> Result<ModuleId, anyhow::Error> {
    let parts: Vec<_> = s.split("::").collect();
    if parts.len() != 2 {
        anyhow::bail!("invalid module id");
    }
    let module_addr = parts[0].parse::<AccountAddress>()?;
    let module_name = Identifier::new(parts[1])?;
    Ok(ModuleId::new(module_addr, module_name))
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

/// The structure of TypeInfo is consistent of contract type_info
#[derive(Clone, Debug, Eq, Ord, PartialOrd, PartialEq, Serialize, Deserialize, Hash)]
pub struct TypeInfo {
    pub account_address: AccountAddress,
    pub module_name: Identifier,
    pub struct_name: Identifier,
}

impl TypeInfo {
    pub fn new(
        account_address: AccountAddress,
        module_name: Identifier,
        struct_name: Identifier,
    ) -> Self {
        Self {
            account_address,
            module_name,
            struct_name,
        }
    }
}

pub fn as_struct_tag(type_tag: TypeTag) -> Result<StructTag> {
    if let TypeTag::Struct(struct_tag) = type_tag {
        Ok(*struct_tag)
    } else {
        bail!("invalid struct tag: {:?}", type_tag)
    }
}

#[cfg(any(test, feature = "fuzzing"))]
pub fn type_tag_prop_strategy() -> impl Strategy<Value = TypeTag> {
    let leaf = prop_oneof![
        Just(TypeTag::Bool),
        Just(TypeTag::U8),
        Just(TypeTag::U16),
        Just(TypeTag::U32),
        Just(TypeTag::U64),
        Just(TypeTag::U128),
        Just(TypeTag::U256),
        Just(TypeTag::Address),
        Just(TypeTag::Signer),
    ];

    let type_tag_strategy = leaf.prop_recursive(
        8,   // Arbitrarily chosen depth, adjust to suit your needs
        256, // Arbitrarily chosen size limit, adjust to suit your needs
        10,  // Per-vec limit, adjust to suit your needs
        |elem| {
            prop_oneof![
                // Recursively generate TypeTag for Vector
                elem.clone().prop_map(|t| TypeTag::Vector(Box::new(t))),
                // Recursively generate TypeTag for StructTag
                any::<Vec<TypeTag>>()
                    .prop_flat_map(move |type_params| {
                        (
                            any::<Identifier>(),
                            any::<Identifier>(),
                            Just(AccountAddress::random()),
                            Just(type_params),
                        )
                    })
                    .prop_map(|(module, name, address, type_params)| {
                        TypeTag::Struct(Box::new(StructTag {
                            address,
                            module,
                            name,
                            type_params,
                        }))
                    }),
            ]
        },
    );

    type_tag_strategy
}

pub fn is_table(struct_tag: &StructTag) -> bool {
    struct_tag.address == MOVEOS_STD_ADDRESS
        && struct_tag.module.as_ident_str() == object::TABLE_INFO_MODULE_NAME
        && struct_tag.name.as_ident_str() == object::TABLE_INFO_STRUCT_NAME
}
