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
use move_core_types::language_storage::TypeTag;
use move_core_types::value::MoveStructLayout;
use move_core_types::value::MoveTypeLayout;
use move_core_types::value::MoveValue as CoreMoveValue;
use move_core_types::value::MoveStruct;
use move_core_types::vm_status::StatusCode;
use move_core_types::gas_algebra::{InternalGas, InternalGasPerByte, NumBytes};
use move_vm_runtime::native_functions::{NativeContext, NativeFunction};
use move_vm_types::{
    loaded_data::runtime_types::Type,
    natives::function::NativeResult,
    pop_arg,
    values::{values_impl::Reference, Struct, Value as MoveValue, Vector},
};
use move_core_types::u256::{self, U256_NUM_BYTES};
use moveos_types::addresses::MOVE_STD_ADDRESS;
use primitive_types::U256 as PrimitiveU256;
use smallvec::smallvec;
use std::collections::VecDeque;
use std::io::Cursor;

const E_TYPE_NOT_MATCH: u64 = 1;
const STATUS_CODE_FAILED_TO_SERIALIZE_VALUE: u64 = 2;

fn parse_struct_value_from_cbor(
    layout: &MoveStructLayout,
    bytes: Vec<u8>,
    context: &NativeContext,
) -> Result<Struct> {
    let cursor = Cursor::new(bytes);
    let cbor_value: CborValue = from_reader(cursor)?;
    parse_struct_value_from_cbor_value(layout, &cbor_value, context)
}

fn parse_struct_value_from_cbor_value(
    layout: &MoveStructLayout,
    cbor_value: &CborValue,
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
                        .iter().find(|x|{x.0.into_text().unwrap()==name})
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
            let u128_value = cbor_value
                .as_integer()
                .and_then(|int| u128::try_from(int).ok())
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
            let u256_bytes = cbor_value
                .as_bytes()
                .ok_or_else(|| anyhow::anyhow!("Invalid u256 value"))?;
            
            let value = PrimitiveU256::from_big_endian(&u256_bytes);
            let mut buffer = [0u8; U256_NUM_BYTES];
            value.to_little_endian(&mut buffer);
            Ok(MoveValue::u256(u256::U256::from_le_bytes(&buffer)))
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
fn serialize_move_value_to_cbor(layout: &MoveTypeLayout, value: &CoreMoveValue) -> Result<Vec<u8>> {
    use MoveTypeLayout as L;
    use CoreMoveValue as MoveValue;

    let mut writer = Vec::new();

    match (layout, value) {
        (L::Struct(layout), MoveValue::Struct(struct_)) => {
            let cbor_value = serialize_move_struct_to_cbor(layout, struct_)?;
            into_writer(&cbor_value, &mut writer)?;
        }
        (L::Bool, MoveValue::Bool(b)) => into_writer( &CborValue::from(*b), &mut writer)?,
        (L::U8, MoveValue::U8(b)) => into_writer( &CborValue::from(*b), &mut writer)?,
        (L::U16, MoveValue::U16(b)) => into_writer( &CborValue::from(*b), &mut writer)?,
        (L::U32, MoveValue::U32(b)) => into_writer( &CborValue::from(*b), &mut writer)?,
        (L::U64, MoveValue::U64(b)) => into_writer( &CborValue::from(*b), &mut writer)?,
        (L::U128, MoveValue::U128(b)) => into_writer( &CborValue::from(*b), &mut writer)?,
        (L::U256, MoveValue::U256(i)) => {
            let slice = i.to_le_bytes();
            let value = PrimitiveU256::from_little_endian(&slice);
            let leading_empty_bytes = value.leading_zeros() as usize / 8;
            let mut buffer = [0u8; U256_NUM_BYTES];
            value.to_big_endian(&mut buffer);
            let bytes = buffer[leading_empty_bytes..].to_vec();
            into_writer(&CborValue::Bytes(bytes), &mut writer)?;
        }
        (L::Address, MoveValue::Address(addr)) => into_writer(&CborValue::from(addr.to_vec()), &mut writer)?,
        (L::Signer, MoveValue::Signer(a)) => into_writer(&CborValue::from(a.to_vec()), &mut writer)?,
        (L::Vector(layout), MoveValue::Vector(vec)) => {
            let mut cbor_vec = Vec::new();
            for item in vec.iter() {
                let bytes = serialize_move_value_to_cbor(layout, item)?;
                let cbor_value = CborValue::from(bytes);
                cbor_vec.push(cbor_value);
            }

            into_writer(&CborValue::Array(cbor_vec), &mut writer)?;
        }
        _ => unreachable!(),
    }

    Ok(writer)
}

fn serialize_move_struct_to_cbor(layout: &MoveStructLayout, struct_: &MoveStruct) -> Result<CborValue> {
    use MoveStructLayout as L;

    let value = match (layout, struct_) {
        (L::Runtime(layouts), MoveStruct::Runtime(s)) => {
            let mut cbor_array = vec![];
            for (layout, v) in layouts.iter().zip(s) {
                let bytes = serialize_move_value_to_cbor(layout, v)?;
                let cbor_value = CborValue::from(bytes);
                cbor_array.push(cbor_value);
            }
            CborValue::Array(cbor_array)
        }
        (L::WithFields(layout_fields), MoveStruct::WithFields(value_fields)) => {
            let mut fields = Vec::new();
        
            for (filed_layout, (name, value)) in layout_fields.iter().zip(value_fields) {
                let cbor_value = serialize_move_value_to_cbor(&filed_layout.layout, value)?;
                let values = (CborValue::Text(name.into_string()), CborValue::Bytes(cbor_value));
                fields.push(values);
            }
        
            CborValue::Map(fields)
        },
        (L::WithTypes {type_: layout_type_, fields: layout_fields}, MoveStruct::WithTypes { type_, fields }) => {
            let mut cbor_fields = Vec::new();

            for (filed_layout, (name, value)) in layout_fields.iter().zip(fields) {
                let cbor_bytes = serialize_move_value_to_cbor(&filed_layout.layout, &value)?;
                let values = (CborValue::Text(name.into_string()), CborValue::Bytes(cbor_bytes));
                cbor_fields.push(values);
            }

            CborValue::Map(cbor_fields)
        },
        _ => return Err(anyhow::anyhow!("Invalid combination of MoveStructLayout and MoveStruct")),
    };

    Ok(value)
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

    let ref_to_val = pop_arg!(args, Reference);

    if let MoveTypeLayout::Struct(struct_layout) = layout {
        let move_val = ref_to_val.read_ref()?.as_move_value(&layout);;
        let bytes = match serialize_move_value_to_cbor(&layout, &move_val) {
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

    make_module_natives(natives)
}
