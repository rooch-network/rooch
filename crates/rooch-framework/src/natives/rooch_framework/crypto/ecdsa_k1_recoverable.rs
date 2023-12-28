// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::natives::helpers::{make_module_natives, make_native};
use fastcrypto::{
    hash::{Keccak256, Sha256},
    secp256k1::{recoverable::Secp256k1RecoverableSignature, Secp256k1PublicKey},
    traits::{RecoverableSignature, ToFromBytes},
};
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

pub const E_FAIL_TO_RECOVER_PUBKEY: u64 = 1;
pub const E_INVALID_SIGNATURE: u64 = 2;
pub const E_INVALID_PUBKEY: u64 = 3;
pub const E_INVALID_HASH_TYPE: u64 = 4;

pub const KECCAK256: u8 = 0;
pub const SHA256: u8 = 1;

pub fn native_ecrecover(
    gas_params: &FromBytesGasParameters,
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
    let cost = gas_params.base;

    let Ok(sig) = <Secp256k1RecoverableSignature as ToFromBytes>::from_bytes(&signature_ref) else {
        return Ok(NativeResult::err(cost, E_INVALID_SIGNATURE));
    };

    let pk = match hash {
        KECCAK256 => sig.recover_with_hash::<Keccak256>(&msg_ref),
        SHA256 => sig.recover_with_hash::<Sha256>(&msg_ref),
        _ => return Ok(NativeResult::err(cost, E_INVALID_HASH_TYPE)), // We should never reach here
    };

    match pk {
        Ok(pk) => Ok(NativeResult::ok(
            cost,
            smallvec![Value::vector_u8(pk.as_bytes().to_vec())],
        )),
        Err(_) => Ok(NativeResult::err(cost, E_FAIL_TO_RECOVER_PUBKEY)),
    }
}

pub fn native_decompress_pubkey(
    gas_params: &FromBytesGasParameters,
    _context: &mut NativeContext,
    ty_args: Vec<Type>,
    mut args: VecDeque<Value>,
) -> PartialVMResult<NativeResult> {
    debug_assert!(ty_args.is_empty());
    debug_assert!(args.len() == 1);

    let pubkey = pop_arg!(args, VectorRef);
    let pubkey_ref = pubkey.as_bytes_ref();

    // TODO(Gas): Charge the arg size dependent costs
    let cost = gas_params.base;

    match Secp256k1PublicKey::from_bytes(&pubkey_ref) {
        Ok(pubkey) => {
            let uncompressed = &pubkey.pubkey.serialize_uncompressed();
            Ok(NativeResult::ok(
                cost,
                smallvec![Value::vector_u8(uncompressed.to_vec())],
            ))
        }
        Err(_) => Ok(NativeResult::err(cost, E_INVALID_PUBKEY)),
    }
}

pub fn native_verify(
    gas_params: &FromBytesGasParameters,
    _context: &mut NativeContext,
    ty_args: Vec<Type>,
    mut args: VecDeque<Value>,
) -> PartialVMResult<NativeResult> {
    debug_assert!(ty_args.is_empty());
    debug_assert!(args.len() == 3);

    let hash = pop_arg!(args, u8);

    let msg = pop_arg!(args, VectorRef);
    let signature_bytes = pop_arg!(args, VectorRef);

    let msg_ref = msg.as_bytes_ref();
    let signature_bytes_ref = signature_bytes.as_bytes_ref();

    // TODO(Gas): Charge the arg size dependent costs

    let cost = gas_params.base;

    let Ok(sig) = <Secp256k1RecoverableSignature as ToFromBytes>::from_bytes(&signature_bytes_ref)
    else {
        return Ok(NativeResult::err(cost, E_INVALID_SIGNATURE));
    };

    let pk = match hash {
        KECCAK256 => match sig.recover_with_hash::<Keccak256>(&msg_ref) {
            Ok(pk) => pk,
            Err(_) => {
                return Ok(NativeResult::ok(cost, smallvec![Value::bool(false)]));
            }
        },
        SHA256 => match sig.recover_with_hash::<Sha256>(&msg_ref) {
            Ok(pk) => pk,
            Err(_) => {
                return Ok(NativeResult::ok(cost, smallvec![Value::bool(false)]));
            }
        },
        _ => {
            return Ok(NativeResult::ok(cost, smallvec![Value::bool(false)]));
        }
    };

    let result = match hash {
        KECCAK256 => pk
            .verify_recoverable_with_hash::<Keccak256>(&msg_ref, &sig)
            .is_ok(),
        SHA256 => pk
            .verify_recoverable_with_hash::<Sha256>(&msg_ref, &sig)
            .is_ok(),
        _ => false,
    };

    Ok(NativeResult::ok(cost, smallvec![Value::bool(result)]))
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
    pub verify: FromBytesGasParameters,
    pub ecrecover: FromBytesGasParameters,
    pub decompress_pubkey: FromBytesGasParameters,
}

impl GasParameters {
    pub fn zeros() -> Self {
        Self {
            verify: FromBytesGasParameters::zeros(),
            ecrecover: FromBytesGasParameters::zeros(),
            decompress_pubkey: FromBytesGasParameters::zeros(),
        }
    }
}

pub fn make_all(gas_params: GasParameters) -> impl Iterator<Item = (String, NativeFunction)> {
    let natives = [
        (
            "decompress_pubkey",
            make_native(gas_params.decompress_pubkey, native_decompress_pubkey),
        ),
        (
            "ecrecover",
            make_native(gas_params.ecrecover, native_ecrecover),
        ),
        ("verify", make_native(gas_params.verify, native_verify)),
    ];

    make_module_natives(natives)
}
