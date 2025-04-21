// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::natives::helpers::{make_module_natives, make_native};
use k256::schnorr::{Signature, VerifyingKey};
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

pub const E_INVALID_SIGNATURE: u64 = 1;
pub const E_INVALID_VERIFYING_KEY: u64 = 2;

// optional function
/// verify schnorr signature
pub fn native_verify(
    gas_params: &VerifyGasParametersOption,
    _context: &mut NativeContext,
    ty_args: Vec<Type>,
    mut args: VecDeque<Value>,
) -> PartialVMResult<NativeResult> {
    debug_assert!(ty_args.is_empty());
    debug_assert!(args.len() == 3);

    let msg = pop_arg!(args, VectorRef);
    let verifying_key = pop_arg!(args, VectorRef);
    let signature = pop_arg!(args, VectorRef);

    let msg_bytes_ref = msg.as_bytes_ref();
    let verifying_key_bytes_ref = verifying_key.as_bytes_ref();
    let signature_bytes_ref = signature.as_bytes_ref();

    let cost = gas_params.base.unwrap_or_else(InternalGas::zero)
        + gas_params.per_byte.unwrap_or_else(InternalGasPerByte::zero)
            * NumBytes::new(msg_bytes_ref.len() as u64)
        + gas_params.per_byte.unwrap_or_else(InternalGasPerByte::zero)
            * NumBytes::new(verifying_key_bytes_ref.len() as u64)
        + gas_params.per_byte.unwrap_or_else(InternalGasPerByte::zero)
            * NumBytes::new(signature_bytes_ref.len() as u64);

    let Ok(sig) = Signature::try_from(signature_bytes_ref.to_vec().as_slice()) else {
        return Ok(NativeResult::err(cost, E_INVALID_SIGNATURE));
    };

    let Ok(verifying_key) = VerifyingKey::from_bytes(&verifying_key_bytes_ref) else {
        return Ok(NativeResult::err(cost, E_INVALID_VERIFYING_KEY));
    };

    let result = verifying_key.verify_raw(&msg_bytes_ref, &sig).is_ok();

    Ok(NativeResult::ok(cost, smallvec![Value::bool(result)]))
}

#[derive(Debug, Clone)]
pub struct VerifyGasParametersOption {
    pub base: Option<InternalGas>,
    pub per_byte: Option<InternalGasPerByte>,
}

impl VerifyGasParametersOption {
    pub fn zeros() -> Self {
        Self {
            base: Some(0.into()),
            per_byte: Some(0.into()),
        }
    }

    pub fn is_empty(&self) -> bool {
        self.base.is_none() || self.per_byte.is_none()
    }
}

/***************************************************************************************************
 * module
 **************************************************************************************************/

#[derive(Debug, Clone)]
pub struct VerifyGasParameters {
    pub verify: VerifyGasParametersOption,
}

impl VerifyGasParameters {
    pub fn zeros() -> Self {
        Self {
            verify: VerifyGasParametersOption::zeros(),
        }
    }

    pub fn is_empty(&self) -> bool {
        self.verify.is_empty()
    }
}

pub fn make_all(gas_params: VerifyGasParameters) -> impl Iterator<Item = (String, NativeFunction)> {
    let mut natives = Vec::new();

    if !gas_params.is_empty() {
        natives.push(("verify", make_native(gas_params.verify, native_verify)));
    }

    make_module_natives(natives)
}
