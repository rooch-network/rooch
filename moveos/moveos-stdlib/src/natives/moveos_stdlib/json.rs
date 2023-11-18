// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::natives::helpers::{make_module_natives, make_native};
use anyhow::Result;
use move_binary_format::errors::{PartialVMError, PartialVMResult};
use move_core_types::account_address::AccountAddress;
use move_core_types::language_storage::TypeTag;
use move_core_types::u256::U256;
use move_core_types::value::MoveStructLayout;
use move_core_types::vm_status::StatusCode;
use move_core_types::{gas_algebra::InternalGas, value::MoveTypeLayout};
use move_vm_runtime::native_functions::{NativeContext, NativeFunction};
use move_vm_types::{
    loaded_data::runtime_types::Type,
    natives::function::NativeResult,
    pop_arg,
    values::{Struct, Value, Vector},
};
use serde_json;
use smallvec::smallvec;
use std::collections::VecDeque;
use std::str::FromStr;

const E_TYPE_NOT_MATCH: u64 = 1;
const E_INVALID_JSON_STRING: u64 = 2;

fn parse_struct_value_from_json(
    layout: &MoveStructLayout,
    json_value: &serde_json::Value,
    context: &NativeContext,
) -> Result<Struct> {
    if let MoveStructLayout::WithTypes { fields, .. } = layout {
        let field_values = fields
            .iter()
            .map(|field| -> Result<Value> {
                let name = field.name.as_str();
                let json_field = json_value
                    .get(name)
                    .ok_or_else(|| anyhow::anyhow!("Missing field {}", name))?;
                parse_move_value_from_json(&field.layout, json_field, context)
            })
            .collect::<Result<Vec<Value>>>()?;
        Ok(Struct::pack(field_values))
    } else {
        Err(anyhow::anyhow!("Invalid MoveStructLayout"))
    }
}
fn parse_move_value_from_json(
    layout: &MoveTypeLayout,
    json_value: &serde_json::Value,
    context: &NativeContext,
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
        MoveTypeLayout::Signer => {
            let addr_str = json_value
                .as_str()
                .ok_or_else(|| anyhow::anyhow!("Invalid address value"))?;
            let addr = AccountAddress::from_hex_literal(addr_str)
                .map_err(|_| anyhow::anyhow!("Invalid address value"))?;
            Ok(Value::signer(addr))
        }
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
    }
}

#[derive(Debug, Clone)]
pub struct FromBytesGasParameters {
    pub base: InternalGas,
}

impl FromBytesGasParameters {
    pub fn zeros() -> Self {
        Self { base: 0.into() }
    }
}

/// Rust implementation of Move's `native public(friend) fun from_bytes<T>(vector<u8>): T in bcs module`
/// Bytes are in BCS (Binary Canonical Serialization) format.
#[inline]
fn native_from_json(
    gas_params: &FromBytesGasParameters,
    context: &mut NativeContext,
    ty_args: Vec<Type>,
    mut args: VecDeque<Value>,
) -> PartialVMResult<NativeResult> {
    debug_assert_eq!(ty_args.len(), 1);
    debug_assert_eq!(args.len(), 1);

    let cost = gas_params.base;

    // TODO(Gas): charge for getting the layout
    let layout = context
        .type_to_fully_annotated_layout(&ty_args[0])?
        .ok_or_else(|| {
            PartialVMError::new(StatusCode::UNKNOWN_INVARIANT_VIOLATION_ERROR).with_message(
                format!(
                    "Failed to get layout of type {:?} -- this should not happen",
                    ty_args[0]
                ),
            )
        })?;

    let bytes = pop_arg!(args, Vec<u8>);
    let json_str = match std::str::from_utf8(&bytes) {
        Ok(s) => s,
        Err(_) => {
            return Ok(NativeResult::err(
                cost,
                moveos_types::move_std::error::invalid_argument(E_INVALID_JSON_STRING),
            ));
        }
    };
    let json_obj: serde_json::Value = match serde_json::from_str(json_str) {
        Ok(obj) => obj,
        Err(_) => {
            return Ok(NativeResult::err(
                cost,
                moveos_types::move_std::error::invalid_argument(E_INVALID_JSON_STRING),
            ));
        }
    };

    // If layout is not MoveTypeLayout::MoveStructLayout, return error
    if let MoveTypeLayout::Struct(struct_layout) = layout {
        match parse_struct_value_from_json(&struct_layout, &json_obj, context) {
            Ok(val) => Ok(NativeResult::ok(cost, smallvec![Value::struct_(val)])),
            Err(_) => Ok(NativeResult::err(
                cost,
                moveos_types::move_std::error::invalid_argument(E_INVALID_JSON_STRING),
            )),
        }
    } else {
        Ok(NativeResult::err(
            cost,
            moveos_types::move_std::error::invalid_argument(E_TYPE_NOT_MATCH),
        ))
    }
}

/***************************************************************************************************
 * module
 **************************************************************************************************/

#[derive(Debug, Clone)]
pub struct GasParameters {
    pub from_bytes: FromBytesGasParameters,
}

impl GasParameters {
    pub fn zeros() -> Self {
        Self {
            from_bytes: FromBytesGasParameters::zeros(),
        }
    }
}

pub fn make_all(gas_params: GasParameters) -> impl Iterator<Item = (String, NativeFunction)> {
    let natives = [(
        "native_from_json",
        make_native(gas_params.from_bytes, native_from_json),
    )];

    make_module_natives(natives)
}
