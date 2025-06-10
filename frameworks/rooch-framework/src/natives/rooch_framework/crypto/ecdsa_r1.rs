// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::natives::helpers::{make_module_natives, make_native};
use move_binary_format::errors::PartialVMResult;
use move_core_types::gas_algebra::{InternalGas, InternalGasPerByte};
use move_vm_runtime::native_functions::{NativeContext, NativeFunction};
use move_vm_types::{
    loaded_data::runtime_types::Type,
    natives::function::NativeResult,
    pop_arg,
    values::{Value, VectorRef},
};
use p256::ecdsa::{signature::hazmat::PrehashVerifier, Signature, VerifyingKey};
use sha2::{Digest, Sha256};
use smallvec::smallvec;
use std::collections::VecDeque;
use generic_array::GenericArray;

// Error codes
const E_INVALID_SIGNATURE: u64 = 1;
const E_INVALID_PUBKEY: u64 = 2;

/// The signature type used to distinguish which signature to be used when verifying.
const SIGNATURE_SCHEME_ID: u8 = 2;

#[derive(Debug, Clone)]
pub struct GasParameters {
    pub verify: InternalGas,
    pub verify_per_byte: InternalGasPerByte,
}

impl GasParameters {
    pub fn zeros() -> Self {
        Self {
            verify: 0.into(),
            verify_per_byte: 0.into(),
        }
    }
}

pub fn native_verify(
    gas_params: &GasParameters,
    _context: &mut NativeContext,
    _ty_args: Vec<Type>,
    mut args: VecDeque<Value>,
) -> PartialVMResult<NativeResult> {
    debug_assert!(_ty_args.is_empty());
    debug_assert!(args.len() == 3);

    let msg = pop_arg!(args, Vec<u8>);
    let signature = pop_arg!(args, Vec<u8>);
    let public_key = pop_arg!(args, Vec<u8>);

    // Hash the message
    let mut hasher = Sha256::new();
    hasher.update(&msg);
    let hashed_msg = hasher.finalize();

    // Parse the public key
    let verifying_key = match VerifyingKey::from_sec1_bytes(&public_key) {
        Ok(key) => key,
        Err(_) => return Ok(NativeResult::ok(gas_params.verify, smallvec![Value::bool(false)])),
    };

    // Parse the signature
    let sig = if signature.len() == 64 {
        let sig = GenericArray::from_slice(&signature);
        match Signature::from_bytes(sig) {
            Ok(sig) => sig,
            Err(_) => return Ok(NativeResult::ok(gas_params.verify, smallvec![Value::bool(false)])),
        }
    } else {
        return Ok(NativeResult::ok(gas_params.verify, smallvec![Value::bool(false)]));
    };

    // Verify the signature
    let result = verifying_key
        .verify_prehash(&hashed_msg, &sig)
        .is_ok();

    Ok(NativeResult::ok(gas_params.verify, smallvec![Value::bool(result)]))
}

pub fn make_all(gas_params: GasParameters) -> impl Iterator<Item = (String, NativeFunction)> {
    let natives = [("verify", make_native(gas_params.clone(), native_verify))];

    natives.into_iter().map(|(func_name, func)| {
        (
            format!("ecdsa_r1::{}", func_name),
            func,
        )
    })
} 