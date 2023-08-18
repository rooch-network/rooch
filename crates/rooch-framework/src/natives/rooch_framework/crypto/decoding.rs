// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::natives::helpers::{make_module_natives, make_native};
use move_binary_format::errors::PartialVMResult;
use move_vm_runtime::native_functions::{NativeContext, NativeFunction};
use move_vm_types::{
    loaded_data::runtime_types::Type,
    natives::function::NativeResult,
    pop_arg,
    values::{Value, VectorRef},
};
use smallvec::smallvec;
use std::collections::VecDeque;

pub const DECODE_FAILED: u64 = 0;

/***************************************************************************************************
 * native fun base58
 * Implementation of the Move native function `decoding::base58(encoded_address_bytes: &vector<u8>): vector<u8>`
 *   gas cost: decoding_base58_cost_base                               | base cost for function call and fixed opers
 *              + decoding_base58_data_cost_per_byte * msg.len()       | cost depends on length of message
 *              + decoding_base58_data_cost_per_block * num_blocks     | cost depends on number of blocks in message
 **************************************************************************************************/
 pub fn native_base58(
    _gas_params: &FromBytesGasParameters,
    _context: &mut NativeContext,
    ty_args: Vec<Type>,
    mut args: VecDeque<Value>,
) -> PartialVMResult<NativeResult> {
    debug_assert!(ty_args.is_empty());
    debug_assert!(args.len() == 1);

    // TODO(Gas): Charge the arg size dependent costs

    let cost = 0.into();

    let encoded_address_bytes = pop_arg!(args, VectorRef);

    let Ok(bs58_raw_bytes) = bs58::decode(encoded_address_bytes.as_bytes_ref().to_vec())
        .into_vec()
    else {
        return Ok(NativeResult::err(cost, DECODE_FAILED));
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
    _gas_params: &FromBytesGasParameters,
    _context: &mut NativeContext,
    ty_args: Vec<Type>,
    mut args: VecDeque<Value>,
) -> PartialVMResult<NativeResult> {
    debug_assert!(ty_args.is_empty());
    debug_assert!(args.len() == 2);

    // TODO(Gas): Charge the arg size dependent costs

    let cost = 0.into();

    let version_byte = pop_arg!(args, u8);
    let encoded_address_bytes = pop_arg!(args, VectorRef);

    let Ok(bs58_raw_bytes_without_checksum) = bs58::decode(encoded_address_bytes.as_bytes_ref().to_vec())
        .with_check(Some(version_byte))
        .into_vec()
    else {
        return Ok(NativeResult::err(cost, DECODE_FAILED));
    };

    Ok(NativeResult::ok(
        cost,
        smallvec![Value::vector_u8(bs58_raw_bytes_without_checksum)],
    ))
}

#[derive(Debug, Clone)]
pub struct FromBytesGasParameters {}

impl FromBytesGasParameters {
    pub fn zeros() -> Self {
        Self {}
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
    let natives = [(
        "base58",
        make_native(gas_params.base58, native_base58),
    ),
    (
        "base58check",
        make_native(gas_params.base58check, native_base58check),
    )];

    make_module_natives(natives)
}
