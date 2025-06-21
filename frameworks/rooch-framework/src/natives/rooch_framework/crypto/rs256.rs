// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::natives::helpers::{make_module_natives, make_native};
use fastcrypto::rsa::{RSAPublicKey, RSASignature};
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
const SHA256: u8 = 0;

#[derive(Debug, Clone)]
pub struct VerifyGasParameters {
    pub base: Option<InternalGas>,
    pub per_byte: Option<InternalGasPerByte>,
}

impl VerifyGasParameters {
    pub fn zeros() -> Self {
        Self {
            base: Some(0.into()),
            per_byte: Some(0.into()),
        }
    }

    pub fn init(base: InternalGas, per_byte: InternalGasPerByte) -> Self {
        Self {
            base: Some(base),
            per_byte: Some(per_byte),
        }
    }
}

/***************************************************************************************************
 * module
 **************************************************************************************************/

#[derive(Debug, Clone)]
pub struct GasParameters {
    pub verify: VerifyGasParameters,
    pub verify_prehash: VerifyGasParameters,
}

impl GasParameters {
    pub fn zeros() -> Self {
        Self {
            verify: VerifyGasParameters::zeros(),
            verify_prehash: VerifyGasParameters::zeros(),
        }
    }
}

pub fn native_verify(
    gas_params: &VerifyGasParameters,
    _context: &mut NativeContext,
    _ty_args: Vec<Type>,
    mut args: VecDeque<Value>,
) -> PartialVMResult<NativeResult> {
    debug_assert!(_ty_args.is_empty());
    debug_assert!(args.len() == 4);

    let msg = pop_arg!(args, VectorRef);
    let e = pop_arg!(args, VectorRef);
    let n = pop_arg!(args, VectorRef);
    let signature = pop_arg!(args, VectorRef);

    let msg_bytes_ref = msg.as_bytes_ref();
    let e_bytes_ref = e.as_bytes_ref();
    let n_bytes_ref = n.as_bytes_ref();
    let signature_bytes_ref = signature.as_bytes_ref();

    let base = gas_params.base.expect("gas parameter should initialize");
    let per_byte = gas_params
        .per_byte
        .expect("gas parameter should initialize");

    let cost = base
        + per_byte * NumBytes::new(msg_bytes_ref.len() as u64)
        + per_byte * NumBytes::new(e_bytes_ref.len() as u64)
        + per_byte * NumBytes::new(n_bytes_ref.len() as u64)
        + per_byte * NumBytes::new(signature_bytes_ref.len() as u64);

    // Parse the signature from raw bytes
    let sig = match RSASignature::from_bytes(signature_bytes_ref.as_slice()) {
        Ok(sig) => sig,
        Err(_e) => {
            return Ok(NativeResult::err(cost, E_INVALID_SIGNATURE));
        }
    };

    // Parse the public key from modulus (n) and exponent (e)
    let pubkey =
        match RSAPublicKey::from_raw_components(n_bytes_ref.as_slice(), e_bytes_ref.as_slice()) {
            Ok(key) => key,
            Err(_) => return Ok(NativeResult::err(cost, E_INVALID_PUBKEY)),
        };

    // Verify the signature
    let result = pubkey.verify(msg_bytes_ref.as_slice(), &sig).is_ok();

    Ok(NativeResult::ok(cost, smallvec![Value::bool(result)]))
}

pub fn native_verify_prehash(
    gas_params: &VerifyGasParameters,
    _context: &mut NativeContext,
    _ty_args: Vec<Type>,
    mut args: VecDeque<Value>,
) -> PartialVMResult<NativeResult> {
    debug_assert!(_ty_args.is_empty());
    debug_assert!(args.len() == 5);

    let hash_type = pop_arg!(args, u8);
    let hashed_msg = pop_arg!(args, VectorRef);
    let e = pop_arg!(args, VectorRef);
    let n = pop_arg!(args, VectorRef);
    let signature = pop_arg!(args, VectorRef);

    let hashed_msg_bytes_ref = hashed_msg.as_bytes_ref();
    let e_bytes_ref = e.as_bytes_ref();
    let n_bytes_ref = n.as_bytes_ref();
    let signature_bytes_ref = signature.as_bytes_ref();

    let base = gas_params.base.expect("gas parameter should initialize");
    let per_byte = gas_params
        .per_byte
        .expect("gas parameter should initialize");

    let cost = base
        + per_byte * NumBytes::new(hashed_msg_bytes_ref.len() as u64)
        + per_byte * NumBytes::new(e_bytes_ref.len() as u64)
        + per_byte * NumBytes::new(n_bytes_ref.len() as u64)
        + per_byte * NumBytes::new(signature_bytes_ref.len() as u64);

    // Parse the signature from raw bytes
    let sig = match RSASignature::from_bytes(signature_bytes_ref.as_slice()) {
        Ok(sig) => sig,
        Err(_e) => {
            return Ok(NativeResult::err(cost, E_INVALID_SIGNATURE));
        }
    };

    // Parse the public key from modulus (n) and exponent (e)
    let pubkey =
        match RSAPublicKey::from_raw_components(n_bytes_ref.as_slice(), e_bytes_ref.as_slice()) {
            Ok(key) => key,
            Err(_) => return Ok(NativeResult::err(cost, E_INVALID_PUBKEY)),
        };

    // Verify the signature with sha256 hash type
    let result = if hash_type == SHA256 {
        pubkey
            .verify_prehash(hashed_msg_bytes_ref.as_slice(), &sig)
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
    if gas_params.verify_prehash.base.is_some() {
        natives.push((
            "native_verify_prehash",
            make_native(gas_params.verify_prehash, native_verify_prehash),
        ));
    }
    make_module_natives(natives)
}
