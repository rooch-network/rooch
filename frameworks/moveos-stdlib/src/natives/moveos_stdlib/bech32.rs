// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::natives::helpers::{make_module_natives, make_native};
use bech32::{
    primitives::hrp,
    segwit::{self, VERSION_0, VERSION_1},
    Bech32, Bech32m, Fe32, Hrp, NoChecksum,
};
use move_binary_format::errors::PartialVMResult;
use move_core_types::gas_algebra::{InternalGas, InternalGasPerByte, NumBytes};
use move_vm_runtime::native_functions::{NativeContext, NativeFunction};
use move_vm_types::{
    loaded_data::runtime_types::Type, natives::function::NativeResult, pop_arg, values::Value,
};
use smallvec::smallvec;
use std::collections::VecDeque;

pub const E_ENCODE_FAILED: u64 = 1;
pub const E_DECODE_FAILED: u64 = 2;
pub const E_INVALID_BIP_CODE: u64 = 3;
pub const E_INVALID_NETWORK: u64 = 4;
pub const E_INVALID_WITNESS_VERSION: u64 = 5;

/***************************************************************************************************
 * native fun encode
 * Implementation of the Move native function `bech32::encode(bip: u16, hrp: vector<u8>, data: vector<u8>): vector<u8>`
 *   gas cost: encode_cost_base                               | base cost for function call and fixed opers
 *              + encode_data_cost_per_byte * msg.len()       | cost depends on length of message
 *              + encode_data_cost_per_block * num_blocks     | cost depends on number of blocks in message
 **************************************************************************************************/
pub fn native_encode(
    gas_params: &FromBytesGasParameters,
    _context: &mut NativeContext,
    ty_args: Vec<Type>,
    mut args: VecDeque<Value>,
) -> PartialVMResult<NativeResult> {
    debug_assert!(ty_args.is_empty());
    debug_assert!(args.len() == 3);

    let data = pop_arg!(args, Vec<u8>);
    let hrp = pop_arg!(args, Vec<u8>);
    let bip = pop_arg!(args, u16);

    let cost = gas_params.base
        + (gas_params.per_byte * NumBytes::new(hrp.len() as u64 + data.len() as u64 + bip as u64));

    let hrp_string = match String::from_utf8(hrp.to_vec()) {
        Ok(v) => v,
        Err(_) => return Ok(NativeResult::err(cost, E_ENCODE_FAILED)),
    };
    let hrp = match Hrp::parse(&hrp_string) {
        Ok(v) => v,
        Err(_) => return Ok(NativeResult::err(cost, E_ENCODE_FAILED)),
    };

    // bip173 = bech32; bip350 = bech32m; bip0 = NoChecksum.
    let encoded = if bip == 173 {
        match bech32::encode::<Bech32>(hrp, &data.to_vec()) {
            Ok(v) => v,
            Err(_) => return Ok(NativeResult::err(cost, E_ENCODE_FAILED)),
        }
    } else if bip == 350 {
        match bech32::encode::<Bech32m>(hrp, &data.to_vec()) {
            Ok(v) => v,
            Err(_) => return Ok(NativeResult::err(cost, E_ENCODE_FAILED)),
        }
    } else if bip == 0 {
        match bech32::encode::<NoChecksum>(hrp, &data.to_vec()) {
            Ok(v) => v,
            Err(_) => return Ok(NativeResult::err(cost, E_ENCODE_FAILED)),
        }
    } else {
        return Ok(NativeResult::err(cost, E_INVALID_BIP_CODE));
    };

    Ok(NativeResult::ok(
        cost,
        smallvec![Value::vector_u8(encoded.bytes())],
    ))
}

/***************************************************************************************************
 * native fun segwit_encode
 * Implementation of the Move native function `bech32::segwit_encode(network: vector<u8>, witness_version: u8, data: vector<u8>): vector<u8>`
 *   gas cost: segwit_encode_cost_base                               | base cost for function call and fixed opers
 *              + segwit_encode_data_cost_per_byte * msg.len()       | cost depends on length of message
 *              + segwit_encode_data_cost_per_block * num_blocks     | cost depends on number of blocks in message
 **************************************************************************************************/
pub fn native_segwit_encode(
    gas_params: &FromBytesGasParameters,
    _context: &mut NativeContext,
    ty_args: Vec<Type>,
    mut args: VecDeque<Value>,
) -> PartialVMResult<NativeResult> {
    debug_assert!(ty_args.is_empty());
    debug_assert!(args.len() == 3);

    let data = pop_arg!(args, Vec<u8>);
    let witness_version = pop_arg!(args, u8);
    let network = pop_arg!(args, Vec<u8>);

    let cost = gas_params.base
        + (gas_params.per_byte
            * NumBytes::new(network.len() as u64 + data.len() as u64 + witness_version as u64));

    let network_string = match String::from_utf8(network.to_vec()) {
        Ok(v) => v,
        Err(_) => return Ok(NativeResult::err(cost, E_ENCODE_FAILED)),
    };

    let hrp = if network_string == "bc" {
        hrp::BC // bitcoin mainnet network
    } else if network_string == "tb" {
        hrp::TB // bitcoin testnet or signet network
    } else if network_string == "bcrt" {
        hrp::BCRT // bitcoin regtest network
    } else {
        return Ok(NativeResult::err(cost, E_INVALID_NETWORK));
    };

    let version = if witness_version == 0 {
        VERSION_0 // bech32 -> q
    } else if (1..=16).contains(&witness_version) {
        VERSION_1 // bech32m -> p (taproot)
    } else {
        return Ok(NativeResult::err(cost, E_INVALID_WITNESS_VERSION));
    };

    let encoded = match segwit::encode(hrp, version, &data) {
        Ok(v) => v,
        Err(_) => return Ok(NativeResult::err(cost, E_ENCODE_FAILED)),
    };

    Ok(NativeResult::ok(
        cost,
        smallvec![Value::vector_u8(encoded.bytes())],
    ))
}

/***************************************************************************************************
 * native fun decode
 * Implementation of the Move native function `bech32::decode(hrp: vector<u8>, encoded: vector<u8>): vector<u8>`
 *   gas cost: decode_cost_base                               | base cost for function call and fixed opers
 *              + decode_data_cost_per_byte * msg.len()       | cost depends on length of message
 *              + decode_data_cost_per_block * num_blocks     | cost depends on number of blocks in message
 **************************************************************************************************/
pub fn native_decode(
    gas_params: &FromBytesGasParameters,
    _context: &mut NativeContext,
    ty_args: Vec<Type>,
    mut args: VecDeque<Value>,
) -> PartialVMResult<NativeResult> {
    debug_assert!(ty_args.is_empty());
    debug_assert!(args.len() == 2);

    let encoded = pop_arg!(args, Vec<u8>);
    let hrp = pop_arg!(args, Vec<u8>);

    let cost = gas_params.base
        + (gas_params.per_byte * NumBytes::new(hrp.len() as u64 + encoded.len() as u64));

    let binding = encoded;
    let encoded_str = match std::str::from_utf8(&binding) {
        Ok(v) => v,
        Err(_) => return Ok(NativeResult::err(cost, E_DECODE_FAILED)),
    };

    let hrp_string = match String::from_utf8(hrp.to_vec()) {
        Ok(v) => v,
        Err(_) => return Ok(NativeResult::err(cost, E_DECODE_FAILED)),
    };
    let hrp_origin = match Hrp::parse(&hrp_string) {
        Ok(v) => v,
        Err(_) => return Ok(NativeResult::err(cost, E_DECODE_FAILED)),
    };
    let (hrp, data) = match bech32::decode(encoded_str) {
        Ok((v1, v2)) => (v1, v2),
        Err(_) => return Ok(NativeResult::err(cost, E_DECODE_FAILED)),
    };
    if hrp != hrp_origin {
        return Ok(NativeResult::err(cost, E_DECODE_FAILED));
    };

    Ok(NativeResult::ok(cost, smallvec![Value::vector_u8(data)]))
}

/***************************************************************************************************
 * native fun segwit_decode
 * Implementation of the Move native function `bech32::segwit_decode(hrp: vector<u8>, witness_ascii_version: u8, encoded: vector<u8>): vector<u8>`
 *   gas cost: segwit_decode_cost_base                               | base cost for function call and fixed opers
 *              + segwit_decode_data_cost_per_byte * msg.len()       | cost depends on length of message
 *              + segwit_decode_data_cost_per_block * num_blocks     | cost depends on number of blocks in message
 **************************************************************************************************/
pub fn native_segwit_decode(
    gas_params: &FromBytesGasParameters,
    _context: &mut NativeContext,
    ty_args: Vec<Type>,
    mut args: VecDeque<Value>,
) -> PartialVMResult<NativeResult> {
    debug_assert!(ty_args.is_empty());
    debug_assert!(args.len() == 3);

    let encoded = pop_arg!(args, Vec<u8>);
    let witness_ascii_version = pop_arg!(args, u8);
    let hrp = pop_arg!(args, Vec<u8>);

    let cost = gas_params.base
        + (gas_params.per_byte
            * NumBytes::new(
                encoded.len() as u64 + witness_ascii_version as u64 + hrp.len() as u64,
            ));

    let binding = encoded;
    let encoded_str = match std::str::from_utf8(&binding) {
        Ok(v) => v,
        Err(_) => return Ok(NativeResult::err(cost, E_DECODE_FAILED)),
    };

    let hrp_string = match String::from_utf8(hrp.to_vec()) {
        Ok(v) => v,
        Err(_) => return Ok(NativeResult::err(cost, E_DECODE_FAILED)),
    };
    let hrp_origin = match Hrp::parse(&hrp_string) {
        Ok(v) => v,
        Err(_) => return Ok(NativeResult::err(cost, E_DECODE_FAILED)),
    };
    let version_origin = Fe32::from_char_unchecked(witness_ascii_version);

    let (hrp, version, data) = match segwit::decode(encoded_str) {
        Ok((v1, v2, v3)) => (v1, v2, v3),
        Err(_) => return Ok(NativeResult::err(cost, E_DECODE_FAILED)),
    };
    if hrp != hrp_origin {
        return Ok(NativeResult::err(cost, E_DECODE_FAILED));
    };
    if version != version_origin {
        return Ok(NativeResult::err(cost, E_DECODE_FAILED));
    }

    Ok(NativeResult::ok(cost, smallvec![Value::vector_u8(data)]))
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
    pub encode: FromBytesGasParameters,
    pub segwit_encode: FromBytesGasParameters,
    pub decode: FromBytesGasParameters,
    pub segwit_decode: FromBytesGasParameters,
}

impl GasParameters {
    pub fn zeros() -> Self {
        Self {
            encode: FromBytesGasParameters::zeros(),
            segwit_encode: FromBytesGasParameters::zeros(),
            decode: FromBytesGasParameters::zeros(),
            segwit_decode: FromBytesGasParameters::zeros(),
        }
    }
}

pub fn make_all(gas_params: GasParameters) -> impl Iterator<Item = (String, NativeFunction)> {
    let natives = [
        ("encode", make_native(gas_params.encode, native_encode)),
        (
            "segwit_encode",
            make_native(gas_params.segwit_encode, native_segwit_encode),
        ),
        ("decode", make_native(gas_params.decode, native_decode)),
        (
            "segwit_decode",
            make_native(gas_params.segwit_decode, native_segwit_decode),
        ),
    ];
    make_module_natives(natives)
}
