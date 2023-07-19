// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::natives::helpers::{make_module_natives, make_native};
use fastcrypto::{
    hash::{Keccak256, Sha256},
    secp256k1::schnorr::{SchnorrPublicKey, SchnorrSignature},
    traits::ToFromBytes,
};
use move_binary_format::errors::PartialVMResult;
use move_vm_runtime::native_functions::{NativeContext, NativeFunction};
use move_vm_types::{
    loaded_data::runtime_types::Type,
    natives::function::NativeResult,
    pop_arg,
    values::{Value, VectorRef},
};
use rust_secp256k1::schnorr::Signature;
use smallvec::smallvec;
use std::collections::VecDeque;

pub const INVALID_SIGNATURE: u64 = 0;
pub const INVALID_PUBKEY: u64 = 1;

pub const KECCAK256: u8 = 0;
pub const SHA256: u8 = 1;

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
    let signature_bytes_ref = signature_bytes.as_bytes_ref();
    let public_key_bytes_ref = public_key_bytes.as_bytes_ref();

    // TODO(Gas): Charge the arg size dependent costs

    let cost = 0.into();

    let Ok(sig) = Signature::from_slice(&signature_bytes_ref) else {
        return Ok(NativeResult::err(cost, INVALID_SIGNATURE));
    };

    let Ok(public_key) = <SchnorrPublicKey as ToFromBytes>::from_bytes(&public_key_bytes_ref) else {
        return Ok(NativeResult::err(cost, INVALID_PUBKEY));
    };

    let sign = SchnorrSignature::from(&sig);
    let result = match hash {
        KECCAK256 => public_key
            .verify_with_hash::<Keccak256>(&msg_ref, &sign)
            .is_ok(),
        SHA256 => public_key
            .verify_with_hash::<Sha256>(&msg_ref, &sign)
            .is_ok(),
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
    pub verify: FromBytesGasParameters,
}

impl GasParameters {
    pub fn zeros() -> Self {
        Self {
            verify: FromBytesGasParameters::zeros(),
        }
    }
}

pub fn make_all(gas_params: GasParameters) -> impl Iterator<Item = (String, NativeFunction)> {
    let natives = [("verify", make_native(gas_params.verify, native_verify))];

    make_module_natives(natives)
}
