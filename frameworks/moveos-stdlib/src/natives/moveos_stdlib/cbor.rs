// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::natives::helpers::{make_module_natives, make_native};
use anyhow::Result;
use ciborium::de::from_reader;
use ciborium::ser::into_writer;
use ciborium::value::Value as CborValue;
use log::debug;
use move_binary_format::errors::{PartialVMError, PartialVMResult};
use move_core_types::account_address::AccountAddress;
use move_core_types::gas_algebra::{InternalGas, InternalGasPerByte, NumBytes};
use move_core_types::identifier::Identifier;
use move_core_types::language_storage::{StructTag, TypeTag};
use move_core_types::value::MoveStruct;
use move_core_types::value::MoveValue as CoreMoveValue;
use move_core_types::value::{MoveFieldLayout, MoveStructLayout, MoveTypeLayout};
use move_core_types::vm_status::StatusCode;
use move_vm_runtime::native_functions::{NativeContext, NativeFunction};
use move_vm_types::{
    loaded_data::runtime_types::Type,
    natives::function::NativeResult,
    pop_arg,
    values::{values_impl::Reference, Struct, Value as MoveValue, Vector},
};

use move_core_types::u256::{self, U256_NUM_BYTES};
use moveos_types::addresses::MOVE_STD_ADDRESS;
use moveos_types::move_std::string::MoveString;
use moveos_types::moveos_std::simple_map::{Element, SimpleMap};
use moveos_types::state::{MoveStructType, MoveType};
use primitive_types::U128 as PrimitiveU128;
use primitive_types::U256 as PrimitiveU256;
use smallvec::smallvec;
use std::collections::VecDeque;
use std::io::Cursor;

const STATUS_CODE_FAILED_TO_SERIALIZE_VALUE: u64 = 1;
const E_CBOR_SERIALIZATION_FAILURE: u64 = 2;

const TAG_BIGPOS: u64 = 2;

fn parse_move_value_from_cbor(
    layout: &MoveTypeLayout,
    bytes: Vec<u8>,
    context: &NativeContext,
) -> Result<MoveValue> {
    let cursor = Cursor::new(bytes);
    let cbor_value: CborValue = from_reader(cursor)?;

    parse_move_value_from_cbor_value(layout, &cbor_value, context)
}

fn parse_struct_value_from_cbor_value(
    layout: &MoveStructLayout,
    cbor_value: &CborValue,
    context: &NativeContext,
) -> Result<Struct> {
    if let MoveStructLayout::WithTypes {
        type_: struct_type,
        fields: move_fields_layout,
    } = layout
    {
        if struct_type.is_std_string(&MOVE_STD_ADDRESS) {
            let str_value = cbor_value
                .as_text()
                .ok_or_else(|| anyhow::anyhow!("Invalid string value"))?;
            Ok(Struct::pack(vec![MoveValue::vector_u8(
                str_value.as_bytes().to_vec(),
            )]))
        } else if struct_type.is_ascii_string(&MOVE_STD_ADDRESS) {
            let str_value = cbor_value
                .as_text()
                .ok_or_else(|| anyhow::anyhow!("Invalid ascii string value"))?;
            if !str_value.is_ascii() {
                return Err(anyhow::anyhow!("Invalid ascii string value"));
            }
            Ok(Struct::pack(vec![MoveValue::vector_u8(
                str_value.as_bytes().to_vec(),
            )]))
        } else if is_std_option(struct_type, &MOVE_STD_ADDRESS) {
            let mut vec_value = Vec::new();
            let vec_layout = move_fields_layout.first().unwrap();
            let type_tag: TypeTag = (&vec_layout.layout).try_into()?;
            let ty = context.load_type(&type_tag)?;

            if let CborValue::Null = cbor_value {
                let value = Vector::pack(&ty, vec_value)?;
                return Ok(Struct::pack(vec![value]));
            }

            if let MoveTypeLayout::Vector(vec_layout) = vec_layout.layout.clone() {
                let struct_layout = vec_layout.as_ref();
                let move_struct_value =
                    parse_move_value_from_cbor_value(struct_layout, cbor_value, context)?;
                vec_value.push(move_struct_value);
            }

            let value = Vector::pack(&ty, vec_value)?;
            Ok(Struct::pack(vec![value]))
        } else if struct_type == &SimpleMap::<MoveString, Vec<u8>>::struct_tag() {
            let key_value_pairs = cbor_obj_to_key_value_pairs(cbor_value)?;
            let mut key_values = Vec::new();

            for (key, bytes) in key_value_pairs {
                key_values.push(MoveValue::struct_(Struct::pack(vec![
                    MoveValue::struct_(Struct::pack(vec![MoveValue::vector_u8(
                        key.as_bytes().to_vec(),
                    )])),
                    MoveValue::vector_u8(bytes),
                ])));
            }

            let element_type = context.load_type(&Element::<MoveString, Vec<u8>>::type_tag())?;
            Ok(Struct::pack(vec![Vector::pack(&element_type, key_values)?]))
        } else {
            match cbor_value {
                CborValue::Map(cbor_map) => {
                    let field_values = move_fields_layout
                        .iter()
                        .map(|field| -> Result<MoveValue> {
                            let name = field.name.as_str();
                            let cbor_field = &cbor_map
                                .iter()
                                .find(|(key, _)| match key.clone().into_text() {
                                    Ok(text) => text == name,
                                    Err(_) => false,
                                })
                                .ok_or_else(|| {
                                    anyhow::anyhow!("type: {}, Missing field {}", struct_type, name)
                                })?
                                .1;
                            parse_move_value_from_cbor_value(&field.layout, cbor_field, context)
                        })
                        .collect::<Result<Vec<MoveValue>>>()?;
                    Ok(Struct::pack(field_values))
                }
                CborValue::Array(cbor_fields) => {
                    let field_values = move_fields_layout
                        .iter()
                        .zip(cbor_fields)
                        .map(|(field_layout, cbor_value)| -> Result<MoveValue> {
                            parse_move_value_from_cbor_value(
                                &field_layout.layout,
                                cbor_value,
                                context,
                            )
                        })
                        .collect::<Result<Vec<MoveValue>>>()?;

                    Ok(Struct::pack(field_values))
                }
                _ => unreachable!(),
            }
        }
    } else {
        Err(anyhow::anyhow!("Invalid MoveStructLayout"))
    }
}

fn cbor_obj_to_key_value_pairs(cbor_value: &CborValue) -> Result<Vec<(String, Vec<u8>)>> {
    if let CborValue::Map(cbor_map) = cbor_value {
        let key_value_pairs = cbor_map
            .iter()
            .map(|(key, value)| {
                let name = key
                    .clone()
                    .into_text()
                    .ok()
                    .ok_or_else(|| anyhow::anyhow!("Invalid key"))?;
                let cbor_field = value;

                let bytes = match cbor_field {
                    CborValue::Bytes(t) => t.clone(),
                    _ => {
                        let mut writer = Vec::new();
                        into_writer(&cbor_field, &mut writer)?;
                        writer
                    }
                };

                Ok((name, bytes))
            })
            .collect::<Result<Vec<(String, Vec<u8>)>>>()?;

        Ok(key_value_pairs)
    } else {
        Err(anyhow::anyhow!("Invalid cbor object"))
    }
}

fn parse_move_value_from_cbor_value(
    layout: &MoveTypeLayout,
    cbor_value: &CborValue,
    context: &NativeContext,
) -> Result<MoveValue> {
    match layout {
        // Parse a boolean value
        MoveTypeLayout::Bool => {
            let bool_value = cbor_value
                .as_bool()
                .ok_or_else(|| anyhow::anyhow!("Invalid bool value"))?;
            Ok(MoveValue::bool(bool_value))
        }
        // Parse an unsigned 8-bit integer
        MoveTypeLayout::U8 => {
            let u8_value = cbor_value
                .as_integer()
                .and_then(|int| u8::try_from(int).ok())
                .ok_or_else(|| anyhow::anyhow!("Invalid u8 value"))?;
            Ok(MoveValue::u8(u8_value))
        }
        // Parse an unsigned 64-bit integer
        MoveTypeLayout::U64 => {
            let u64_value = cbor_value
                .as_integer()
                .and_then(|int| u64::try_from(int).ok())
                .ok_or_else(|| anyhow::anyhow!("Invalid u64 value"))?;
            Ok(MoveValue::u64(u64_value))
        }
        // Parse an unsigned 128-bit integer
        MoveTypeLayout::U128 => {
            let (tag, value) = cbor_value
                .as_tag()
                .ok_or_else(|| anyhow::anyhow!("Invalid u128 value"))?;

            // Verify tag is correct
            if tag != TAG_BIGPOS {
                return Err(anyhow::anyhow!("Invalid CBOR tag for u128 value"));
            }

            let u128_bytes = value
                .as_bytes()
                .ok_or_else(|| anyhow::anyhow!("Invalid u128 value"))?;

            let u128_value = PrimitiveU128::from_big_endian(u128_bytes);
            Ok(MoveValue::u128(u128_value.as_u128()))
        }

        // Parse an address value
        MoveTypeLayout::Address => {
            let bytes = cbor_value
                .as_bytes()
                .ok_or_else(|| anyhow::anyhow!("Invalid address value"))?;
            let addr = AccountAddress::from_bytes(bytes)
                .map_err(|_| anyhow::anyhow!("Invalid address value"))?;
            Ok(MoveValue::address(addr))
        }
        // Parse a vector value
        MoveTypeLayout::Vector(item_layout) => {
            let layout = item_layout.as_ref();

            if let MoveTypeLayout::U8 = layout {
                let bytes = cbor_value
                    .as_bytes()
                    .ok_or_else(|| anyhow::anyhow!("Invalid bytes"))?;
                Ok(MoveValue::vector_u8(bytes.clone()))
            } else {
                let vec_value = cbor_value
                    .as_array()
                    .ok_or_else(|| anyhow::anyhow!("Invalid vector value"))?
                    .iter()
                    .map(|v| parse_move_value_from_cbor_value(item_layout, v, context))
                    .collect::<Result<Vec<_>>>()?;
                let type_tag: TypeTag = (&**item_layout).try_into()?;
                let ty = context.load_type(&type_tag)?;
                let value = Vector::pack(&ty, vec_value)?;
                Ok(value)
            }
        }
        // Parse a struct value
        MoveTypeLayout::Struct(struct_layout) => {
            let struct_value =
                parse_struct_value_from_cbor_value(struct_layout, cbor_value, context)?;
            Ok(MoveValue::struct_(struct_value))
        }
        // Signer type is not supported
        MoveTypeLayout::Signer => Err(anyhow::anyhow!("Do not support Signer type")),
        // Parse an unsigned 16-bit integer
        MoveTypeLayout::U16 => {
            let u16_value = cbor_value
                .as_integer()
                .and_then(|int| u16::try_from(int).ok())
                .ok_or_else(|| anyhow::anyhow!("Invalid u16 value"))?;
            Ok(MoveValue::u16(u16_value))
        }
        // Parse an unsigned 32-bit integer
        MoveTypeLayout::U32 => {
            let u32_value = cbor_value
                .as_integer()
                .and_then(|int| u32::try_from(int).ok())
                .ok_or_else(|| anyhow::anyhow!("Invalid u32 value"))?;
            Ok(MoveValue::u32(u32_value))
        }
        // Parse an unsigned 256-bit integer
        MoveTypeLayout::U256 => {
            let (tag, value) = cbor_value
                .as_tag()
                .ok_or_else(|| anyhow::anyhow!("Invalid u128 value"))?;

            // Verify tag is correct
            if tag != TAG_BIGPOS {
                return Err(anyhow::anyhow!("Invalid CBOR tag for u256 value"));
            }

            let u256_bytes = value
                .as_bytes()
                .ok_or_else(|| anyhow::anyhow!("Invalid u256 value"))?;

            let value = PrimitiveU256::from_big_endian(u256_bytes);
            let mut buffer = [0u8; U256_NUM_BYTES];
            value.to_little_endian(&mut buffer);
            Ok(MoveValue::u256(u256::U256::from_le_bytes(&buffer)))
        }
    }
}

fn serialize_move_value_to_cbor(layout: &MoveTypeLayout, value: &CoreMoveValue) -> Result<Vec<u8>> {
    let mut writer = Vec::new();

    let cbor_value = serialize_move_value_to_cbor_value(layout, value)?;
    into_writer(&cbor_value, &mut writer)?;

    Ok(writer)
}

fn serialize_move_value_to_cbor_value(
    layout: &MoveTypeLayout,
    value: &CoreMoveValue,
) -> Result<CborValue> {
    use CoreMoveValue as MoveValue;
    use MoveTypeLayout as L;

    let cbor_value = match (layout, value) {
        (L::Struct(layout), MoveValue::Struct(struct_)) => {
            serialize_move_struct_to_cbor_value(layout, struct_)?
        }
        (L::Bool, MoveValue::Bool(b)) => CborValue::from(*b),
        (L::U8, MoveValue::U8(b)) => CborValue::from(*b),
        (L::U16, MoveValue::U16(b)) => CborValue::from(*b),
        (L::U32, MoveValue::U32(b)) => CborValue::from(*b),
        (L::U64, MoveValue::U64(b)) => CborValue::from(*b),
        (L::U128, MoveValue::U128(b)) => CborValue::from(*b),
        (L::U256, MoveValue::U256(i)) => {
            let slice = i.to_le_bytes();
            let value = PrimitiveU256::from_little_endian(&slice);
            let leading_empty_bytes = value.leading_zeros() as usize / 8;
            let mut buffer = [0u8; U256_NUM_BYTES];
            value.to_big_endian(&mut buffer);
            let bytes = buffer[leading_empty_bytes..].to_vec();
            CborValue::Tag(TAG_BIGPOS, CborValue::Bytes(bytes).into())
        }
        (L::Address, MoveValue::Address(addr)) => CborValue::Bytes(addr.to_vec()),
        (L::Signer, MoveValue::Signer(_a)) => {
            return Err(anyhow::anyhow!("Do not support Signer type"))
        }
        (L::Vector(vec_layout), MoveValue::Vector(vec)) => {
            let layout = vec_layout.as_ref();

            if let L::U8 = layout {
                let mut cbor_vec = Vec::new();

                for item in vec.iter() {
                    if let MoveValue::U8(b) = item {
                        cbor_vec.push(*b);
                    }
                }

                CborValue::Bytes(cbor_vec)
            } else {
                let mut cbor_vec = Vec::new();

                for item in vec.iter() {
                    let cbor_value = serialize_move_value_to_cbor_value(layout, item)?;
                    cbor_vec.push(cbor_value);
                }

                CborValue::Array(cbor_vec)
            }
        }
        _ => {
            return Err(anyhow::anyhow!(
                "Invalid combination of MoveStructLayout and MoveStruct"
            ))
        }
    };

    Ok(cbor_value)
}

fn serialize_move_struct_to_cbor_value(
    layout: &MoveStructLayout,
    struct_: &MoveStruct,
) -> Result<CborValue> {
    use CoreMoveValue as MoveValue;
    use MoveStructLayout as L;

    let value = match (layout, struct_) {
        (L::Runtime(layouts), MoveStruct::Runtime(s)) => {
            let mut cbor_array = vec![];
            for (layout, v) in layouts.iter().zip(s) {
                let cbor_value = serialize_move_value_to_cbor_value(layout, v)?;
                cbor_array.push(cbor_value);
            }
            CborValue::Array(cbor_array)
        }
        (L::WithFields(layout_fields), MoveStruct::WithFields(value_fields)) => {
            serialize_move_fields_to_cbor_value(layout_fields, value_fields)?
        }
        (
            L::WithTypes {
                type_: struct_type,
                fields: layout_fields,
            },
            MoveStruct::WithTypes {
                type_: _,
                fields: value_fields,
            },
        ) => {
            if struct_type.is_ascii_string(&MOVE_STD_ADDRESS) {
                let bytes_field = value_fields
                    .first()
                    .ok_or_else(|| anyhow::anyhow!("Invalid bytes field"))?;

                match bytes_field.1.clone() {
                    MoveValue::Vector(vec) => {
                        let cbor_bytes = MoveValue::vec_to_vec_u8(vec)?;
                        let cbor_text = String::from_utf8(cbor_bytes)
                            .ok()
                            .ok_or_else(|| anyhow::anyhow!("Invalid utf8 String"))?;
                        CborValue::Text(cbor_text)
                    }
                    _ => return Err(anyhow::anyhow!("Invalid ascii string")),
                }
            } else if struct_type.is_std_string(&MOVE_STD_ADDRESS) {
                let bytes_field = value_fields
                    .first()
                    .ok_or_else(|| anyhow::anyhow!("Invalid bytes field"))?;

                match bytes_field.1.clone() {
                    MoveValue::Vector(vec) => {
                        let cbor_bytes = MoveValue::vec_to_vec_u8(vec)?;
                        let cbor_text = String::from_utf8(cbor_bytes)
                            .ok()
                            .ok_or_else(|| anyhow::anyhow!("Invalid utf8 String"))?;
                        CborValue::Text(cbor_text)
                    }
                    _ => return Err(anyhow::anyhow!("Invalid std string")),
                }
            } else if is_std_option(struct_type, &MOVE_STD_ADDRESS) {
                let vec_layout = layout_fields
                    .first()
                    .ok_or_else(|| anyhow::anyhow!("Invalid std option layout"))?;
                let vec_field = value_fields
                    .first()
                    .ok_or_else(|| anyhow::anyhow!("Invalid std option field"))?;

                match (vec_layout.clone().layout, vec_field.1.clone()) {
                    (MoveTypeLayout::Vector(vec_layout), MoveValue::Vector(vec)) => {
                        let item_layout = vec_layout.as_ref();

                        if !vec.is_empty() {
                            serialize_move_value_to_cbor_value(item_layout, vec.first().unwrap())?
                        } else {
                            CborValue::Null
                        }
                    }
                    _ => return Err(anyhow::anyhow!("Invalid std option")),
                }
            } else {
                serialize_move_fields_to_cbor_value(layout_fields, value_fields)?
            }
        }
        _ => {
            return Err(anyhow::anyhow!(
                "Invalid combination of MoveStructLayout and MoveStruct"
            ))
        }
    };

    Ok(value)
}

fn is_std_option(struck_tag: &StructTag, move_std_addr: &AccountAddress) -> bool {
    struck_tag.address == *move_std_addr
        && struck_tag.module.as_str().eq("option")
        && struck_tag.name.as_str().eq("Option")
}

fn serialize_move_fields_to_cbor_value(
    layout_fields: &[MoveFieldLayout],
    value_fields: &Vec<(Identifier, move_core_types::value::MoveValue)>,
) -> Result<CborValue> {
    let mut fields = Vec::new();

    for (field_layout, (name, value)) in layout_fields.iter().zip(value_fields) {
        let cbor_value = serialize_move_value_to_cbor_value(&field_layout.layout, value)?;
        let values = (CborValue::Text(name.clone().into_string()), cbor_value);
        fields.push(values);
    }

    Ok(CborValue::Map(fields))
}

#[derive(Debug, Clone)]
pub struct FromBytesGasParameters {
    pub base: InternalGas,
    pub per_byte_in_str: InternalGasPerByte,
}

impl FromBytesGasParameters {
    pub fn zeros() -> Self {
        Self {
            base: 0.into(),
            per_byte_in_str: 0.into(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct ToBytesGasParameters {
    pub base: InternalGas,
    pub per_byte_in_str: InternalGasPerByte,
}

impl ToBytesGasParameters {
    pub fn zeros() -> Self {
        Self {
            base: 0.into(),
            per_byte_in_str: 0.into(),
        }
    }
}

/// Rust implementation of Move's `native fun native_from_cbor<T>(bytes: vector<u8>): Option<T>` in cbor module
///
/// This function deserializes a vector of CBOR bytes into a Move value of type `T`.
///
/// Arguments:
///
/// * `gas_params` - The `FromBytesGasParameters` struct containing gas parameters for the operation.
/// * `context` - The `NativeContext` providing access to the Move VM runtime.
/// * `ty_args` - A vector of `Type` representing the type arguments for the deserialization.
/// * `args` - A `VecDeque` of `MoveValue` containing the arguments for the function.
///
/// Returns:
///
/// A `PartialVMResult` containing a `NativeResult` with the deserialized Move value wrapped in an `Option`.
/// If the input type is not a struct, an error with the code `E_TYPE_NOT_MATCH` is returned.
///
/// The gas cost for this operation is calculated based on the provided `gas_params`.
#[inline]
fn native_from_cbor(
    gas_params: &FromBytesGasParameters,
    context: &mut NativeContext,
    ty_args: Vec<Type>,
    mut args: VecDeque<MoveValue>,
) -> PartialVMResult<NativeResult> {
    debug_assert_eq!(ty_args.len(), 1);
    debug_assert_eq!(args.len(), 1);

    let mut cost = gas_params.base;
    let type_param = &ty_args[0];
    let layout = context
        .type_to_fully_annotated_layout(type_param)?
        .ok_or_else(|| {
            PartialVMError::new(StatusCode::UNKNOWN_INVARIANT_VIOLATION_ERROR).with_message(
                format!(
                    "Failed to get layout of type {:?} -- this should not happen",
                    ty_args[0]
                ),
            )
        })?;

    let bytes = pop_arg!(args, Vec<u8>);
    cost += gas_params.per_byte_in_str * NumBytes::new(bytes.len() as u64);

    let result = match parse_move_value_from_cbor(&layout, bytes, context) {
        Ok(val) => {
            // Pack the MoveOption Some
            Struct::pack(vec![Vector::pack(type_param, vec![val]).map_err(|e| {
                PartialVMError::new(StatusCode::UNKNOWN_INVARIANT_VIOLATION_ERROR)
                    .with_message(format!("Failed to pack Option: {:?}", e))
            })?])
        }
        Err(e) => {
            debug!("Failed to parse struct_value: {:?}", e);
            // Pack the MoveOption None
            Struct::pack(vec![Vector::pack(type_param, vec![]).map_err(|e| {
                PartialVMError::new(StatusCode::UNKNOWN_INVARIANT_VIOLATION_ERROR)
                    .with_message(format!("Failed to pack Option: {:?}", e))
            })?])
        }
    };

    Ok(NativeResult::ok(
        cost,
        smallvec![MoveValue::struct_(result)],
    ))
}

/// Rust implementation of Move's `native fun native_to_cbor<T>(value: T): vector<u8>` in cbor module
///
/// This function serializes a Move value of type `T` into a vector of CBOR bytes.
///
/// Arguments:
///
/// * `gas_params` - The `ToBytesGasParameters` struct containing gas parameters for the operation.
/// * `context` - The `NativeContext` providing access to the Move VM runtime.
/// * `ty_args` - A vector of `Type` representing the type arguments for the serialization.
/// * `args` - A `VecDeque` of `MoveValue` containing the arguments for the function.
///
/// Returns:
///
/// A `PartialVMResult` containing a `NativeResult` with the serialized CBOR bytes as a `vector<u8>`.
/// If the input type is not a struct, an error with the code `E_TYPE_NOT_MATCH` is returned.
///
/// The gas cost for this operation is calculated based on the provided `gas_params`.
#[inline]
fn native_to_cbor(
    gas_params: &ToBytesGasParameters,
    context: &mut NativeContext,
    mut ty_args: Vec<Type>,
    mut args: VecDeque<MoveValue>,
) -> PartialVMResult<NativeResult> {
    debug_assert_eq!(ty_args.len(), 1);
    debug_assert_eq!(args.len(), 1);

    let mut cost = gas_params.base;

    // pop type and value
    let ref_to_val = pop_arg!(args, Reference);
    let arg_type = ty_args.pop().unwrap();

    // get type layout
    let layout = match context.type_to_type_layout(&arg_type)? {
        Some(layout) => layout,
        None => {
            return Ok(NativeResult::err(cost, E_CBOR_SERIALIZATION_FAILURE));
        }
    };

    let move_val = ref_to_val.read_ref()?.as_move_value(&layout);

    let annotated_layout = match context.type_to_fully_annotated_layout(&arg_type)? {
        Some(layout) => layout,
        None => {
            return Ok(NativeResult::err(cost, E_CBOR_SERIALIZATION_FAILURE));
        }
    };

    let annotated_move_val = move_val.decorate(&annotated_layout);

    let bytes = match serialize_move_value_to_cbor(&annotated_layout, &annotated_move_val) {
        Ok(bytes) => {
            cost += gas_params.per_byte_in_str * NumBytes::new(bytes.len() as u64);
            bytes
        }
        Err(e) => {
            debug!("Failed to serialize value: {:?}", e);
            return Ok(NativeResult::err(
                cost,
                STATUS_CODE_FAILED_TO_SERIALIZE_VALUE,
            ));
        }
    };
    Ok(NativeResult::ok(
        cost,
        smallvec![MoveValue::vector_u8(bytes)],
    ))
}

/***************************************************************************************************
 * module
 **************************************************************************************************/

#[derive(Debug, Clone)]
pub struct GasParameters {
    pub from_bytes: FromBytesGasParameters,
    pub to_bytes: ToBytesGasParameters,
}

impl GasParameters {
    pub fn zeros() -> Self {
        Self {
            from_bytes: FromBytesGasParameters::zeros(),
            to_bytes: ToBytesGasParameters::zeros(),
        }
    }
}

pub fn make_all(gas_params: GasParameters) -> impl Iterator<Item = (String, NativeFunction)> {
    let natives = [
        (
            "native_from_cbor",
            make_native(gas_params.from_bytes, native_from_cbor),
        ),
        (
            "native_to_cbor",
            make_native(gas_params.to_bytes, native_to_cbor),
        ),
    ];

    make_module_natives(natives)
}
