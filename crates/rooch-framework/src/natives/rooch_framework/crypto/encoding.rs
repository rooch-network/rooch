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

pub const E_INVALID_PUBKEY: u64 = 1;
pub const E_EXCESSIVE_SCRIPT_SIZE: u64 = 2;
pub const E_INVALID_DATA: u64 = 3;
pub const E_INVALID_SCRIPT_VERSION: u64 = 4;

/***************************************************************************************************
 * native fun base58
 * Implementation of the Move native function `encoding::base58(address_bytes: &vector<u8>): vector<u8>`
 *   gas cost: encoding_base58_cost_base                               | base cost for function call and fixed opers
 *              + encoding_base58_data_cost_per_byte * msg.len()       | cost depends on length of message
 *              + encoding_base58_data_cost_per_block * num_blocks     | cost depends on number of blocks in message
 **************************************************************************************************/
pub fn native_base58(
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
 * native fun base58check
 * Implementation of the Move native function `encoding::base58check(address_bytes: &vector<u8>, version_byte: u8): vector<u8>`
 *   gas cost: encoding_base58check_cost_base                               | base cost for function call and fixed opers
 *              + encoding_base58check_data_cost_per_byte * msg.len()       | cost depends on length of message
 *              + encoding_base58check_data_cost_per_block * num_blocks     | cost depends on number of blocks in message
 **************************************************************************************************/
pub fn native_base58check(
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
 * native fun bech32
 * Implementation of the Move native function `encoding::bech32(public_key: &vector<u8>, version: u8): vector<u8>`
 *   gas cost: encoding_bech32_cost_base                               | base cost for function call and fixed opers
 *              + encoding_bech32_data_cost_per_byte * msg.len()       | cost depends on length of message
 *              + encoding_bech32_data_cost_per_block * num_blocks     | cost depends on number of blocks in message
 **************************************************************************************************/
pub fn native_bech32(
    gas_params: &FromBytesGasParameters,
    _context: &mut NativeContext,
    ty_args: Vec<Type>,
    mut args: VecDeque<Value>,
) -> PartialVMResult<NativeResult> {
    debug_assert!(ty_args.is_empty());
    debug_assert!(args.len() == 2);

    let hrp = pop_arg!(args, u8);
    let public_key = pop_arg!(args, VectorRef);
    let mut encoded = "".to_owned();

    let cost = gas_params.base
        + (gas_params.per_byte * NumBytes::new(public_key.as_bytes_ref().len() as u64));

    if hrp == 0 {
        encoded = bech32::encode("bech32", public_key.as_bytes_ref().to_vec().to_base32(), Variant::Bech32).unwrap();
    } else {
        encoded = bech32::encode("bech32m", public_key.as_bytes_ref().to_vec().to_base32(), Variant::Bech32m).unwrap();
    }

    Ok(NativeResult::ok(
        cost,
        smallvec![Value::vector_u8(encoded.bytes())],
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
    pub bech32: FromBytesGasParameters,
}

impl GasParameters {
    pub fn zeros() -> Self {
        Self {
            base58: FromBytesGasParameters::zeros(),
            base58check: FromBytesGasParameters::zeros(),
            bech32: FromBytesGasParameters::zeros(),
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
        ("bech32", make_native(gas_params.bech32, native_bech32)),
    ];

    make_module_natives(natives)
}
