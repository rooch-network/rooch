// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::natives::helpers::{make_module_natives, make_native};
use anyhow::Result;
use ciborium::de::from_reader;
use ciborium::ser::into_writer;
use ciborium::value::Value;
use log::debug;
use move_binary_format::errors::{PartialVMError, PartialVMResult};
use move_core_types::account_address::AccountAddress;
use move_core_types::language_storage::TypeTag;
use move_core_types::value::MoveStructLayout;
use move_core_types::vm_status::StatusCode;
use move_core_types::{
    gas_algebra::{InternalGas, InternalGasPerByte, NumBytes},
    value::MoveTypeLayout,
};
use move_vm_runtime::native_functions::{NativeContext, NativeFunction};
use move_vm_types::{
    loaded_data::runtime_types::Type,
    natives::function::NativeResult,
    pop_arg,
    values::{Struct, Value as MoveValue, Vector},
};
use smallvec::smallvec;
use std::collections::VecDeque;
use std::io::Cursor;

const E_TYPE_NOT_MATCH: u64 = 1;

fn parse_struct_value_from_cbor(
    layout: &MoveStructLayout,
    bytes: Vec<u8>,
    context: &NativeContext,
) -> Result<Struct> {
    let cursor = Cursor::new(bytes);
    let cbor_value: Value = from_reader(cursor)?;
    parse_struct_value_from_cbor_value(layout, &cbor_value, context)
}

fn parse_struct_value_from_cbor_value(
    layout: &MoveStructLayout,
    cbor_value: &Value,
    context: &NativeContext,
) -> Result<Struct> {
    if let MoveStructLayout::WithTypes {
        type_: struct_type,
        fields,
    } = layout
    {
        if struct_type.is_std_string(&MOVE_STD_ADDRESS) {
            let str_value = cbor_value
                .as_bytes()
                .ok_or_else(|| anyhow::anyhow!("Invalid string value"))?;
            Ok(Struct::pack(vec![MoveValue::vector_u8(
                str_value.to_vec(),
            )]))
        } else if struct_type.is_ascii_string(&MOVE_STD_ADDRESS) {
            let str_value = cbor_value
                .as_bytes()
                .ok_or_else(|| anyhow::anyhow!("Invalid ascii string value"))?;
            if !str_value.iter().all(|&b| b.is_ascii()) {
                return Err(anyhow::anyhow!("Invalid ascii string value"));
            }
            Ok(Struct::pack(vec![MoveValue::vector_u8(
                str_value.to_vec(),
            )]))
        } else {
            let field_values = fields
                .iter()
                .map(|field| -> Result<MoveValue> {
                    let name = field.name.as_str();
                    let cbor_field = cbor_value
                        .as_map()
                        .ok_or_else(|| {
                            anyhow::anyhow!("type: {}, Expected a map value", struct_type)
                        })?
                        .get(name)
                        .ok_or_else(|| {
                            anyhow::anyhow!("type: {}, Missing field {}", struct_type, name)
                        })?;
                    parse_move_value_from_cbor_value(&field.layout, cbor_field, context)
                })
                .collect::<Result<Vec<MoveValue>>>()?;
            Ok(Struct::pack(field_values))
        }
    } else {
        Err(anyhow::anyhow!("Invalid MoveStructLayout"))
    }
}


/// Parse a Move value from a CBOR value based on the provided layout.
///
/// This function takes a `MoveTypeLayout` and a `Value` (from the `ciborium` crate),
/// and recursively parses the CBOR value into the corresponding Move value.
///
/// Arguments:
///
/// * `layout` - The `MoveTypeLayout` describing the expected type of the Move value.
/// * `cbor_value` - The `Value` representing the CBOR value to be parsed.
/// * `context` - The `NativeContext` providing access to the Move VM runtime.
///
/// Returns:
///
/// A `Result` containing the parsed `MoveValue`, or an `anyhow::Error` if the parsing failed.
fn parse_move_value_from_cbor_value(
    layout: &MoveTypeLayout,
    cbor_value: &Value,
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
            let u64_value = cbor_value
                .as_integer()
                .and_then(|int| int.try_into().ok())
                .ok_or_else(|| anyhow::anyhow!("Invalid u8 value"))?;
            if u64_value > (u8::MAX as u64) {
                return Err(anyhow::anyhow!("Invalid u8 value"));
            }
            Ok(MoveValue::u8(u64_value as u8))
        }
        // Parse an unsigned 64-bit integer
        MoveTypeLayout::U64 => {
            let u64_value = cbor_value
                .as_integer()
                .and_then(|int| int.try_into().ok())
                .ok_or_else(|| anyhow::anyhow!("Invalid u64 value"))?;
            Ok(MoveValue::u64(u64_value))
        }
        // Parse an unsigned 128-bit integer
        MoveTypeLayout::U128 => {
            let u128_value = cbor_value
                .as_integer()
                .and_then(|int| int.try_into().ok())
                .ok_or_else(|| anyhow::anyhow!("Invalid u128 value"))?;
            Ok(MoveValue::u128(u128_value))
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
            let u64_value = cbor_value
                .as_integer()
                .and_then(|int| int.try_into().ok())
                .ok_or_else(|| anyhow::anyhow!("Invalid u16 value"))?;
            if u64_value > (u16::MAX as u64) {
                return Err(anyhow::anyhow!("Invalid u16 value"));
            }
            Ok(MoveValue::u16(u64_value as u16))
        }
        // Parse an unsigned 32-bit integer
        MoveTypeLayout::U32 => {
            let u64_value = cbor_value
                .as_integer()
                .and_then(|int| int.try_into().ok())
                .ok_or_else(|| anyhow::anyhow!("Invalid u32 value"))?;
            if u64_value > (u32::MAX as u64) {
                return Err(anyhow::anyhow!("Invalid u32 value"));
            }
            Ok(MoveValue::u32(u64_value as u32))
        }
        // Parse an unsigned 256-bit integer
        MoveTypeLayout::U256 => {
            let u256_bytes = cbor_value
                .as_bytes()
                .ok_or_else(|| anyhow::anyhow!("Invalid u256 value"))?;
            let u256_value = move_core_types::u256::U256::from_bytes_big_endian(u256_bytes)
                .map_err(|_| anyhow::anyhow!("Invalid u256 value"))?;
            Ok(MoveValue::u256(u256_value))
        }
    }
}

/// Serialize a Move value to CBOR bytes.
///
/// This function takes a `MoveValue` and serializes it to a vector of CBOR bytes.
///
/// Arguments:
///
/// * `value` - The `MoveValue` to be serialized.
///
/// Returns:
///
/// A `Result` containing the serialized CBOR bytes as a `Vec<u8>`, or an `anyhow::Error` if the serialization failed.
fn serialize_move_value_to_cbor(value: &MoveValue) -> Result<Vec<u8>> {
    let mut writer = Vec::new();
    match value {
        // Serialize a boolean value
        MoveValue::Bool(b) => {
            into_writer(&mut writer, &Value::Bool(*b))?;
        }
        // Serialize an unsigned 8-bit integer
        MoveValue::U8(u) => {
            into_writer(&mut writer, &Value::U64(*u as u64))?;
        }
        // Serialize an unsigned 64-bit integer
        MoveValue::U64(u) => {
            into_writer(&mut writer, &Value::U64(*u))?;
        }
        // Serialize an unsigned 128-bit integer
        MoveValue::U128(u) => {
            into_writer(&mut writer, &Value::U128(*u))?;
        }
        // Serialize an address value
        MoveValue::Address(addr) => {
            into_writer(&mut writer, &Value::Bytes(addr.to_vec()))?;
        }
        // Serialize a vector value
        MoveValue::Vector(vec) => {
            let mut cbor_vec = Vec::new();
            for item in vec.iter() {
                cbor_vec.push(serialize_move_value_to_cbor(item)?);
            }
            into_writer(&mut writer, &Value::Array(cbor_vec))?;
        }
        // Serialize a struct value
        MoveValue::Struct(struct_value) => {
            let fields = struct_value
                .fields
                .iter()
                .map(|(name, value)| {
                    let cbor_value = serialize_move_value_to_cbor(value)?;
                    Ok((name.as_str().to_string(), Value::Bytes(cbor_value)))
                })
                .collect::<Result<Vec<_>>>()?;
            into_writer(&mut writer, &Value::Map(fields))?;
        }
        // Serialize an unsigned 16-bit integer
        MoveValue::U16(u) => {
            into_writer(&mut writer, &Value::U64(*u as u64))?;
        }
        // Serialize an unsigned 32-bit integer
        MoveValue::U32(u) => {
            into_writer(&mut writer, &Value::U64(*u as u64))?;
        }
        // Serialize an unsigned 256-bit integer
        MoveValue::U256(u) => {
            into_writer(&mut writer, &Value::Bytes(u.as_bytes_big_endian()))?;
        }
        // Unsupported Move value type
        _ => return Err(anyhow::anyhow!("Unsupported Move value type")),
    }
    Ok(writer)
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

    if let MoveTypeLayout::Struct(struct_layout) = layout {
        let result = match parse_struct_value_from_cbor(&struct_layout, bytes, context) {
            Ok(val) => {
                // Pack the MoveOption Some
                Struct::pack(vec![Vector::pack(type_param, vec![MoveValue::struct_(val)])
                    .map_err(|e| {
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
        Ok(NativeResult::ok(cost, smallvec![MoveValue::struct_(result)]))
    } else {
        Ok(NativeResult::err(cost, E_TYPE_NOT_MATCH))
    }
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

    let value = pop_arg!(args, MoveValue);

    if let MoveTypeLayout::Struct(struct_layout) = layout {
        let bytes = match serialize_move_value_to_cbor(&value) {
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
        Ok(NativeResult::ok(cost, smallvec![MoveValue::vector_u8(bytes)]))
    } else {
        Ok(NativeResult::err(cost, E_TYPE_NOT_MATCH))
    }
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

    make_module_natives(natives);
}
