// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::natives::helpers::{make_module_natives, make_native};
use bech32::{self, ToBase32, Variant};
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
 * native fun encoding
 * Implementation of the Move native function `bech32::encoding(public_key: &vector<u8>, witness_version: u8): vector<u8>`
 *   gas cost: encoding_cost_base                               | base cost for function call and fixed opers
 *              + encoding_data_cost_per_byte * msg.len()       | cost depends on length of message
 *              + encoding_data_cost_per_block * num_blocks     | cost depends on number of blocks in message
 **************************************************************************************************/
pub fn native_encoding(
    gas_params: &FromBytesGasParameters,
    _context: &mut NativeContext,
    ty_args: Vec<Type>,
    mut args: VecDeque<Value>,
) -> PartialVMResult<NativeResult> {
    debug_assert!(ty_args.is_empty());
    debug_assert!(args.len() == 2);

    let witness_version = pop_arg!(args, u8);
    let public_key = pop_arg!(args, VectorRef);

    let cost = gas_params.base
        + (gas_params.per_byte * NumBytes::new(public_key.as_bytes_ref().len() as u64));

    let (hrp, variant) = if witness_version == 0 {
        ("bech32", Variant::Bech32)
    } else {
        ("bech32m", Variant::Bech32m)
    };

    let encoded =
        bech32::encode(hrp, public_key.as_bytes_ref().to_vec().to_base32(), variant).unwrap();

    Ok(NativeResult::ok(
        cost,
        smallvec![Value::vector_u8(encoded.bytes())],
    ))
}

/***************************************************************************************************
 * native fun decoding
 * Implementation of the Move native function `bech32::decoding(data: u8): vector<u8>`
 *   gas cost: decoding_cost_base                               | base cost for function call and fixed opers
 *              + decoding_data_cost_per_byte * msg.len()       | cost depends on length of message
 *              + decoding_data_cost_per_block * num_blocks     | cost depends on number of blocks in message
 **************************************************************************************************/
pub fn native_decoding(
    gas_params: &FromBytesGasParameters,
    _context: &mut NativeContext,
    ty_args: Vec<Type>,
    mut args: VecDeque<Value>,
) -> PartialVMResult<NativeResult> {
    debug_assert!(ty_args.is_empty());
    debug_assert!(args.len() == 1);

    let data = pop_arg!(args, VectorRef);

    let cost =
        gas_params.base + (gas_params.per_byte * NumBytes::new(data.as_bytes_ref().len() as u64));

    let binding = data.as_bytes_ref();
    let data_str = match std::str::from_utf8(&binding) {
        Ok(v) => v,
        Err(_) => return Ok(NativeResult::err(cost, E_DECODE_FAILED)),
    };

    let (hrp, public_key, variant) = match bech32::decode(data_str) {
        Ok((v1, v2, v3)) => (v1, v2, v3),
        Err(_) => return Ok(NativeResult::err(cost, E_DECODE_FAILED)),
    };

    if hrp != "bech32" && hrp != "bech32m" {
        return Ok(NativeResult::err(cost, E_DECODE_FAILED));
    };

    if variant != Variant::Bech32 && variant != Variant::Bech32m {
        return Ok(NativeResult::err(cost, E_DECODE_FAILED));
    }

    // TODO redact public length to 20 and 32 bytes
    if public_key.len() != 32 && public_key.len() != 52 {
        return Ok(NativeResult::err(cost, E_DECODE_FAILED));
    }

    Ok(NativeResult::ok(
        cost,
        smallvec![Value::vector_u8(public_key.into_iter().map(u8::from))],
    ))
}

#[derive(Debug, Clone)]
pub struct FromBytesGasParameters {
    pub base: InternalGas,
    pub per_byte: InternalGasPerByte,
}

impl FromBytesGasParameters {
    pub fn zeros() -> Self {
        Self {
            base: 0.into(),
            per_byte: 0.into(),
        }
    }
}

/***************************************************************************************************
 * module
 **************************************************************************************************/

#[derive(Debug, Clone)]
pub struct GasParameters {
    pub encoding: FromBytesGasParameters,
    pub decoding: FromBytesGasParameters,
}

impl GasParameters {
    pub fn zeros() -> Self {
        Self {
            encoding: FromBytesGasParameters::zeros(),
            decoding: FromBytesGasParameters::zeros(),
        }
    }
}

pub fn make_all(gas_params: GasParameters) -> impl Iterator<Item = (String, NativeFunction)> {
    let natives = [
        (
            "encoding",
            make_native(gas_params.encoding, native_encoding),
        ),
        (
            "decoding",
            make_native(gas_params.decoding, native_decoding),
        ),
    ];

    make_module_natives(natives)
}
