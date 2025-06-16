// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::natives::helpers::{make_module_natives, make_native};
use fastcrypto::{
    hash::Sha256,
    secp256r1::{Secp256r1PublicKey, Secp256r1Signature},
    traits::ToFromBytes,
};
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

// Error codes
const E_INVALID_SIGNATURE: u64 = 1;
const E_INVALID_PUBKEY: u64 = 2;
const E_INVALID_HASH_TYPE: u64 = 3;

// Hash type
const HASH_TYPE_SHA256: u8 = 1;

#[derive(Debug, Clone)]
pub struct FromBytesGasParameters {
    pub base: Option<InternalGas>,
    pub per_byte: Option<InternalGasPerByte>,
}

impl FromBytesGasParameters {
    pub fn zeros() -> Self {
        Self {
            base: Some(0.into()),
            per_byte: Some(0.into()),
        }
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

pub fn native_verify(
    gas_params: &FromBytesGasParameters,
    _context: &mut NativeContext,
    _ty_args: Vec<Type>,
    mut args: VecDeque<Value>,
) -> PartialVMResult<NativeResult> {
    debug_assert!(_ty_args.is_empty());
    debug_assert!(args.len() == 4);

    let hash_type = pop_arg!(args, u8);
    let msg = pop_arg!(args, VectorRef);
    let public_key = pop_arg!(args, VectorRef);
    let signature = pop_arg!(args, VectorRef);

    let msg_ref = msg.as_bytes_ref();
    let signature_bytes_ref = signature.as_bytes_ref();
    let public_key_bytes_ref = public_key.as_bytes_ref();

    let base = gas_params.base.expect("gas parameter should initialize");
    let per_byte = gas_params
        .per_byte
        .expect("gas parameter should initialize");

    let cost = base
        + per_byte * NumBytes::new(msg_ref.len() as u64)
        + per_byte * NumBytes::new(signature_bytes_ref.len() as u64)
        + per_byte * NumBytes::new(public_key_bytes_ref.as_slice().len() as u64);

    let sig = match Secp256r1Signature::from_bytes(signature_bytes_ref.as_slice()) {
        Ok(sig) => sig,
        Err(_e) => {
            return Ok(NativeResult::err(cost, E_INVALID_SIGNATURE));
        }
    };

    // Parse the public key
    let verifying_key = match Secp256r1PublicKey::from_bytes(public_key_bytes_ref.as_slice()) {
        Ok(key) => key,
        Err(_) => return Ok(NativeResult::err(cost, E_INVALID_PUBKEY)),
    };

    // Verify the signature
    let result = if hash_type == HASH_TYPE_SHA256 {
        verifying_key
            .verify_with_hash::<Sha256>(msg_ref.as_slice(), &sig)
            .is_ok()
    } else {
        return Ok(NativeResult::err(cost, E_INVALID_HASH_TYPE));
    };

    Ok(NativeResult::ok(cost, smallvec![Value::bool(result)]))
}

pub fn make_all(gas_params: GasParameters) -> impl Iterator<Item = (String, NativeFunction)> {
    let mut natives = vec![];
    if gas_params.verify.base.is_some() {
        natives.push((
            "native_verify",
            make_native(gas_params.verify, native_verify),
        ));
    }
    make_module_natives(natives)
}
