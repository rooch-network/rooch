// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use fastcrypto::{
    ed25519::{Ed25519PublicKey, Ed25519Signature},
    traits::{ToFromBytes, VerifyingKey},
};
use move_binary_format::errors::PartialVMResult;
use move_vm_types::{
    loaded_data::runtime_types::Type,
    natives::function::NativeResult,
    pop_arg,
    values::{Value, VectorRef},
};

use crate::natives::helpers::{make_module_natives, make_native};
use move_vm_runtime::native_functions::{NativeContext, NativeFunction};

use move_core_types::gas_algebra::InternalGas;
use smallvec::smallvec;
use std::collections::VecDeque;

/***************************************************************************************************
 * native fun ed25519_verify
 * Implementation of the Move native function `ed25519::ed25519_verify(signature: &vector<u8>, public_key: &vector<u8>, msg: &vector<u8>): bool;`
 *   gas cost: ed25519_ed25519_verify_cost_base                          | base cost for function call and fixed opers
 *              + ed25519_ed25519_verify_msg_cost_per_byte * msg.len()   | cost depends on length of message
 *              + ed25519_ed25519_verify_msg_cost_per_block * block_size | cost depends on number of blocks in message
 **************************************************************************************************/
pub fn native_verify(
    _gas_params: &FromBytesGasParameters,
    _context: &mut NativeContext,
    ty_args: Vec<Type>,
    mut args: VecDeque<Value>,
) -> PartialVMResult<NativeResult> {
    debug_assert!(ty_args.is_empty());
    debug_assert!(args.len() == 3);

    // TODO(Gas): Charge the arg size dependent costs

    let cost = 0.into();

    let msg = pop_arg!(args, VectorRef);
    let msg_ref = msg.as_bytes_ref();
    let public_key_bytes = pop_arg!(args, VectorRef);
    let public_key_bytes_ref = public_key_bytes.as_bytes_ref();
    let signature_bytes = pop_arg!(args, VectorRef);
    let signature_bytes_ref = signature_bytes.as_bytes_ref();

    let Ok(signature) = <Ed25519Signature as ToFromBytes>::from_bytes(&signature_bytes_ref) else {
        return Ok(NativeResult::ok(cost, smallvec![Value::bool(false)]));
    };

    let Ok(public_key) = <Ed25519PublicKey as ToFromBytes>::from_bytes(&public_key_bytes_ref) else {
        return Ok(NativeResult::ok(cost, smallvec![Value::bool(false)]));
    };

    Ok(NativeResult::ok(
        cost,
        smallvec![Value::bool(public_key.verify(&msg_ref, &signature).is_ok())],
    ))
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
