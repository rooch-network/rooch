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
 * native fun encoding
 * Implementation of the Move native function `base58::encoding(address_bytes: &vector<u8>): vector<u8>`
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
    debug_assert!(args.len() == 1);

    let address_bytes = pop_arg!(args, VectorRef);

    let cost = gas_params.base
        + (gas_params.per_byte * NumBytes::new(address_bytes.as_bytes_ref().len() as u64));

    let bs58_bytes = bs58::encode(address_bytes.as_bytes_ref().to_vec()).into_vec();

    Ok(NativeResult::ok(
        cost,
        smallvec![Value::vector_u8(bs58_bytes)],
    ))
}

/***************************************************************************************************
 * native fun checksum_encoding
 * Implementation of the Move native function `base58::checksum_encoding(address_bytes: &vector<u8>, version_byte: u8): vector<u8>`
 *   gas cost: checksum_encoding_cost_base                               | base cost for function call and fixed opers
 *              + checksum_encoding_data_cost_per_byte * msg.len()       | cost depends on length of message
 *              + checksum_encoding_data_cost_per_block * num_blocks     | cost depends on number of blocks in message
 **************************************************************************************************/
pub fn native_checksum_encoding(
    gas_params: &FromBytesGasParameters,
    _context: &mut NativeContext,
    ty_args: Vec<Type>,
    mut args: VecDeque<Value>,
) -> PartialVMResult<NativeResult> {
    debug_assert!(ty_args.is_empty());
    debug_assert!(args.len() == 2);

    let version_byte = pop_arg!(args, u8);
    let address_bytes = pop_arg!(args, VectorRef);

    let cost = gas_params.base
        + (gas_params.per_byte * NumBytes::new(address_bytes.as_bytes_ref().len() as u64));

    let bs58_checksum_bytes = bs58::encode(address_bytes.as_bytes_ref().to_vec())
        .with_check_version(version_byte)
        .into_vec();

    Ok(NativeResult::ok(
        cost,
        smallvec![Value::vector_u8(bs58_checksum_bytes)],
    ))
}

/***************************************************************************************************
 * native fun decoding
 * Implementation of the Move native function `base58::decoding(encoded_address_bytes: &vector<u8>): vector<u8>`
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
 * native fun checksum_decoding
 * Implementation of the Move native function `base58::checksum_decoding(encoded_address_bytes: &vector<u8>, version_byte: u8): vector<u8>`
 *   gas cost: checksum_decoding_cost_base                               | base cost for function call and fixed opers
 *              + checksum_decoding_data_cost_per_byte * msg.len()       | cost depends on length of message
 *              + checksum_decoding_data_cost_per_block * num_blocks     | cost depends on number of blocks in message
 **************************************************************************************************/
pub fn native_checksum_decoding(
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
    pub encoding: FromBytesGasParameters,
    pub checksum_encoding: FromBytesGasParameters,
    pub decoding: FromBytesGasParameters,
    pub checksum_decoding: FromBytesGasParameters,
}

impl GasParameters {
    pub fn zeros() -> Self {
        Self {
            encoding: FromBytesGasParameters::zeros(),
            checksum_encoding: FromBytesGasParameters::zeros(),
            decoding: FromBytesGasParameters::zeros(),
            checksum_decoding: FromBytesGasParameters::zeros(),
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
            "checksum_encoding",
            make_native(gas_params.checksum_encoding, native_checksum_encoding),
        ),
        (
            "decoding",
            make_native(gas_params.decoding, native_decoding),
        ),
        (
            "checksum_decoding",
            make_native(gas_params.checksum_decoding, native_checksum_decoding),
        ),
    ];

    make_module_natives(natives)
}
