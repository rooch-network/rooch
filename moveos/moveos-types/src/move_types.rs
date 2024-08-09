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

    leaf.prop_recursive(
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
    )
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

/// Parse a type tag from a canonical string
/// This is the reverse of TypeTag::to_canonical_string()
pub fn from_canonical_string(s: &str) -> Result<TypeTag> {
    match s {
        "bool" => Ok(TypeTag::Bool),
        "u8" => Ok(TypeTag::U8),
        "u16" => Ok(TypeTag::U16),
        "u32" => Ok(TypeTag::U32),
        "u64" => Ok(TypeTag::U64),
        "u128" => Ok(TypeTag::U128),
        "u256" => Ok(TypeTag::U256),
        "address" => Ok(TypeTag::Address),
        "signer" => Ok(TypeTag::Signer),
        _ => {
            if s.starts_with("vector<") && s.ends_with('>') {
                let inner = &s[7..s.len() - 1];
                let inner_type = from_canonical_string(inner)?;
                Ok(TypeTag::Vector(Box::new(inner_type)))
            } else {
                struct_tag_from_canonical_string(s).map(|s| TypeTag::Struct(Box::new(s)))
            }
        }
    }
}

fn struct_tag_from_canonical_string(s: &str) -> Result<StructTag> {
    let parts: Vec<&str> = s.splitn(3, "::").collect();
    if parts.len() < 3 {
        return Err(anyhow::anyhow!("Invalid struct tag format"));
    }

    let first_double_colon = s
        .find("::")
        .ok_or_else(|| anyhow::anyhow!("Invalid struct tag format: missing first '::'"))?;
    let second_double_colon = s[first_double_colon + 2..]
        .find("::")
        .map(|i| i + first_double_colon + 2)
        .ok_or_else(|| anyhow::anyhow!("Invalid struct tag format: missing second '::'"))?;

    let address = AccountAddress::from_hex(&s[..first_double_colon])?;
    let module = Identifier::new(&s[first_double_colon + 2..second_double_colon])?;

    let name_and_generics = &s[second_double_colon + 2..];
    let name_end = name_and_generics
        .find('<')
        .unwrap_or(name_and_generics.len());
    let name = Identifier::new(&name_and_generics[..name_end])?;

    let type_params = if name_end < name_and_generics.len() {
        // ensure the last char is '>' and params are not empty
        if !name_and_generics.ends_with('>') || name_and_generics[name_end + 1..].trim() == ">" {
            return Err(anyhow::anyhow!("Invalid generic parameters format"));
        }
        let generics = &name_and_generics[name_end + 1..name_and_generics.len() - 1];
        parse_type_params(generics)?
    } else {
        vec![]
    };

    Ok(StructTag {
        address,
        module,
        name,
        type_params,
    })
}

fn parse_type_params(s: &str) -> Result<Vec<TypeTag>, anyhow::Error> {
    if s.trim().is_empty() {
        return Err(anyhow::anyhow!("Empty type parameters are not allowed"));
    }

    let mut params = Vec::new();
    let mut depth = 0;
    let mut start = 0;

    for (i, c) in s.char_indices() {
        match c {
            '<' => depth += 1,
            '>' => depth -= 1,
            ',' if depth == 0 => {
                let param = from_canonical_string(s[start..i].trim())?;
                params.push(param);
                start = i + 1;
            }
            _ => {}
        }
    }

    if start < s.len() {
        let param = from_canonical_string(s[start..].trim())?;
        params.push(param);
    }

    if params.is_empty() {
        return Err(anyhow::anyhow!("Empty type parameters are not allowed"));
    }

    Ok(params)
}

/// Parse a type tag from a string
/// This function support parse type tag from TypeTag::to_string() or TypeTag::to_canonical_string()
/// The canonical string format is:
/// `0000000000000000000000000000000a::module_name1::type_name1<0000000000000000000000000000000a::module_name2::type_name2<u64>>
/// The non-canonical string format is:
/// `0xa::module_name1::type_name1<0xa::module_name2::type_name2<u64>>`
/// TODO: Should we unify the canonical and non-canonical string format?
/// https://github.com/rooch-network/rooch/issues/2395
pub fn parse_type_tag(s: &str) -> Result<TypeTag> {
    let s = s.trim();
    if s.starts_with("0x") {
        TypeTag::from_str(s)
    } else {
        from_canonical_string(s)
    }
}

/// Parse a struct tag from a string
/// This function support parse struct tag from StructTag::to_string() or StructTag::to_canonical_string()
pub fn parse_struct_tag(s: &str) -> Result<StructTag> {
    let s = s.trim();
    if s.starts_with("0x") {
        StructTag::from_str(s)
    } else {
        struct_tag_from_canonical_string(s)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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

    #[test]
    fn test_type_tag_from_canonical_string() {
        // primitive types
        assert_eq!(from_canonical_string("bool").unwrap(), TypeTag::Bool);
        assert_eq!(from_canonical_string("u8").unwrap(), TypeTag::U8);
        assert_eq!(from_canonical_string("u64").unwrap(), TypeTag::U64);
        assert_eq!(from_canonical_string("u128").unwrap(), TypeTag::U128);
        assert_eq!(from_canonical_string("address").unwrap(), TypeTag::Address);
        assert_eq!(from_canonical_string("signer").unwrap(), TypeTag::Signer);

        // Vector
        assert_eq!(
            from_canonical_string("vector<u8>").unwrap(),
            TypeTag::Vector(Box::new(TypeTag::U8))
        );
        assert_eq!(
            from_canonical_string("vector<vector<u64>>").unwrap(),
            TypeTag::Vector(Box::new(TypeTag::Vector(Box::new(TypeTag::U64))))
        );

        // Struct
        let struct_tag = from_canonical_string(
            "0000000000000000000000000000000000000000000000000000000000000001::string::String",
        )
        .unwrap();
        if let TypeTag::Struct(s) = struct_tag {
            assert_eq!(s.address, AccountAddress::from_hex_literal("0x1").unwrap());
            assert_eq!(s.module.as_str(), "string");
            assert_eq!(s.name.as_str(), "String");
            assert!(s.type_params.is_empty());
        } else {
            panic!("Expected Struct TypeTag");
        }

        // struct with type params
        let complex_struct = from_canonical_string("0000000000000000000000000000000000000000000000000000000000000001::table::Table<0000000000000000000000000000000000000000000000000000000000000001::string::String,u64>").unwrap();
        if let TypeTag::Struct(s) = complex_struct {
            assert_eq!(s.address, AccountAddress::from_hex_literal("0x1").unwrap());
            assert_eq!(s.module.as_str(), "table");
            assert_eq!(s.name.as_str(), "Table");
            assert_eq!(s.type_params.len(), 2);
        } else {
            panic!("Expected Struct TypeTag");
        }

        // error cases
        assert!(from_canonical_string("invalid_type").is_err());
        assert!(from_canonical_string("vector<>").is_err());
        assert!(from_canonical_string(
            "0000000000000000000000000000000000000000000000000000000000000001::module::"
        )
        .is_err());
    }

    #[test]
    fn test_struct_tag_from_canonical_string() {
        // struct
        let basic_struct = struct_tag_from_canonical_string(
            "0000000000000000000000000000000000000000000000000000000000000001::module::Name",
        )
        .unwrap();
        assert_eq!(
            basic_struct.address,
            AccountAddress::from_hex_literal("0x1").unwrap()
        );
        assert_eq!(basic_struct.module.as_str(), "module");
        assert_eq!(basic_struct.name.as_str(), "Name");
        assert!(basic_struct.type_params.is_empty());

        // one type param
        let single_generic = struct_tag_from_canonical_string(
            "0000000000000000000000000000000000000000000000000000000000000001::vector::Vector<u8>",
        )
        .unwrap();
        assert_eq!(
            single_generic.address,
            AccountAddress::from_hex_literal("0x1").unwrap()
        );
        assert_eq!(single_generic.module.as_str(), "vector");
        assert_eq!(single_generic.name.as_str(), "Vector");
        assert_eq!(single_generic.type_params.len(), 1);
        assert_eq!(single_generic.type_params[0], TypeTag::U8);

        // multiple type params
        let multi_generic = struct_tag_from_canonical_string("0000000000000000000000000000000000000000000000000000000000000001::table::Table<0000000000000000000000000000000000000000000000000000000000000001::string::String,u64>").unwrap();
        assert_eq!(
            multi_generic.address,
            AccountAddress::from_hex_literal("0x1").unwrap()
        );
        assert_eq!(multi_generic.module.as_str(), "table");
        assert_eq!(multi_generic.name.as_str(), "Table");
        assert_eq!(multi_generic.type_params.len(), 2);

        // nested generic
        let nested_generic = struct_tag_from_canonical_string("0000000000000000000000000000000000000000000000000000000000000001::complex::Type<vector<0000000000000000000000000000000000000000000000000000000000000001::coin::Coin<0000000000000000000000000000000000000000000000000000000000000001::gas_coin::GasCoin>>>").unwrap();
        assert_eq!(
            nested_generic.address,
            AccountAddress::from_hex_literal("0x1").unwrap()
        );
        assert_eq!(nested_generic.module.as_str(), "complex");
        assert_eq!(nested_generic.name.as_str(), "Type");
        assert_eq!(nested_generic.type_params.len(), 1);

        if let TypeTag::Vector(inner) = &nested_generic.type_params[0] {
            if let TypeTag::Struct(coin_struct) = inner.as_ref() {
                assert_eq!(coin_struct.module.as_str(), "coin");
                assert_eq!(coin_struct.name.as_str(), "Coin");
                assert_eq!(coin_struct.type_params.len(), 1);
                if let TypeTag::Struct(gas_coin_struct) = &coin_struct.type_params[0] {
                    assert_eq!(gas_coin_struct.module.as_str(), "gas_coin");
                    assert_eq!(gas_coin_struct.name.as_str(), "GasCoin");
                } else {
                    panic!("Expected Struct TypeTag for GasCoin");
                }
            } else {
                panic!("Expected Struct TypeTag for Coin");
            }
        } else {
            panic!("Expected Vector TypeTag");
        }

        assert!(struct_tag_from_canonical_string(
            "0000000000000000000000000000000000000000000000000000000000000001::module"
        )
        .is_err());
        assert!(struct_tag_from_canonical_string(
            "0000000000000000000000000000000000000000000000000000000000000001::module::Name<>"
        )
        .is_err());
        assert!(struct_tag_from_canonical_string(
            "0000000000000000000000000000000000000000000000000000000000000001::module::Name<u8"
        )
        .is_err());
        assert!(struct_tag_from_canonical_string("invalid::module::Name").is_err());
    }
}
