// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::natives::helpers::{make_module_natives, make_native};
use base64::{engine::general_purpose, Engine as _};
use move_binary_format::errors::PartialVMResult;
use move_core_types::gas_algebra::{InternalGas, InternalGasPerByte, NumBytes};
use move_vm_runtime::native_functions::{NativeContext, NativeFunction};
use move_vm_types::{
    loaded_data::runtime_types::Type,
    natives::function::NativeResult,
    pop_arg,
    values::{Value, VectorRef},
};
use smallvec::smallvec;
use std::collections::VecDeque;

pub const E_DECODE_FAILED: u64 = 1;

/***************************************************************************************************
 * native fun encode
 **************************************************************************************************/
pub fn native_encode(
    gas_params: &EncodeDecodeGasParametersOption,
    _context: &mut NativeContext,
    ty_args: Vec<Type>,
    mut args: VecDeque<Value>,
) -> PartialVMResult<NativeResult> {
    debug_assert!(ty_args.is_empty());
    debug_assert!(args.len() == 1);

    let input = pop_arg!(args, VectorRef);

    let cost = gas_params.base.unwrap_or_else(InternalGas::zero)
        + (gas_params.per_byte.unwrap_or_else(InternalGasPerByte::zero)
            * NumBytes::new(input.as_bytes_ref().len() as u64));

    let encoded = general_purpose::STANDARD.encode(input.as_bytes_ref().to_vec());

    Ok(NativeResult::ok(
        cost,
        smallvec![Value::vector_u8(encoded.into_bytes())],
    ))
}

/***************************************************************************************************
 * native fun decode
 **************************************************************************************************/
pub fn native_decode(
    gas_params: &EncodeDecodeGasParametersOption,
    _context: &mut NativeContext,
    ty_args: Vec<Type>,
    mut args: VecDeque<Value>,
) -> PartialVMResult<NativeResult> {
    debug_assert!(ty_args.is_empty());
    debug_assert!(args.len() == 1);

    let encoded_input = pop_arg!(args, VectorRef);

    let cost = gas_params.base.unwrap_or_else(InternalGas::zero)
        + (gas_params.per_byte.unwrap_or_else(InternalGasPerByte::zero)
            * NumBytes::new(encoded_input.as_bytes_ref().len() as u64));

    let bytes_ref = encoded_input.as_bytes_ref();
    let encoded_str = match std::str::from_utf8(&bytes_ref) {
        Ok(s) => s,
        Err(_) => return Ok(NativeResult::err(cost, E_DECODE_FAILED)),
    };

    let decoded =
        if encoded_str.contains('+') || encoded_str.contains('/') || encoded_str.contains('=') {
            match general_purpose::STANDARD.decode(encoded_str) {
                Ok(bytes) => bytes,
                Err(_) => return Ok(NativeResult::err(cost, E_DECODE_FAILED)),
            }
        } else {
            let padding_needed = (4 - (encoded_str.len() % 4)) % 4;
            let mut padded_bytes = Vec::with_capacity(encoded_str.len() + padding_needed);
            padded_bytes.extend_from_slice(encoded_str.as_bytes());
            padded_bytes.extend(std::iter::repeat(b'=').take(padding_needed));

            match general_purpose::URL_SAFE.decode(&padded_bytes) {
                Ok(bytes) => bytes,
                Err(_) => return Ok(NativeResult::err(cost, E_DECODE_FAILED)),
            }
        };
    Ok(NativeResult::ok(cost, smallvec![Value::vector_u8(decoded)]))
}

/***************************************************************************************************
 * gas parameter structs
 **************************************************************************************************/
#[derive(Debug, Clone)]
pub struct EncodeDecodeGasParametersOption {
    pub base: Option<InternalGas>,
    pub per_byte: Option<InternalGasPerByte>,
}

#[derive(Debug, Clone)]
pub struct GasParameters {
    pub encode: EncodeDecodeGasParametersOption,
    pub decode: EncodeDecodeGasParametersOption,
}

impl GasParameters {
    pub fn zeros() -> Self {
        Self {
            encode: EncodeDecodeGasParametersOption {
                base: Some(0.into()),
                per_byte: Some(0.into()),
            },
            decode: EncodeDecodeGasParametersOption {
                base: Some(0.into()),
                per_byte: Some(0.into()),
            },
        }
    }
}

pub fn make_all(gas_params: GasParameters) -> impl Iterator<Item = (String, NativeFunction)> {
    let mut natives = Vec::new();

    if gas_params.encode.base.is_some() || gas_params.encode.per_byte.is_some() {
        natives.push((
            "encode".to_string(),
            make_native(gas_params.encode, native_encode),
        ));
    }

    if gas_params.decode.base.is_some() || gas_params.decode.per_byte.is_some() {
        natives.push((
            "decode".to_string(),
            make_native(gas_params.decode, native_decode),
        ));
    }

    make_module_natives(natives)
}
