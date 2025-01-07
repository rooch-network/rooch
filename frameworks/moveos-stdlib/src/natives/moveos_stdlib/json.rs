// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use std::collections::VecDeque;
use std::str::FromStr;

use anyhow::Result;
use primitive_types::U128 as PrimitiveU128;
use primitive_types::U256 as PrimitiveU256;
use serde_json;
use serde_json::Value as JsonValue;
use smallvec::smallvec;
use tracing::debug;

use move_binary_format::errors::{PartialVMError, PartialVMResult};
use move_core_types::account_address::AccountAddress;
use move_core_types::gas_algebra::{InternalGas, InternalGasPerByte, NumBytes};
use move_core_types::identifier::Identifier;
use move_core_types::language_storage::StructTag;
use move_core_types::language_storage::TypeTag;
use move_core_types::u256::U256;
use move_core_types::value::MoveStruct;
use move_core_types::value::MoveValue;
use move_core_types::value::{MoveFieldLayout, MoveStructLayout, MoveTypeLayout};
use move_core_types::vm_status::StatusCode;

use move_vm_runtime::native_functions::{NativeContext, NativeFunction};

use move_vm_types::{
    loaded_data::runtime_types::Type,
    natives::function::NativeResult,
    pop_arg,
    values::{values_impl::Reference, Struct, Value, Vector},
};

use moveos_types::addresses::MOVE_STD_ADDRESS;
use moveos_types::move_std::string::MoveString;
use moveos_types::moveos_std::simple_map::{Element, SimpleMap};
use moveos_types::state::{MoveStructType, MoveType};

use crate::natives::helpers::{make_module_natives, make_native};

const E_TYPE_NOT_MATCH: u64 = 1;
const STATUS_CODE_FAILED_TO_SERIALIZE_VALUE: u64 = 2;
const E_JSON_SERIALIZATION_FAILURE: u64 = 3;

fn parse_struct_value_from_bytes(
    layout: &MoveStructLayout,
    bytes: Vec<u8>,
    context: &mut NativeContext,
) -> Result<Struct> {
    let json_str = std::str::from_utf8(&bytes)?;
    let json_obj: JsonValue = serde_json::from_str(json_str)?;
    parse_struct_value_from_json(layout, &json_obj, context)
}

fn parse_struct_value_from_json(
    layout: &MoveStructLayout,
    json_value: &JsonValue,
    context: &mut NativeContext,
) -> Result<Struct> {
    if let MoveStructLayout::WithTypes {
        type_: struct_type,
        fields,
    } = layout
    {
        if struct_type.is_std_string(&MOVE_STD_ADDRESS) {
            let str_value = json_value
                .as_str()
                .ok_or_else(|| anyhow::anyhow!("Invalid string value"))?;
            Ok(Struct::pack(vec![Value::vector_u8(
                str_value.as_bytes().to_vec(),
            )]))
        } else if struct_type.is_ascii_string(&MOVE_STD_ADDRESS) {
            let str_value = json_value
                .as_str()
                .ok_or_else(|| anyhow::anyhow!("Invalid ascii string value"))?;
            if !str_value.is_ascii() {
                return Err(anyhow::anyhow!("Invalid ascii string value"));
            }
            Ok(Struct::pack(vec![Value::vector_u8(
                str_value.as_bytes().to_vec(),
            )]))
        } else if is_std_option(struct_type, &MOVE_STD_ADDRESS) {
            let vec_layout = fields
                .first()
                .ok_or_else(|| anyhow::anyhow!("Invalid std option layout"))?;
            let type_tag: TypeTag = (&vec_layout.layout).try_into()?;
            let ty = context.load_type(&type_tag)?;

            if json_value.is_null() {
                let value = Vector::pack(&ty, vec![])?;
                return Ok(Struct::pack(vec![value]));
            }

            if let MoveTypeLayout::Vector(vec_layout) = vec_layout.layout.clone() {
                let struct_layout = vec_layout.as_ref();
                let move_struct_value =
                    parse_move_value_from_json(struct_layout, json_value, context)?;
                let value = Vector::pack(&ty, vec![move_struct_value])?;
                return Ok(Struct::pack(vec![value]));
            }

            Err(anyhow::anyhow!("Invalid std option layout"))
        } else if struct_type == &SimpleMap::<MoveString, MoveString>::struct_tag() {
            let key_value_pairs = json_obj_to_key_value_pairs(json_value)?;
            let mut key_values = Vec::with_capacity(key_value_pairs.len());
            for (key, value) in key_value_pairs {
                key_values.push(Value::struct_(Struct::pack(vec![
                    Value::struct_(Struct::pack(vec![Value::vector_u8(
                        key.as_bytes().to_vec(),
                    )])),
                    Value::struct_(Struct::pack(vec![Value::vector_u8(
                        value.as_bytes().to_vec(),
                    )])),
                ])));
            }
            let element_type = context.load_type(&Element::<MoveString, MoveString>::type_tag())?;
            Ok(Struct::pack(vec![Vector::pack(&element_type, key_values)?]))
        } else {
            let field_values = fields
                .iter()
                .map(|field| -> Result<Value> {
                    let name = field.name.as_str();
                    let json_field = match json_value.get(name) {
                        Some(value) => value,
                        None => {
                            if let MoveTypeLayout::Struct(_) | MoveTypeLayout::Vector(_) =
                                field.layout
                            {
                                &JsonValue::Null
                            } else {
                                return Err(anyhow::anyhow!(
                                    "type: {}, Missing field {}",
                                    struct_type,
                                    name
                                ));
                            }
                        }
                    };
                    parse_move_value_from_json(&field.layout, json_field, context)
                })
                .collect::<Result<Vec<Value>>>()?;
            Ok(Struct::pack(field_values))
        }
    } else {
        Err(anyhow::anyhow!("Invalid MoveStructLayout"))
    }
}
fn parse_move_value_from_json(
    layout: &MoveTypeLayout,
    json_value: &JsonValue,
    context: &mut NativeContext,
) -> Result<Value> {
    match layout {
        MoveTypeLayout::Bool => {
            let bool_value = json_value
                .as_bool()
                .ok_or_else(|| anyhow::anyhow!("Invalid bool value"))?;
            Ok(Value::bool(bool_value))
        }
        MoveTypeLayout::U8 => {
            let u64_value = json_value
                .as_u64()
                .ok_or_else(|| anyhow::anyhow!("Invalid u8 value"))?;
            if u64_value > (u8::MAX as u64) {
                return Err(anyhow::anyhow!("Invalid u8 value"));
            }
            Ok(Value::u8(u64_value as u8))
        }
        MoveTypeLayout::U64 => {
            let u64_value = json_value
                .as_u64()
                .ok_or_else(|| anyhow::anyhow!("Invalid u64 value"))?;
            Ok(Value::u64(u64_value))
        }
        MoveTypeLayout::U128 => {
            let u128_value = json_value
                .as_str()
                .ok_or_else(|| anyhow::anyhow!("Invalid u128 value"))?
                .parse::<u128>()?;
            Ok(Value::u128(u128_value))
        }
        MoveTypeLayout::Address => {
            let addr_str = json_value
                .as_str()
                .ok_or_else(|| anyhow::anyhow!("Invalid address value"))?;
            let addr = AccountAddress::from_hex_literal(addr_str)
                .map_err(|_| anyhow::anyhow!("Invalid address value"))?;
            Ok(Value::address(addr))
        }
        MoveTypeLayout::Vector(item_layout) => {
            if json_value.is_null() {
                let type_tag: TypeTag = (&**item_layout).try_into()?;
                let ty = context.load_type(&type_tag)?;
                return Ok(Vector::pack(&ty, vec![])?);
            }

            let vec_value = json_value
                .as_array()
                .ok_or_else(|| anyhow::anyhow!("Invalid vector value"))?
                .iter()
                .map(|v| parse_move_value_from_json(item_layout, v, context))
                .collect::<Result<Vec<_>>>()?;
            let type_tag: TypeTag = (&**item_layout).try_into()?;
            let ty = context.load_type(&type_tag)?;
            let value = Vector::pack(&ty, vec_value)?;
            Ok(value)
        }
        MoveTypeLayout::Struct(struct_layout) => {
            let struct_value = parse_struct_value_from_json(struct_layout, json_value, context)?;
            Ok(Value::struct_(struct_value))
        }
        MoveTypeLayout::Signer => Err(anyhow::anyhow!("Do not support Signer type")),
        MoveTypeLayout::U16 => {
            let u64_value = json_value
                .as_u64()
                .ok_or_else(|| anyhow::anyhow!("Invalid u16 value"))?;
            if u64_value > (u16::MAX as u64) {
                return Err(anyhow::anyhow!("Invalid u16 value"));
            }
            Ok(Value::u16(u64_value as u16))
        }
        MoveTypeLayout::U32 => {
            let u64_value = json_value
                .as_u64()
                .ok_or_else(|| anyhow::anyhow!("Invalid u32 value"))?;
            if u64_value > (u32::MAX as u64) {
                return Err(anyhow::anyhow!("Invalid u32 value"));
            }
            Ok(Value::u32(u64_value as u32))
        }
        MoveTypeLayout::U256 => {
            let u256_str = json_value
                .as_str()
                .ok_or_else(|| anyhow::anyhow!("Invalid u256 value"))?;
            let u256_value =
                U256::from_str(u256_str).map_err(|_| anyhow::anyhow!("Invalid u256 value"))?;
            Ok(Value::u256(u256_value))
        }
        _ => Err(anyhow::anyhow!("Invalid MoveTypeLayout")),
    }
}

fn json_obj_to_key_value_pairs(json_obj: &JsonValue) -> Result<Vec<(String, String)>> {
    if let JsonValue::Object(obj) = json_obj {
        let mut key_value_pairs = Vec::with_capacity(obj.len());
        for (key, value) in obj.iter() {
            let key = key.to_string();
            let value = match value {
                JsonValue::String(s) => s.to_string(),
                JsonValue::Number(n) => n.to_string(),
                JsonValue::Bool(b) => b.to_string(),
                JsonValue::Null => "null".to_string(),
                //convert array and object to string
                value => value.to_string(),
            };
            key_value_pairs.push((key, value));
        }
        Ok(key_value_pairs)
    } else {
        Err(anyhow::anyhow!("Invalid json object"))
    }
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

/// Rust implementation of Move's `native fun native_from_json<T>(json_str: vector<u8>): T` in json module
/// Input arguments:
///   - json_str: vector<u8>, string bytes of json object
#[inline]
fn native_from_json(
    gas_params: &FromBytesGasParameters,
    context: &mut NativeContext,
    ty_args: Vec<Type>,
    mut args: VecDeque<Value>,
) -> PartialVMResult<NativeResult> {
    debug_assert_eq!(ty_args.len(), 1);
    debug_assert_eq!(args.len(), 1);

    let mut cost = gas_params.base;
    let type_param = &ty_args[0];
    // TODO(Gas): charge for getting the layout
    let layout = match context.type_to_fully_annotated_layout(type_param) {
        Ok(layout) => layout,
        Err(_) => {
            return Err(
                PartialVMError::new(StatusCode::UNKNOWN_INVARIANT_VIOLATION_ERROR).with_message(
                    format!(
                        "Failed to get layout of type {:?} -- this should not happen",
                        ty_args[0]
                    ),
                ),
            )
        }
    };

    let bytes = pop_arg!(args, Vec<u8>);
    cost += gas_params.per_byte_in_str * NumBytes::new(bytes.len() as u64);

    // If layout is not MoveTypeLayout::MoveStructLayout, return error
    if let MoveTypeLayout::Struct(struct_layout) = layout {
        let result = match parse_struct_value_from_bytes(&struct_layout, bytes, context) {
            Ok(val) => {
                //Pack the MoveOption Some
                Struct::pack(vec![Vector::pack(type_param, vec![Value::struct_(val)])
                    .map_err(|e| {
                        PartialVMError::new(StatusCode::UNKNOWN_INVARIANT_VIOLATION_ERROR)
                            .with_message(format!("Failed to pack MoveOption: {:?}", e))
                    })?])
            }
            Err(e) => {
                debug!("Failed to parse struct_value: {:?}", e);
                //Pack the MoveOption None
                Struct::pack(vec![Vector::pack(type_param, vec![]).map_err(|e| {
                    PartialVMError::new(StatusCode::UNKNOWN_INVARIANT_VIOLATION_ERROR)
                        .with_message(format!("Failed to pack MoveOption: {:?}", e))
                })?])
            }
        };
        Ok(NativeResult::ok(cost, smallvec![Value::struct_(result)]))
    } else {
        Ok(NativeResult::err(cost, E_TYPE_NOT_MATCH))
    }
}

#[derive(Debug, Clone)]
pub struct ToBytesGasParametersOption {
    pub base: Option<InternalGas>,
    pub per_byte_in_str: Option<InternalGasPerByte>,
}

impl ToBytesGasParametersOption {
    pub fn zeros() -> Self {
        Self {
            base: Some(0.into()),
            per_byte_in_str: Some(0.into()),
        }
    }
}

impl ToBytesGasParametersOption {
    pub fn is_empty(&self) -> bool {
        self.base.is_none() || self.per_byte_in_str.is_none()
    }
}

fn serialize_move_value_to_json(layout: &MoveTypeLayout, value: &MoveValue) -> Result<JsonValue> {
    use MoveTypeLayout as L;

    let json_value = match (layout, value) {
        (L::Struct(layout), MoveValue::Struct(struct_)) => {
            serialize_move_struct_to_json(layout, struct_)?
        }
        (L::Bool, MoveValue::Bool(b)) => JsonValue::Bool(*b),
        (L::U8, MoveValue::U8(b)) => JsonValue::Number((*b).into()),
        (L::U16, MoveValue::U16(b)) => JsonValue::Number((*b).into()),
        (L::U32, MoveValue::U32(b)) => JsonValue::Number((*b).into()),
        (L::U64, MoveValue::U64(b)) => JsonValue::Number((*b).into()),
        (L::U128, MoveValue::U128(i)) => {
            let slice = i.to_le_bytes();
            let value = PrimitiveU128::from_little_endian(&slice);
            JsonValue::String(value.to_string())
        }
        (L::U256, MoveValue::U256(i)) => {
            let slice = i.to_le_bytes();
            let value = PrimitiveU256::from_little_endian(&slice);
            JsonValue::String(value.to_string())
        }
        (L::Address, MoveValue::Address(addr)) => JsonValue::String(addr.to_hex_literal()),
        (L::Signer, MoveValue::Signer(_a)) => {
            return Err(anyhow::anyhow!("Do not support Signer type"))
        }
        (L::Vector(vec_layout), MoveValue::Vector(vec)) => {
            let layout = vec_layout.as_ref();

            if let L::U8 = layout {
                let mut json_vec = Vec::new();

                for item in vec.iter() {
                    if let MoveValue::U8(b) = item {
                        json_vec.push(JsonValue::Number((*b).into()));
                    }
                }

                JsonValue::Array(json_vec)
            } else {
                let mut json_vec = Vec::with_capacity(vec.len());

                for item in vec.iter() {
                    let json_value = serialize_move_value_to_json(layout, item)?;
                    json_vec.push(json_value);
                }

                JsonValue::Array(json_vec)
            }
        }
        _ => {
            return Err(anyhow::anyhow!(
                "Invalid combination of MoveStructLayout and MoveStruct"
            ))
        }
    };

    Ok(json_value)
}

fn serialize_move_struct_to_json(
    layout: &MoveStructLayout,
    struct_: &MoveStruct,
) -> Result<JsonValue> {
    use MoveStructLayout as L;

    let value = match (layout, struct_) {
        (L::Runtime(layouts), MoveStruct::Runtime(s)) => {
            let mut json_array = Vec::new();
            for (layout, v) in layouts.iter().zip(s) {
                let json_value = serialize_move_value_to_json(layout, v)?;
                json_array.push(json_value);
            }
            JsonValue::Array(json_array)
        }
        (L::WithFields(layout_fields), MoveStruct::WithFields(value_fields)) => {
            serialize_move_fields_to_json(layout_fields, value_fields)?
        }
        (
            L::WithTypes {
                type_: struct_type,
                fields: layout_fields,
            },
            MoveStruct::WithTypes {
                _type_: _,
                _fields: value_fields,
            },
        ) => {
            if struct_type.is_ascii_string(&MOVE_STD_ADDRESS)
                || struct_type.is_std_string(&MOVE_STD_ADDRESS)
            {
                let bytes_field = value_fields
                    .first()
                    .ok_or_else(|| anyhow::anyhow!("Invalid bytes field"))?;

                match &bytes_field.1 {
                    MoveValue::Vector(vec) => {
                        let bytes = MoveValue::vec_to_vec_u8(vec.clone())?;
                        let string = String::from_utf8(bytes)
                            .map_err(|_| anyhow::anyhow!("Invalid utf8 String"))?;
                        JsonValue::String(string)
                    }
                    _ => return Err(anyhow::anyhow!("Invalid string")),
                }
            } else if is_std_option(struct_type, &MOVE_STD_ADDRESS) {
                let vec_layout = layout_fields
                    .first()
                    .ok_or_else(|| anyhow::anyhow!("Invalid std option layout"))?;
                let vec_field = value_fields
                    .first()
                    .ok_or_else(|| anyhow::anyhow!("Invalid std option field"))?;

                match (&vec_layout.layout, &vec_field.1) {
                    (MoveTypeLayout::Vector(vec_layout), MoveValue::Vector(vec)) => {
                        let item_layout = vec_layout.as_ref();

                        if !vec.is_empty() {
                            serialize_move_value_to_json(item_layout, vec.first().unwrap())?
                        } else {
                            JsonValue::Null
                        }
                    }
                    _ => return Err(anyhow::anyhow!("Invalid std option")),
                }
            } else if struct_type == &SimpleMap::<MoveString, Vec<u8>>::struct_tag() {
                let data_field = value_fields
                    .iter()
                    .find(|(name, _)| name.as_str() == "data")
                    .ok_or_else(|| anyhow::anyhow!("Missing data field in SimpleMap"))?;

                let data_vector = match &data_field.1 {
                    MoveValue::Vector(vec) => vec,
                    _ => return Err(anyhow::anyhow!("Invalid data field in SimpleMap")),
                };

                let key_value_pairs = data_vector
                    .iter()
                    .map(|element| {
                        let struct_ = match element {
                            MoveValue::Struct(s) => s,
                            _ => return Err(anyhow::anyhow!("Invalid element in SimpleMap data")),
                        };

                        let fields = match struct_ {
                            MoveStruct::WithTypes {
                                _type_: _,
                                _fields: value_fields,
                            } => value_fields,
                            _ => return Err(anyhow::anyhow!("Invalid element in SimpleMap data")),
                        };

                        let key = match &fields[0].1 {
                            MoveValue::Struct(struct_) => {
                                let value_fields = match struct_ {
                                    MoveStruct::WithTypes {
                                        _type_: _,
                                        _fields: value_fields,
                                    } => value_fields,
                                    _ => {
                                        return Err(anyhow::anyhow!(
                                            "Invalid element in SimpleMap data"
                                        ))
                                    }
                                };

                                let bytes_field = value_fields
                                    .first()
                                    .ok_or_else(|| anyhow::anyhow!("Invalid bytes field"))?;

                                match bytes_field.1.clone() {
                                    MoveValue::Vector(vec) => {
                                        let bytes = MoveValue::vec_to_vec_u8(vec)?;
                                        String::from_utf8(bytes)
                                            .map_err(|_| anyhow::anyhow!("Invalid utf8 String"))?
                                    }
                                    _ => return Err(anyhow::anyhow!("Invalid std string")),
                                }
                            }
                            _ => return Err(anyhow::anyhow!("Invalid key in SimpleMap")),
                        };

                        let json_value = match &fields[1].1 {
                            MoveValue::Vector(vec) => {
                                let bytes = MoveValue::vec_to_vec_u8(vec.clone())?;
                                let json_value: JsonValue = serde_json::from_slice(&bytes)
                                    .map_err(|_| {
                                        anyhow::anyhow!("Invalid JSON value in SimpleMap")
                                    })?;
                                json_value
                            }
                            _ => return Err(anyhow::anyhow!("Invalid value in SimpleMap")),
                        };

                        Ok((key, json_value))
                    })
                    .collect::<Result<Vec<_>>>()?;

                JsonValue::Object(key_value_pairs.into_iter().collect())
            } else {
                serialize_move_fields_to_json(layout_fields, value_fields)?
            }
        }
        _ => {
            debug!(
                "Invalid combination of MoveStructLayout and MoveStruct, layout:{:?}, struct:{:?}",
                layout, struct_
            );

            return Err(anyhow::anyhow!(
                "Invalid combination of MoveStructLayout and MoveStruct"
            ));
        }
    };

    Ok(value)
}

fn is_std_option(struct_tag: &StructTag, move_std_addr: &AccountAddress) -> bool {
    struct_tag.address == *move_std_addr
        && struct_tag.module.as_str().eq("option")
        && struct_tag.name.as_str().eq("Option")
}

fn serialize_move_fields_to_json(
    layout_fields: &[MoveFieldLayout],
    value_fields: &Vec<(Identifier, MoveValue)>,
) -> Result<JsonValue> {
    let mut fields = serde_json::Map::new();

    for (field_layout, (name, value)) in layout_fields.iter().zip(value_fields) {
        let json_value = serialize_move_value_to_json(&field_layout.layout, value)?;

        if !json_value.is_null() {
            fields.insert(name.clone().into_string(), json_value);
        }
    }

    Ok(JsonValue::Object(fields))
}

#[inline]
fn native_to_json(
    gas_params: &ToBytesGasParametersOption,
    context: &mut NativeContext,
    mut ty_args: Vec<Type>,
    mut args: VecDeque<Value>,
) -> PartialVMResult<NativeResult> {
    debug_assert_eq!(ty_args.len(), 1);
    debug_assert_eq!(args.len(), 1);

    let gas_base = gas_params.base.expect("base gas is missing");
    let per_byte_in_str = gas_params
        .per_byte_in_str
        .expect("per byte in str gas is missing");

    let mut cost = gas_base;

    // pop type and value
    let ref_to_val = pop_arg!(args, Reference);
    let arg_type = ty_args.pop().unwrap();

    // get type layout
    let layout = match context.type_to_type_layout(&arg_type) {
        Ok(layout) => layout,
        Err(_) => {
            return Ok(NativeResult::err(cost, E_JSON_SERIALIZATION_FAILURE));
        }
    };

    let move_val = ref_to_val.read_ref()?.as_move_value(&layout);

    let annotated_layout = match context.type_to_fully_annotated_layout(&arg_type) {
        Ok(layout) => layout,
        Err(_) => {
            return Ok(NativeResult::err(cost, E_JSON_SERIALIZATION_FAILURE));
        }
    };

    let annotated_move_val = move_val.decorate(&annotated_layout);

    match serialize_move_value_to_json(&annotated_layout, &annotated_move_val) {
        Ok(json_value) => {
            let json_string = json_value.to_string();
            cost += per_byte_in_str * NumBytes::new(json_string.len() as u64);

            Ok(NativeResult::ok(
                cost,
                smallvec![Value::vector_u8(json_string.into_bytes())],
            ))
        }
        Err(e) => {
            debug!("Failed to serialize value: {:?}", e);

            Ok(NativeResult::err(
                cost,
                STATUS_CODE_FAILED_TO_SERIALIZE_VALUE,
            ))
        }
    }
}

/***************************************************************************************************
 * module
 **************************************************************************************************/

#[derive(Debug, Clone)]
pub struct GasParameters {
    pub from_bytes: FromBytesGasParameters,
    pub to_bytes: ToBytesGasParametersOption,
}

impl GasParameters {
    pub fn zeros() -> Self {
        Self {
            from_bytes: FromBytesGasParameters::zeros(),
            to_bytes: ToBytesGasParametersOption::zeros(),
        }
    }
}

pub fn make_all(gas_params: GasParameters) -> impl Iterator<Item = (String, NativeFunction)> {
    let mut natives = [(
        "native_from_json",
        make_native(gas_params.from_bytes, native_from_json),
    )]
    .to_vec();

    if !gas_params.to_bytes.is_empty() {
        natives.push((
            "native_to_json",
            make_native(gas_params.to_bytes, native_to_json),
        ));
    }

    make_module_natives(natives)
}
