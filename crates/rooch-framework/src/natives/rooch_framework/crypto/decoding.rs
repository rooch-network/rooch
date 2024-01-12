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

pub const E_DECODE_FAILED: u64 = 1;

/***************************************************************************************************
 * native fun base58
 * Implementation of the Move native function `decoding::base58(encoded_address_bytes: &vector<u8>): vector<u8>`
 *   gas cost: decoding_base58_cost_base                               | base cost for function call and fixed opers
 *              + decoding_base58_data_cost_per_byte * msg.len()       | cost depends on length of message
 *              + decoding_base58_data_cost_per_block * num_blocks     | cost depends on number of blocks in message
 **************************************************************************************************/
pub fn native_base58(
    gas_params: &FromBytesGasParameters,
    _context: &mut NativeContext,
    ty_args: Vec<Type>,
    mut args: VecDeque<Value>,
) -> PartialVMResult<NativeResult> {
    debug_assert!(ty_args.is_empty());
    debug_assert!(args.len() == 1);

    let mut cost = gas_params.base;

    let encoded_address_bytes = pop_arg!(args, VectorRef);
    let input_bytes = encoded_address_bytes.as_bytes_ref().to_vec();
    cost += gas_params.per_byte * NumBytes::new(input_bytes.len() as u64);

    let Ok(bs58_raw_bytes) = bs58::decode(input_bytes).into_vec() else {
        return Ok(NativeResult::err(cost, E_DECODE_FAILED));
    };

    Ok(NativeResult::ok(
        cost,
        smallvec![Value::vector_u8(bs58_raw_bytes)],
    ))
}

/***************************************************************************************************
 * native fun base58check
 * Implementation of the Move native function `decoding::base58check(encoded_address_bytes: &vector<u8>, version_byte: u8): vector<u8>`
 *   gas cost: decoding_base58check_cost_base                               | base cost for function call and fixed opers
 *              + decoding_base58check_data_cost_per_byte * msg.len()       | cost depends on length of message
 *              + decoding_base58check_data_cost_per_block * num_blocks     | cost depends on number of blocks in message
 **************************************************************************************************/
pub fn native_base58check(
    gas_params: &FromBytesGasParameters,
    _context: &mut NativeContext,
    ty_args: Vec<Type>,
    mut args: VecDeque<Value>,
) -> PartialVMResult<NativeResult> {
    debug_assert!(ty_args.is_empty());
    debug_assert!(args.len() == 2);

    let mut cost = gas_params.base;

    let version_byte = pop_arg!(args, u8);
    let encoded_address_bytes = pop_arg!(args, VectorRef);

    let input_bytes = encoded_address_bytes.as_bytes_ref().to_vec();
    cost += gas_params.per_byte * NumBytes::new(input_bytes.len() as u64);

    let Ok(bs58_raw_bytes_without_checksum) = bs58::decode(input_bytes)
        .with_check(Some(version_byte))
        .into_vec()
    else {
        return Ok(NativeResult::err(cost, E_DECODE_FAILED));
    };

    Ok(NativeResult::ok(
        cost,
        smallvec![Value::vector_u8(bs58_raw_bytes_without_checksum)],
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
    pub base58: FromBytesGasParameters,
    pub base58check: FromBytesGasParameters,
}

impl GasParameters {
    pub fn zeros() -> Self {
        Self {
            base58: FromBytesGasParameters::zeros(),
            base58check: FromBytesGasParameters::zeros(),
        }
    }
}

pub fn make_all(gas_params: GasParameters) -> impl Iterator<Item = (String, NativeFunction)> {
    let natives = [
        ("base58", make_native(gas_params.base58, native_base58)),
        (
            "base58check",
            make_native(gas_params.base58check, native_base58check),
        ),
    ];

    make_module_natives(natives)
}
