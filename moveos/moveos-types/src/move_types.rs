// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use anyhow::{anyhow, bail, Result};
use move_core_types::{
    account_address::AccountAddress, identifier::Identifier, language_storage::ModuleId,
    language_storage::StructTag, language_storage::TypeTag,
};
#[cfg(any(test, feature = "fuzzing"))]
use proptest::prelude::*;
use rand::prelude::{Distribution, SliceRandom};
use rand::rngs::OsRng;
use rand::Rng;
use serde::{Deserialize, Serialize};
use std::str::FromStr;

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

pub fn as_struct_tag(type_tag: TypeTag) -> Result<StructTag> {
    if let TypeTag::Struct(struct_tag) = type_tag {
        Ok(*struct_tag)
    } else {
        bail!("Invalid struct tag: {:?}", type_tag)
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

struct IdentifierSymbols;

impl Distribution<char> for IdentifierSymbols {
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> char {
        //TODO add more valid identity char
        *b"abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ"
            .choose(rng)
            .unwrap_or(&97) as char
    }
}

pub fn random_identity() -> Identifier {
    let rng = OsRng;
    let id: String = rng.sample_iter(&IdentifierSymbols).take(7).collect();
    Identifier::new(id).expect("random identity should valid.")
}

pub fn random_struct_tag() -> StructTag {
    StructTag {
        address: AccountAddress::random(),
        module: random_identity(),
        name: random_identity(),
        type_params: vec![],
    }
}

pub fn random_type_tag() -> TypeTag {
    TypeTag::Struct(Box::new(random_struct_tag()))
}

pub fn get_first_ty_as_struct_tag(struct_tag: StructTag) -> Result<StructTag> {
    if let Some(first_ty) = struct_tag.type_params.first() {
        let first_ty_as_struct_tag = match first_ty {
            TypeTag::Struct(first_struct_tag) => *first_struct_tag.clone(),
            _ => bail!("Invalid struct tag: {:?}", struct_tag),
        };
        Ok(first_ty_as_struct_tag)
    } else {
        bail!("Invalid struct tag: {:?}", struct_tag)
    }
}

#[test]
fn test_type_tag() -> Result<()> {
    let type_tag_str = "0x5::test::struct";
    let type_tag_canonical_str =
        "0000000000000000000000000000000000000000000000000000000000000005::test::struct";
    let test_type_tag = TypeTag::from_str(type_tag_str)?;
    assert_eq!(type_tag_str, test_type_tag.to_string());
    assert_eq!(type_tag_canonical_str, test_type_tag.to_canonical_string());
    Ok(())
}
