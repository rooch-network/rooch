// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::natives::helpers::{make_module_natives, make_native};
use bitcoin::Network;
use bitcoin::{Address, PublicKey};
use bitcoin_bech32::{constants::Network as Bech32Network, u5, WitnessProgram};
use fastcrypto::{secp256k1::Secp256k1PublicKey, traits::ToFromBytes};
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

pub const INVALID_PUBKEY: u64 = 0;
pub const EXCESSIVE_SCRIPT_SIZE: u64 = 1;
pub const INVALID_DATA: u64 = 2;
pub const INVALID_SCRIPT_VERSION: u64 = 3;

/***************************************************************************************************
 * native fun base58
 * Implementation of the Move native function `encoding::base58(address_bytes: &vector<u8>): vector<u8>`
 *   gas cost: encoding_base58_cost_base                               | base cost for function call and fixed opers
 *              + encoding_base58_data_cost_per_byte * msg.len()       | cost depends on length of message
 *              + encoding_base58_data_cost_per_block * num_blocks     | cost depends on number of blocks in message
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

    let address_bytes = pop_arg!(args, VectorRef);

    let bs58_bytes = bs58::encode(address_bytes.as_bytes_ref().to_vec())
        .into_vec();

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
    let address_bytes = pop_arg!(args, VectorRef);

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
    _gas_params: &FromBytesGasParameters,
    _context: &mut NativeContext,
    ty_args: Vec<Type>,
    mut args: VecDeque<Value>,
) -> PartialVMResult<NativeResult> {
    debug_assert!(ty_args.is_empty());
    debug_assert!(args.len() == 2);

    // TODO(Gas): Charge the arg size dependent costs

    let cost = 0.into();

    let version = pop_arg!(args, u8);
    let public_key = pop_arg!(args, VectorRef);

    // Version 0 for bech32 encoding and 1-16 are for bech32m encoding
    let Ok(version) = u5::try_from_u8(version) else {
        return Ok(NativeResult::err(cost, INVALID_DATA));
    };

    let Ok(witness_program) = WitnessProgram::new(
        version,
        public_key.as_bytes_ref().to_vec(),
        Bech32Network::Bitcoin, // TODO network selection
    ) else {
        return Ok(NativeResult::err(cost, INVALID_SCRIPT_VERSION));
    };

    let address = witness_program.to_address();
    let address_bytes = address.as_bytes().to_vec();

    Ok(NativeResult::ok(
        cost,
        smallvec![Value::vector_u8(address_bytes)],
    ))
}

/***************************************************************************************************
 * native fun p2pkh
 * Implementation of the Move native function `encoding::p2pkh(public_key: &vector<u8>): vector<u8>`
 *   gas cost: encoding_p2pkh_cost_base                               | base cost for function call and fixed opers
 *              + encoding_p2pkh_data_cost_per_byte * msg.len()       | cost depends on length of message
 *              + encoding_p2pkh_data_cost_per_block * num_blocks     | cost depends on number of blocks in message
 **************************************************************************************************/
pub fn native_p2pkh(
    _gas_params: &FromBytesGasParameters,
    _context: &mut NativeContext,
    ty_args: Vec<Type>,
    mut args: VecDeque<Value>,
) -> PartialVMResult<NativeResult> {
    debug_assert!(ty_args.is_empty());
    debug_assert!(args.len() == 1);

    // TODO(Gas): Charge the arg size dependent costs

    let cost = 0.into();

    let public_key = pop_arg!(args, VectorRef);
    let public_key_bytes_ref = public_key.as_bytes_ref();

    let Ok(public_key) = <Secp256k1PublicKey as ToFromBytes>::from_bytes(&public_key_bytes_ref) else {
        return Ok(NativeResult::err(cost, INVALID_PUBKEY));
    };

    let bitcoin_public_key = PublicKey::new(public_key.pubkey);

    // Generate the P2PKH address from the bitcoin public key
    let p2pkh_address = Address::p2pkh(&bitcoin_public_key, Network::Bitcoin); // TODO network selection
    let p2pkh_address_bytes = p2pkh_address.to_string().as_bytes().to_vec();

    Ok(NativeResult::ok(
        cost,
        smallvec![Value::vector_u8(p2pkh_address_bytes)],
    ))
}

/***************************************************************************************************
 * native fun p2sh
 * Implementation of the Move native function `encoding::p2sh(public_key: &vector<u8>): vector<u8>`
 *   gas cost: encoding_p2sh_cost_base                               | base cost for function call and fixed opers
 *              + encoding_p2sh_data_cost_per_byte * msg.len()       | cost depends on length of message
 *              + encoding_p2sh_data_cost_per_block * num_blocks     | cost depends on number of blocks in message
 **************************************************************************************************/
pub fn native_p2sh(
    _gas_params: &FromBytesGasParameters,
    _context: &mut NativeContext,
    ty_args: Vec<Type>,
    mut args: VecDeque<Value>,
) -> PartialVMResult<NativeResult> {
    debug_assert!(ty_args.is_empty());
    debug_assert!(args.len() == 1);

    // TODO(Gas): Charge the arg size dependent costs

    let cost = 0.into();

    let public_key = pop_arg!(args, VectorRef);
    let public_key_bytes_ref = public_key.as_bytes_ref();

    let Ok(public_key) = <Secp256k1PublicKey as ToFromBytes>::from_bytes(&public_key_bytes_ref) else {
        return Ok(NativeResult::err(cost, INVALID_PUBKEY));
    };

    let bitcoin_public_key = PublicKey::new(public_key.pubkey);

    // Create a redeem script (e.g., P2PKH)
    let script_pubkey = Address::p2pkh(&bitcoin_public_key, Network::Bitcoin).script_pubkey(); // TODO network selection
    let redeem_script = script_pubkey.as_script();
    // Generate the P2SH address from the redeem script
    let Ok(p2sh_address) = Address::p2sh(
        redeem_script,
        Network::Bitcoin, // TODO network selection
    ) else {
        return Ok(NativeResult::err(cost, EXCESSIVE_SCRIPT_SIZE));
    };
    let p2sh_address_bytes = p2sh_address.to_string().as_bytes().to_vec();

    Ok(NativeResult::ok(
        cost,
        smallvec![Value::vector_u8(p2sh_address_bytes)],
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
    pub bech32: FromBytesGasParameters,
    pub p2pkh: FromBytesGasParameters,
    pub p2sh: FromBytesGasParameters,
}

impl GasParameters {
    pub fn zeros() -> Self {
        Self {
            base58: FromBytesGasParameters::zeros(),
            base58check: FromBytesGasParameters::zeros(),
            bech32: FromBytesGasParameters::zeros(),
            p2pkh: FromBytesGasParameters::zeros(),
            p2sh: FromBytesGasParameters::zeros(),
        }
    }
}

pub fn make_all(gas_params: GasParameters) -> impl Iterator<Item = (String, NativeFunction)> {
    let natives = [
        (
            "base58",
            make_native(gas_params.base58, native_base58),
        ),
        (
            "base58check",
            make_native(gas_params.base58check, native_base58check),
        ),
        ("bech32", make_native(gas_params.bech32, native_bech32)),
        ("p2pkh", make_native(gas_params.p2pkh, native_p2pkh)),
        ("p2sh", make_native(gas_params.p2sh, native_p2sh)),
    ];

    make_module_natives(natives)
}
