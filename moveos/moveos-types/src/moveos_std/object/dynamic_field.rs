// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::{
    addresses::MOVEOS_STD_ADDRESS,
    move_std::string::MoveString,
    state::{MoveState, MoveStructState, MoveStructType, MoveType},
};
use anyhow::{anyhow, bail, Result};
use move_core_types::{
    ident_str,
    identifier::IdentStr,
    language_storage::{StructTag, TypeTag},
    value::MoveStructLayout,
};
use move_vm_types::values::{Struct, Value};
use serde::{Deserialize, Serialize};
use std::fmt::Debug;

pub const DYNAMIC_FIELD_STRUCT_NAME: &IdentStr = ident_str!("DynamicField");

/// A wrapper of Object dynamic field value, mirroring `DynamicField<N, V>` in `object.move`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DynamicField<N, V> {
    pub name: N,
    pub value: V,
}

impl<N, V> DynamicField<N, V> {
    pub fn new(name: N, value: V) -> Self {
        Self { name, value }
    }
}

#[derive(Debug, Clone)]
pub struct RawField {
    pub name: Vec<u8>,
    pub name_type: TypeTag,
    pub value: Vec<u8>,
    pub value_type: TypeTag,
}

impl RawField {
    pub fn new(name: Vec<u8>, name_type: TypeTag, value: Vec<u8>, value_type: TypeTag) -> Self {
        Self {
            name,
            name_type,
            value,
            value_type,
        }
    }

    pub fn from_dynamic_field<N, V>(field: &DynamicField<N, V>) -> Self
    where
        N: MoveState,
        V: MoveState,
    {
        Self {
            name: field.name.to_bytes(),
            name_type: N::type_tag(),
            value: field.value.to_bytes(),
            value_type: V::type_tag(),
        }
    }

    /// Parse bcs serialized `DynamicField<MoveString,T>` bytes to `RawField`
    pub fn parse_resource_field(bytes: &[u8], value_type: TypeTag) -> Result<Self> {
        let (used_bytes, name_length) = parse_bcs_length(bytes)?;
        let name_bytes_length = used_bytes + name_length;
        if name_bytes_length > bytes.len() {
            return Err(anyhow!("Invalid name length"));
        }
        let name = &bytes[..name_bytes_length];
        let value = &bytes[name_bytes_length..];
        Ok(Self {
            name: name.to_vec(),
            name_type: MoveString::type_tag(),
            value: value.to_vec(),
            value_type,
        })
    }

    /// Parse bcs serialized `DynamicField<N,V>` bytes to `RawField`
    pub fn parse_unchecked_field(
        bytes: &[u8],
        name_type: TypeTag,
        value_type: TypeTag,
    ) -> Result<Self> {
        // First get the name slice
        let (name_bytes, remaining) = get_bcs_slice(bytes, &name_type)?;
        // Then get the value slice from the remaining bytes
        let (value_bytes, _) = get_bcs_slice(&remaining, &value_type)?;

        Ok(Self {
            name: name_bytes,
            name_type,
            value: value_bytes,
            value_type,
        })
    }
}

//This function is from bcs module,
//find a better way to parse the vec.
pub fn parse_bcs_length(bytes: &[u8]) -> Result<(usize, usize)> {
    let mut value: u64 = 0;
    let mut iter = bytes.iter();
    let mut used_bytes: usize = 0;
    for shift in (0..32).step_by(7) {
        let byte = *iter
            .next()
            .ok_or_else(|| anyhow!("Invalid bytes, NonCanonicalUleb128Encoding"))?;
        used_bytes += 1;
        let digit = byte & 0x7f;
        value |= u64::from(digit) << shift;
        // If the highest bit of `byte` is 0, return the final value.
        if digit == byte {
            if shift > 0 && digit == 0 {
                // We only accept canonical ULEB128 encodings, therefore the
                // heaviest (and last) base-128 digit must be non-zero.
                bail!("Invalid bytes, NonCanonicalUleb128Encoding");
            }
            // Decoded integer must not overflow.
            return Ok((
                used_bytes,
                u32::try_from(value)
                    .map_err(|_| anyhow!("Invalid bytes, IntegerOverflowDuringUleb128Decoding"))?
                    as usize,
            ));
        }
    }
    // Decoded integer must not overflow.
    bail!("Invalid bytes, IntegerOverflowDuringUleb128Decoding")
}

pub fn get_bcs_slice(bytes: &[u8], type_tag: &TypeTag) -> Result<(Vec<u8>, Vec<u8>)> {
    match type_tag {
        // Fixed-length types
        TypeTag::Bool => Ok((bytes[..1].to_vec(), bytes[1..].to_vec())),
        TypeTag::U8 => Ok((bytes[..1].to_vec(), bytes[1..].to_vec())),
        TypeTag::U16 => Ok((bytes[..2].to_vec(), bytes[2..].to_vec())),
        TypeTag::U32 => Ok((bytes[..4].to_vec(), bytes[4..].to_vec())),
        TypeTag::U64 => Ok((bytes[..8].to_vec(), bytes[8..].to_vec())),
        TypeTag::U128 => Ok((bytes[..16].to_vec(), bytes[16..].to_vec())),
        TypeTag::U256 => Ok((bytes[..32].to_vec(), bytes[32..].to_vec())),
        TypeTag::Address => Ok((bytes[..32].to_vec(), bytes[32..].to_vec())), // Assuming 32-byte address

        // Variable-length types
        TypeTag::Vector(_) => {
            // Read length prefix first
            let (prefix_size, length) = parse_bcs_length(bytes)?;
            let data_end = prefix_size + length;
            if data_end > bytes.len() {
                return Err(anyhow!("Invalid vector length"));
            }
            Ok((bytes[..data_end].to_vec(), bytes[data_end..].to_vec()))
        }

        TypeTag::Struct(_struct_tag) => {
            // For structs, it needs custom logic based on the specific struct type
            // This might involve parsing multiple fields
            // TODO find a better way to support parse Struct.
            Err(anyhow!("Unsupported type tag for struct: {:?}", type_tag))
        }

        _ => Err(anyhow!("Unsupported type tag: {:?}", type_tag)),
    }
}

impl<N, V> MoveStructType for DynamicField<N, V>
where
    N: MoveState,
    V: MoveState,
{
    const ADDRESS: move_core_types::account_address::AccountAddress = MOVEOS_STD_ADDRESS;
    const MODULE_NAME: &'static IdentStr = super::MODULE_NAME;
    const STRUCT_NAME: &'static IdentStr = DYNAMIC_FIELD_STRUCT_NAME;

    fn struct_tag() -> StructTag {
        StructTag {
            address: Self::ADDRESS,
            module: Self::MODULE_NAME.to_owned(),
            name: Self::STRUCT_NAME.to_owned(),
            type_params: vec![N::type_tag(), V::type_tag()],
        }
    }
}

impl<N, V> MoveStructState for DynamicField<N, V>
where
    N: MoveState,
    V: MoveState,
{
    fn struct_layout() -> MoveStructLayout {
        MoveStructLayout::new(vec![N::type_layout(), V::type_layout()])
    }

    fn from_runtime_value_struct(value: Struct) -> anyhow::Result<Self>
    where
        Self: Sized,
    {
        let mut fields = value.unpack()?.collect::<Vec<Value>>();
        debug_assert!(fields.len() == 2, "Fields of Field struct must be 2");
        let v = fields.pop().unwrap();
        let k = fields.pop().unwrap();
        Ok(DynamicField {
            name: N::from_runtime_value(k)?,
            value: V::from_runtime_value(v)?,
        })
    }
}

pub fn is_dynamic_field_type(tag: &TypeTag) -> bool {
    match tag {
        TypeTag::Struct(tag) => is_field_struct_tag(tag),
        _ => false,
    }
}

pub fn is_field_struct_tag(tag: &StructTag) -> bool {
    tag.address == MOVEOS_STD_ADDRESS
        && tag.module.as_ref() == super::MODULE_NAME
        && tag.name.as_ref() == DYNAMIC_FIELD_STRUCT_NAME
}

pub fn construct_dynamic_field_struct_tag(name_tag: TypeTag, value_tag: TypeTag) -> StructTag {
    StructTag {
        address: MOVEOS_STD_ADDRESS,
        module: super::MODULE_NAME.to_owned(),
        name: DYNAMIC_FIELD_STRUCT_NAME.to_owned(),
        type_params: vec![name_tag, value_tag],
    }
}

pub fn parse_dynamic_field_type_tags(type_tag: &TypeTag) -> Option<(TypeTag, TypeTag)> {
    if let TypeTag::Struct(struct_tag) = type_tag {
        // Verify this is a DynamicField struct
        if is_field_struct_tag(struct_tag) {
            // DynamicField should have exactly 2 type parameters
            if struct_tag.type_params.len() == 2 {
                // Get Name and Value type tags
                let name_type = struct_tag.type_params[0].clone();
                let value_type = struct_tag.type_params[1].clone();
                return Some((name_type, value_type));
            }
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{move_std::string::MoveString, state::MoveType};
    use move_core_types::u256::U256;
    use move_core_types::{account_address::AccountAddress, value::MoveTypeLayout};

    #[derive(Eq, PartialEq, Debug, Clone, Deserialize, Serialize)]
    struct TestStruct {
        count: u64,
    }

    impl MoveStructType for TestStruct {
        const ADDRESS: AccountAddress = MOVEOS_STD_ADDRESS;
        const MODULE_NAME: &'static IdentStr = ident_str!("object");
        const STRUCT_NAME: &'static IdentStr = ident_str!("TestStruct");
    }

    impl MoveStructState for TestStruct {
        fn struct_layout() -> MoveStructLayout {
            MoveStructLayout::new(vec![MoveTypeLayout::U64])
        }
    }

    #[test]
    fn test_dynamic_field() {
        let field = DynamicField::new(
            // MoveString::from(TestStruct::struct_tag().to_canonical_string()),
            MoveString::from("abc"),
            TestStruct { count: 10 },
        );
        let raw_field_bytes = bcs::to_bytes(&field).unwrap();
        let raw_field = RawField::from_dynamic_field(&field);
        let parsed_raw_field =
            RawField::parse_resource_field(&raw_field_bytes, TestStruct::type_tag()).unwrap();
        assert_eq!(raw_field.name, parsed_raw_field.name);
        assert_eq!(raw_field.value, parsed_raw_field.value);
    }

    #[test]
    fn test_arbitrary_dynamic_field() {
        let field = DynamicField::new(
            AccountAddress::from_hex_literal("0x80").unwrap(),
            U256::from(5u64),
        );
        let raw_field_bytes = bcs::to_bytes(&field).unwrap();
        let raw_field = RawField::from_dynamic_field(&field);
        let parsed_raw_field = RawField::parse_unchecked_field(
            &raw_field_bytes,
            AccountAddress::type_tag(),
            U256::type_tag(),
        )
        .unwrap();
        assert_eq!(raw_field.name, parsed_raw_field.name);
        assert_eq!(raw_field.value, parsed_raw_field.value);

        let raw_bytes_arb: Vec<u8> = [
            154, 222, 70, 99, 77, 21, 240, 0, 252, 167, 186, 194, 242, 33, 33, 25, 216, 35, 196,
            83, 132, 111, 13, 92, 78, 68, 1, 54, 172, 160, 246, 92, 183, 24, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        ]
        .to_vec();
        let _parsed_raw_field_arb = RawField::parse_unchecked_field(
            &raw_bytes_arb,
            AccountAddress::type_tag(),
            U256::type_tag(),
        )
        .unwrap();
    }
}
