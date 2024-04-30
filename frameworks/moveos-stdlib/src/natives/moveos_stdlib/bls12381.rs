// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::natives::helpers::{make_module_natives, make_native};
use fastcrypto::{
    bls12381::{min_pk, min_sig},
    traits::{ToFromBytes, VerifyingKey},
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

// TODO: implement perblock cost?
const _BLS12381_BLOCK_SIZE: usize = 64;

pub const E_SIG_FAILED: u64 = 1;
pub const E_PUBKEY_FAILED: u64 = 2;

/***************************************************************************************************
 * native fun bls12381_min_sig_verify
 * Implementation of the Move native function `bls12381_min_sig_verify(signature: &vector<u8>, public_key: &vector<u8>, msg: &vector<u8>): bool`
 *   gas cost: bls12381_bls12381_min_sig_verify_cost_base                                     | covers various fixed costs in the oper
 *              + bls12381_bls12381_min_sig_verify_msg_cost_per_byte    * size_of(msg)        | covers cost of operating on each byte of `msg`
 *              + bls12381_bls12381_min_sig_verify_msg_cost_per_block   * num_blocks(msg)     | covers cost of operating on each block in `msg`
 * Note: each block is of size `BLS12381_BLOCK_SIZE` bytes, and we round up.
 *       `signature` and `public_key` are fixed size, so their costs are included in the base cost.
 **************************************************************************************************/
pub fn native_bls12381_min_sig_verify(
    gas_params: &FromBytesGasParameters,
    _context: &mut NativeContext,
    ty_args: Vec<Type>,
    mut args: VecDeque<Value>,
) -> PartialVMResult<NativeResult> {
    debug_assert!(ty_args.is_empty());
    debug_assert!(args.len() == 3);

    let msg = pop_arg!(args, VectorRef);
    let public_key = pop_arg!(args, VectorRef);
    let signature = pop_arg!(args, VectorRef);

    let msg_bytes_ref = msg.as_bytes_ref();
    let public_key_bytes_ref = public_key.as_bytes_ref();
    let signature_bytes_ref = signature.as_bytes_ref();

    let cost = gas_params.base
        + (gas_params.per_byte
            * NumBytes::new(
                msg_bytes_ref.len() as u64
                    + public_key_bytes_ref.len() as u64
                    + signature_bytes_ref.len() as u64,
            ));

    let Ok(signature) =
        <min_sig::BLS12381Signature as ToFromBytes>::from_bytes(&signature_bytes_ref)
    else {
        return Ok(NativeResult::err(cost, E_SIG_FAILED));
    };

    let public_key =
        match <min_sig::BLS12381PublicKey as ToFromBytes>::from_bytes(&public_key_bytes_ref) {
            Ok(public_key) => match public_key.validate() {
                Ok(_) => public_key,
                Err(_) => return Ok(NativeResult::err(cost, E_PUBKEY_FAILED)),
            },
            Err(_) => return Ok(NativeResult::err(cost, E_PUBKEY_FAILED)),
        };

    Ok(NativeResult::ok(
        cost,
        smallvec![Value::bool(
            public_key.verify(&msg_bytes_ref, &signature).is_ok()
        )],
    ))
}

/***************************************************************************************************
 * native fun bls12381_min_pk_verify
 * Implementation of the Move native function `bls12381_min_pk_verify(signature: &vector<u8>, public_key: &vector<u8>, msg: &vector<u8>): bool`
 *   gas cost: bls12381_bls12381_min_pk_verify_cost_base                                     | covers various fixed costs in the oper
 *              + bls12381_bls12381_min_pk_verify_msg_cost_per_byte    * size_of(msg)        | covers cost of operating on each byte of `msg`
 *              + bls12381_bls12381_min_pk_verify_msg_cost_per_block   * num_blocks(msg)     | covers cost of operating on each block in `msg`
 * Note: each block is of size `BLS12381_BLOCK_SIZE` bytes, and we round up.
 *       `signature` and `public_key` are fixed size, so their costs are included in the base cost.
 **************************************************************************************************/
pub fn native_bls12381_min_pk_verify(
    gas_params: &FromBytesGasParameters,
    _context: &mut NativeContext,
    ty_args: Vec<Type>,
    mut args: VecDeque<Value>,
) -> PartialVMResult<NativeResult> {
    debug_assert!(ty_args.is_empty());
    debug_assert!(args.len() == 3);

    let msg = pop_arg!(args, VectorRef);
    let public_key = pop_arg!(args, VectorRef);
    let signature = pop_arg!(args, VectorRef);

    let msg_bytes_ref = msg.as_bytes_ref();
    let public_key_bytes_ref = public_key.as_bytes_ref();
    let signature_bytes_ref = signature.as_bytes_ref();

    let cost = gas_params.base
        + (gas_params.per_byte
            * NumBytes::new(
                msg_bytes_ref.len() as u64
                    + public_key_bytes_ref.len() as u64
                    + signature_bytes_ref.len() as u64,
            ));

    let signature =
        match <min_pk::BLS12381Signature as ToFromBytes>::from_bytes(&signature_bytes_ref) {
            Ok(signature) => signature,
            Err(_) => return Ok(NativeResult::err(cost, E_SIG_FAILED)),
        };

    let public_key =
        match <min_pk::BLS12381PublicKey as ToFromBytes>::from_bytes(&public_key_bytes_ref) {
            Ok(public_key) => match public_key.validate() {
                Ok(_) => public_key,
                Err(_) => return Ok(NativeResult::err(cost, E_PUBKEY_FAILED)),
            },
            Err(_) => return Ok(NativeResult::err(cost, E_PUBKEY_FAILED)),
        };

    Ok(NativeResult::ok(
        cost,
        smallvec![Value::bool(
            public_key.verify(&msg_bytes_ref, &signature).is_ok()
        )],
    ))
}

#[derive(Debug, Clone)]
pub struct FromBytesGasParameters {
    pub base: InternalGas,
    pub per_byte: InternalGasPerByte,
}

impl FromBytesGasParameters {
    pub fn zeros() -> Self {
        Self {
            base: 0.into(),
            per_byte: 0.into(),
        }
    }
}

/***************************************************************************************************
 * module
 **************************************************************************************************/

#[derive(Debug, Clone)]
pub struct GasParameters {
    pub bls12381_min_sig_verify: FromBytesGasParameters,
    pub bls12381_min_pk_verify: FromBytesGasParameters,
}

impl GasParameters {
    pub fn zeros() -> Self {
        Self {
            bls12381_min_sig_verify: FromBytesGasParameters::zeros(),
            bls12381_min_pk_verify: FromBytesGasParameters::zeros(),
        }
    }
}

pub fn make_all(gas_params: GasParameters) -> impl Iterator<Item = (String, NativeFunction)> {
    let natives = [
        (
            "bls12381_min_sig_verify",
            make_native(
                gas_params.bls12381_min_sig_verify,
                native_bls12381_min_sig_verify,
            ),
        ),
        (
            "bls12381_min_pk_verify",
            make_native(
                gas_params.bls12381_min_pk_verify,
                native_bls12381_min_pk_verify,
            ),
        ),
    ];
    make_module_natives(natives)
}
