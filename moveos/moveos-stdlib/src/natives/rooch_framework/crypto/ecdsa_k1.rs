// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::natives::helpers::{make_module_natives, make_native};
use fastcrypto::{
    error::FastCryptoError,
    hash::{Keccak256, Sha256},
    secp256k1::{
        recoverable::Secp256k1RecoverableSignature, Secp256k1PublicKey, Secp256k1Signature,
    },
    traits::{RecoverableSignature, ToFromBytes},
};
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

pub const FAIL_TO_RECOVER_PUBKEY: u64 = 0;
pub const INVALID_SIGNATURE: u64 = 1;
pub const INVALID_PUBKEY: u64 = 2;

pub const KECCAK256: u8 = 0;
pub const SHA256: u8 = 1;

pub fn native_ecrecover(
    _gas_params: &FromBytesGasParameters,
    _context: &mut NativeContext,
    ty_args: Vec<Type>,
    mut args: VecDeque<Value>,
) -> PartialVMResult<NativeResult> {
    debug_assert!(ty_args.is_empty());
    debug_assert!(args.len() == 3);

    let hash = pop_arg!(args, u8);
    let msg = pop_arg!(args, VectorRef);
    let signature = pop_arg!(args, VectorRef);

    let msg_ref = msg.as_bytes_ref();
    let signature_ref = signature.as_bytes_ref();

    // TODO(Gas): Charge the arg size dependent costs

    let cost = 0.into();

    let Ok(sig) = <Secp256k1RecoverableSignature as ToFromBytes>::from_bytes(&signature_ref) else {
        return Ok(NativeResult::err(cost, INVALID_SIGNATURE));
    };

    let pk = match hash {
        KECCAK256 => sig.recover_with_hash::<Keccak256>(&msg_ref),
        SHA256 => sig.recover_with_hash::<Sha256>(&msg_ref),
        _ => Err(FastCryptoError::InvalidInput), // We should never reach here
    };

    match pk {
        Ok(pk) => Ok(NativeResult::ok(
            cost,
            smallvec![Value::vector_u8(pk.as_bytes().to_vec())],
        )),
        Err(_) => Ok(NativeResult::err(cost, FAIL_TO_RECOVER_PUBKEY)),
    }
}

pub fn native_decompress_pubkey(
    _gas_params: &FromBytesGasParameters,
    _context: &mut NativeContext,
    ty_args: Vec<Type>,
    mut args: VecDeque<Value>,
) -> PartialVMResult<NativeResult> {
    debug_assert!(ty_args.is_empty());
    debug_assert!(args.len() == 1);

    let pubkey = pop_arg!(args, VectorRef);
    let pubkey_ref = pubkey.as_bytes_ref();

    // TODO(Gas): Charge the arg size dependent costs
    let cost = 0.into();

    match Secp256k1PublicKey::from_bytes(&pubkey_ref) {
        Ok(pubkey) => {
            let uncompressed = &pubkey.pubkey.serialize_uncompressed();
            Ok(NativeResult::ok(
                cost,
                smallvec![Value::vector_u8(uncompressed.to_vec())],
            ))
        }
        Err(_) => Ok(NativeResult::err(cost, INVALID_PUBKEY)),
    }
}

pub fn native_verify(
    _gas_params: &FromBytesGasParameters,
    _context: &mut NativeContext,
    ty_args: Vec<Type>,
    mut args: VecDeque<Value>,
) -> PartialVMResult<NativeResult> {
    debug_assert!(ty_args.is_empty());
    debug_assert!(args.len() == 4);

    let hash = pop_arg!(args, u8);

    let msg = pop_arg!(args, VectorRef);
    let public_key_bytes = pop_arg!(args, VectorRef);
    let signature_bytes = pop_arg!(args, VectorRef);

    let msg_ref = msg.as_bytes_ref();
    let public_key_bytes_ref = public_key_bytes.as_bytes_ref();
    let signature_bytes_ref = signature_bytes.as_bytes_ref();

    // TODO(Gas): Charge the arg size dependent costs

    let cost = 0.into();

    let Ok(sig) = <Secp256k1Signature as ToFromBytes>::from_bytes(&signature_bytes_ref) else {
        return Ok(NativeResult::ok(cost, smallvec![Value::bool(false)]));
    };

    let Ok(pk) = <Secp256k1PublicKey as ToFromBytes>::from_bytes(&public_key_bytes_ref) else {
        return Ok(NativeResult::ok(cost, smallvec![Value::bool(false)]));
    };

    let result = match hash {
        KECCAK256 => pk.verify_with_hash::<Keccak256>(&msg_ref, &sig).is_ok(),
        SHA256 => pk.verify_with_hash::<Sha256>(&msg_ref, &sig).is_ok(),
        _ => false,
    };

    Ok(NativeResult::ok(cost, smallvec![Value::bool(result)]))
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
    pub ecrecover: FromBytesGasParameters,
    pub decompress_pubkey: FromBytesGasParameters,
    pub verify: FromBytesGasParameters,
}

impl GasParameters {
    pub fn zeros() -> Self {
        Self {
            verify: FromBytesGasParameters::zeros(),
            decompress_pubkey: FromBytesGasParameters::zeros(),
            ecrecover: FromBytesGasParameters::zeros(),
        }
    }
}

pub fn make_all(gas_params: GasParameters) -> impl Iterator<Item = (String, NativeFunction)> {
    let natives = [
        (
            "ecrecover",
            make_native(gas_params.ecrecover, native_ecrecover),
        ),
        (
            "decompress_pubkey",
            make_native(gas_params.decompress_pubkey, native_decompress_pubkey),
        ),
        ("verify", make_native(gas_params.verify, native_verify)),
    ];

    make_module_natives(natives)
}
