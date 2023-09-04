// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::natives::helpers::{make_module_natives, make_native};
use fastcrypto::hash::{Blake2b256, HashFunction, Keccak256, Ripemd160};
use move_binary_format::errors::PartialVMResult;
use move_core_types::gas_algebra::InternalGas;
use move_vm_runtime::native_functions::{NativeContext, NativeFunction};
use move_vm_types::{
    loaded_data::runtime_types::Type,
    natives::function::NativeResult,
    pop_arg,
    values::{Value, VectorRef},
};
use smallvec::smallvec;
use std::collections::VecDeque;

fn hash<H: HashFunction<DIGEST_SIZE>, const DIGEST_SIZE: usize>(
    _: &mut NativeContext,
    ty_args: Vec<Type>,
    mut args: VecDeque<Value>,
) -> PartialVMResult<NativeResult> {
    debug_assert!(ty_args.is_empty());
    debug_assert!(args.len() == 1);

    let msg = pop_arg!(args, VectorRef);

    // TODO(Gas): Charge the arg size dependent costs

    Ok(NativeResult::ok(
        0.into(),
        smallvec![Value::vector_u8(
            H::digest(msg.as_bytes_ref().as_slice()).digest
        )],
    ))
}

/***************************************************************************************************
 * native fun keccak256
 * Implementation of the Move native function `hash::keccak256(data: &vector<u8>): vector<u8>`
 *   gas cost: hash_keccak256_cost_base                               | base cost for function call and fixed opers
 *              + hash_keccak256_data_cost_per_byte * msg.len()       | cost depends on length of message
 *              + hash_keccak256_data_cost_per_block * num_blocks     | cost depends on number of blocks in message
 **************************************************************************************************/
pub fn native_keccak256(
    _gas_params: &FromBytesGasParameters,
    context: &mut NativeContext,
    ty_args: Vec<Type>,
    args: VecDeque<Value>,
) -> PartialVMResult<NativeResult> {
    hash::<Keccak256, 32>(context, ty_args, args)
}

/***************************************************************************************************
 * native fun blake2b256
 * Implementation of the Move native function `hash::blake2b256(data: &vector<u8>): vector<u8>`
 *   gas cost: hash_blake2b256_cost_base                               | base cost for function call and fixed opers
 *              + hash_blake2b256_data_cost_per_byte * msg.len()       | cost depends on length of message
 *              + hash_blake2b256_data_cost_per_block * num_blocks     | cost depends on number of blocks in message
 **************************************************************************************************/
pub fn native_blake2b256(
    _gas_params: &FromBytesGasParameters,
    context: &mut NativeContext,
    ty_args: Vec<Type>,
    args: VecDeque<Value>,
) -> PartialVMResult<NativeResult> {
    hash::<Blake2b256, 32>(context, ty_args, args)
}

/***************************************************************************************************
 * native fun ripemd160
 * Implementation of the Move native function `hash::ripemd160(data: &vector<u8>): vector<u8>`
 *   gas cost: hash_ripemd160_cost_base                               | base cost for function call and fixed opers
 *              + hash_ripemd160_data_cost_per_byte * msg.len()       | cost depends on length of message
 *              + hash_ripemd160_data_cost_per_block * num_blocks     | cost depends on number of blocks in message
 **************************************************************************************************/
pub fn native_ripemd160(
    _gas_params: &FromBytesGasParameters,
    context: &mut NativeContext,
    ty_args: Vec<Type>,
    args: VecDeque<Value>,
) -> PartialVMResult<NativeResult> {
    hash::<Ripemd160, 20>(context, ty_args, args)
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

/***************************************************************************************************
 * module
 **************************************************************************************************/

#[derive(Debug, Clone)]
pub struct GasParameters {
    pub keccak256: FromBytesGasParameters,
    pub blake2b256: FromBytesGasParameters,
    pub ripemd160: FromBytesGasParameters,
}

impl GasParameters {
    pub fn zeros() -> Self {
        Self {
            keccak256: FromBytesGasParameters::zeros(),
            blake2b256: FromBytesGasParameters::zeros(),
            ripemd160: FromBytesGasParameters::zeros(),
        }
    }
}

pub fn make_all(gas_params: GasParameters) -> impl Iterator<Item = (String, NativeFunction)> {
    let natives = [
        (
            "keccak256",
            make_native(gas_params.keccak256, native_keccak256),
        ),
        (
            "blake2b256",
            make_native(gas_params.blake2b256, native_blake2b256),
        ),
        (
            "ripemd160",
            make_native(gas_params.ripemd160, native_ripemd160),
        ),
    ];

    make_module_natives(natives)
}
