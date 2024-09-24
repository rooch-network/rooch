// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::natives::helpers::{make_module_natives, make_native};
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
use base64::{Engine as _, engine::general_purpose};

pub const E_DECODE_FAILED: u64 = 1;

/***************************************************************************************************
 * native fun encode
 **************************************************************************************************/
pub fn native_encode(
    gas_params: &GasParameters,
    _context: &mut NativeContext,
    ty_args: Vec<Type>,
    mut args: VecDeque<Value>,
) -> PartialVMResult<NativeResult> {
    debug_assert!(ty_args.is_empty());
    debug_assert!(args.len() == 1);

    let input = pop_arg!(args, VectorRef);

    let cost = gas_params.encode.base
        + (gas_params.encode.per_byte * NumBytes::new(input.as_bytes_ref().len() as u64));

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
    gas_params: &GasParameters,
    _context: &mut NativeContext,
    ty_args: Vec<Type>,
    mut args: VecDeque<Value>,
) -> PartialVMResult<NativeResult> {
    debug_assert!(ty_args.is_empty());
    debug_assert!(args.len() == 1);

    let encoded_input = pop_arg!(args, VectorRef);

    let cost = gas_params.decode.base
        + (gas_params.decode.per_byte * NumBytes::new(encoded_input.as_bytes_ref().len() as u64));

    let decoded = match general_purpose::STANDARD.decode(encoded_input.as_bytes_ref().to_vec()) {
        Ok(bytes) => bytes,
        Err(_) => return Ok(NativeResult::err(cost, E_DECODE_FAILED)),
    };

    Ok(NativeResult::ok(
        cost,
        smallvec![Value::vector_u8(decoded)],
    ))
}

/***************************************************************************************************
 * module
 **************************************************************************************************/
#[derive(Debug, Clone)]
pub struct EncodeDecodeGasParameters {
    pub base: InternalGas,
    pub per_byte: InternalGasPerByte,
}

impl EncodeDecodeGasParameters {
    pub fn zeros() -> Self {
        Self {
            base: 0.into(),
            per_byte: 0.into(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct GasParameters {
    pub encode: EncodeDecodeGasParameters,
    pub decode: EncodeDecodeGasParameters,
}

impl GasParameters {
    pub fn zeros() -> Self {
        Self {
            encode: EncodeDecodeGasParameters::zeros(),
            decode: EncodeDecodeGasParameters::zeros(),
        }
    }
}

pub fn make_all(gas_params: GasParameters) -> impl Iterator<Item = (String, NativeFunction)> {
    let natives = [
        (
            "encode".to_string(),
            make_native(gas_params.clone(), native_encode),
        ),
        (
            "decode".to_string(),
            make_native(gas_params, native_decode),
        ),
    ];

    make_module_natives(natives)
}

