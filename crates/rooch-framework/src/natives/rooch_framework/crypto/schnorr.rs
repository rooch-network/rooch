// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::natives::helpers::{make_module_natives, make_native};
use fastcrypto::{
    hash::{Keccak256, Sha256},
    secp256k1::schnorr::{SchnorrPublicKey, SchnorrSignature},
    traits::ToFromBytes,
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

pub const E_INVALID_SIGNATURE: u64 = 1;
pub const E_INVALID_PUBKEY: u64 = 2;

pub const KECCAK256: u8 = 0;
pub const SHA256: u8 = 1;

pub fn native_verify(
    gas_params: &FromBytesGasParameters,
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

    let cost = gas_params.base;

    let Ok(sign) = SchnorrSignature::from_bytes(&signature_bytes_ref) else {
        return Ok(NativeResult::err(cost, E_INVALID_SIGNATURE));
    };

    let Ok(public_key) = <SchnorrPublicKey as ToFromBytes>::from_bytes(&public_key_bytes_ref)
    else {
        return Ok(NativeResult::err(cost, E_INVALID_PUBKEY));
    };

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
