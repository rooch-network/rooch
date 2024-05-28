// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::natives::helpers::{make_module_natives, make_native};
use move_binary_format::errors::PartialVMResult;
use move_core_types::gas_algebra::{InternalGas, InternalGasPerByte, NumBytes};
use move_vm_runtime::native_functions::{NativeContext, NativeFunction};
use move_vm_types::{
    loaded_data::runtime_types::Type, natives::function::NativeResult, pop_arg, values::Value,
};
use revm_precompile::{blake2, bn128, kzg_point_evaluation, modexp, secp256k1};
use revm_primitives::Env;
use smallvec::smallvec;
use std::collections::VecDeque;

pub const E_EC_RECOVER_FAILED: u64 = 1;
pub const E_MODEXP_FAILED: u64 = 5;
pub const E_EC_ADD_FAILED: u64 = 6;
pub const E_EC_MUL_FAILED: u64 = 7;
pub const E_EC_PAIRING_FAILED: u64 = 8;
pub const E_BLAKE2F_FAILED: u64 = 9;
pub const E_POINT_EVALUATION_FAILED: u64 = 10;
pub const E_INVALID_INPUT_SIZE: u64 = 11;

/***************************************************************************************************
 * native function `ec_recover(hash: vector<u8>, v: vector<u8>, r: vector<u8>, s: vector<u8>): vector<u8>`
 **************************************************************************************************/
pub fn native_ec_recover(
    gas_params: &FromBytesGasParameters,
    _context: &mut NativeContext,
    ty_args: Vec<Type>,
    mut args: VecDeque<Value>,
) -> PartialVMResult<NativeResult> {
    debug_assert!(ty_args.is_empty());
    debug_assert!(args.len() == 4);

    let s = pop_arg!(args, Vec<u8>);
    let r = pop_arg!(args, Vec<u8>);
    let v = pop_arg!(args, Vec<u8>);
    let hash = pop_arg!(args, Vec<u8>);

    if s.len() != 32 || r.len() != 32 || v.len() != 32 || hash.len() != 32 {
        return Ok(NativeResult::err(0.into(), E_INVALID_INPUT_SIZE));
    }
    let cost = gas_params.base + (gas_params.per_byte * NumBytes::new(128_u64));

    let mut data: Vec<u8> = Vec::new();
    data.extend(hash);
    data.extend(v);
    data.extend(r);
    data.extend(s);

    match secp256k1::ec_recover_run(&data.into(), 5000) {
        Ok((_, out)) => Ok(NativeResult::ok(
            cost,
            smallvec![Value::vector_u8(out.to_vec())],
        )),

        Err(_) => Ok(NativeResult::err(cost, E_EC_RECOVER_FAILED)),
    }
}

/***************************************************************************************************
 * native function `modexp(b_size: vector<u8>, e_size: vector<u8>, m_size: vector<u8>, b: vector<u8>, e: vector<u8>, m: vector<u8>): vector<u8>`
 **************************************************************************************************/
pub fn native_modexp(
    gas_params: &FromBytesGasParameters,
    _context: &mut NativeContext,
    ty_args: Vec<Type>,
    mut args: VecDeque<Value>,
) -> PartialVMResult<NativeResult> {
    debug_assert!(ty_args.is_empty());
    debug_assert!(args.len() == 6);

    let m = pop_arg!(args, Vec<u8>);
    let e = pop_arg!(args, Vec<u8>);
    let b = pop_arg!(args, Vec<u8>);
    let m_size = pop_arg!(args, Vec<u8>);
    let e_size = pop_arg!(args, Vec<u8>);
    let b_size = pop_arg!(args, Vec<u8>);

    let cost = gas_params.base
        + (gas_params.per_byte
            * NumBytes::new(
                m.len() as u64
                    + e.len() as u64
                    + b.len() as u64
                    + m_size.len() as u64
                    + e_size.len() as u64
                    + b_size.len() as u64,
            ));

    let mut data: Vec<u8> = Vec::new();
    data.extend(b_size);
    data.extend(e_size);
    data.extend(m_size);
    data.extend(b);
    data.extend(e);
    data.extend(m);

    match modexp::byzantium_run(&data.into(), 100_000_000_000) {
        Ok((_, output)) => Ok(NativeResult::ok(
            cost,
            smallvec![Value::vector_u8(output.to_vec())],
        )),

        Err(_) => Ok(NativeResult::err(cost, E_MODEXP_FAILED)),
    }
}

/***************************************************************************************************
 * native function `ec_add(x1: vector<u8>, y1: vector<u8>, x2: vector<u8>, y2: vector<u8>): (vector<u8>, vector<u8>)`
 **************************************************************************************************/
pub fn native_ec_add(
    gas_params: &FromBytesGasParameters,
    _context: &mut NativeContext,
    ty_args: Vec<Type>,
    mut args: VecDeque<Value>,
) -> PartialVMResult<NativeResult> {
    debug_assert!(ty_args.is_empty());
    debug_assert!(args.len() == 4);

    let y2 = pop_arg!(args, Vec<u8>);
    let x2 = pop_arg!(args, Vec<u8>);
    let y1 = pop_arg!(args, Vec<u8>);
    let x1 = pop_arg!(args, Vec<u8>);

    if y2.len() != 32 || x2.len() != 32 || y1.len() != 32 || x1.len() != 32 {
        return Ok(NativeResult::err(0.into(), E_INVALID_INPUT_SIZE));
    }

    let cost = gas_params.base + (gas_params.per_byte * NumBytes::new(128_u64));

    let mut data: Vec<u8> = Vec::new();
    data.extend(x1);
    data.extend(y1);
    data.extend(x2);
    data.extend(y2);

    match bn128::run_add(&data, 0, 10) {
        Ok((_, sum)) => {
            let (sum_x, sum_y) = sum.split_at(32);
            let sum_x = Value::vector_u8(sum_x.to_vec());
            let sum_y = Value::vector_u8(sum_y.to_vec());
            Ok(NativeResult::ok(cost, smallvec![sum_x, sum_y]))
        }

        Err(_) => Ok(NativeResult::err(cost, E_EC_ADD_FAILED)),
    }
}

/***************************************************************************************************
 * native function `ec_mul(x1: vector<u8>, y1: vector<u8>, s: vector<u8>): (vector<u8>, vector<u8>)`
 **************************************************************************************************/
pub fn native_ec_mul(
    gas_params: &FromBytesGasParameters,
    _context: &mut NativeContext,
    ty_args: Vec<Type>,
    mut args: VecDeque<Value>,
) -> PartialVMResult<NativeResult> {
    debug_assert!(ty_args.is_empty());
    debug_assert!(args.len() == 3);

    let s = pop_arg!(args, Vec<u8>);
    let y1 = pop_arg!(args, Vec<u8>);
    let x1 = pop_arg!(args, Vec<u8>);

    if s.len() != 32 || y1.len() != 32 || x1.len() != 32 {
        return Ok(NativeResult::err(0.into(), E_INVALID_INPUT_SIZE));
    }

    let cost = gas_params.base + (gas_params.per_byte * NumBytes::new(96_u64));

    let mut data: Vec<u8> = Vec::new();
    data.extend(x1);
    data.extend(y1);
    data.extend(s);

    match bn128::run_mul(&data, 0, 10) {
        Ok((_, mul)) => {
            let (mul_x, mul_y) = mul.split_at(32);
            let mul_x = Value::vector_u8(mul_x.to_vec());
            let mul_y = Value::vector_u8(mul_y.to_vec());
            Ok(NativeResult::ok(cost, smallvec![mul_x, mul_y]))
        }

        Err(_) => Ok(NativeResult::err(cost, E_EC_MUL_FAILED)),
    }
}

/***************************************************************************************************
 * native function `ec_pairing(data: vector<u8>): vector<u8>`
 **************************************************************************************************/
pub fn native_ec_pairing(
    gas_params: &FromBytesGasParameters,
    _context: &mut NativeContext,
    ty_args: Vec<Type>,
    mut args: VecDeque<Value>,
) -> PartialVMResult<NativeResult> {
    debug_assert!(ty_args.is_empty());
    debug_assert!(args.len() == 1);

    let data = pop_arg!(args, Vec<u8>);

    let cost = gas_params.base + (gas_params.per_byte * NumBytes::new(data.len() as u64));

    match bn128::run_pair(&data, 0, 0, 10) {
        Ok((_, pair)) => {
            let success = Value::vector_u8(pair.to_vec());
            Ok(NativeResult::ok(cost, smallvec![success]))
        }
        Err(_) => Ok(NativeResult::err(cost, E_EC_PAIRING_FAILED)),
    }
}

/***************************************************************************************************
 * native function `blake2f(rounds: vector<u8>, h: vector<u8>, m: vector<u8>, t: vector<u8>, f: vector<u8>): vector<u8>`
 **************************************************************************************************/
pub fn native_blake2f(
    gas_params: &FromBytesGasParameters,
    _context: &mut NativeContext,
    ty_args: Vec<Type>,
    mut args: VecDeque<Value>,
) -> PartialVMResult<NativeResult> {
    debug_assert!(ty_args.is_empty());
    debug_assert!(args.len() == 5);

    let f = pop_arg!(args, Vec<u8>);
    let t = pop_arg!(args, Vec<u8>);
    let m = pop_arg!(args, Vec<u8>);
    let h = pop_arg!(args, Vec<u8>);
    let rounds = pop_arg!(args, Vec<u8>);

    if rounds.len() != 4 || h.len() != 64 || m.len() != 128 || t.len() != 16 || f.len() != 1 {
        return Ok(NativeResult::err(0.into(), E_INVALID_INPUT_SIZE));
    }

    let cost = gas_params.base + (gas_params.per_byte * NumBytes::new(213_u64));

    let mut data: Vec<u8> = Vec::new();
    data.extend(rounds);
    data.extend(h);
    data.extend(m);
    data.extend(t);
    data.extend(f);

    match blake2::run(&data.into(), 100_000_000_000) {
        Ok((_, out)) => Ok(NativeResult::ok(
            cost,
            smallvec![Value::vector_u8(out.to_vec())],
        )),
        Err(_) => Ok(NativeResult::err(cost, E_EC_PAIRING_FAILED)),
    }
}

/***************************************************************************************************
 * native function `point_evaluation(versioned_hash: vector<u8>, x: vector<u8>, y: vector<u8>, commitment: vector<u8>, proof: vector<u8>): (vector<u8>, vector<u8>)`
 **************************************************************************************************/
pub fn native_point_evaluation(
    gas_params: &FromBytesGasParameters,
    _context: &mut NativeContext,
    ty_args: Vec<Type>,
    mut args: VecDeque<Value>,
) -> PartialVMResult<NativeResult> {
    debug_assert!(ty_args.is_empty());
    debug_assert!(args.len() == 5);

    let proof = pop_arg!(args, Vec<u8>);
    let commitment = pop_arg!(args, Vec<u8>);
    let y = pop_arg!(args, Vec<u8>);
    let x = pop_arg!(args, Vec<u8>);
    let versioned_hash = pop_arg!(args, Vec<u8>);

    if versioned_hash.len() != 32
        || x.len() != 32
        || y.len() != 32
        || commitment.len() != 48
        || proof.len() != 48
    {
        return Ok(NativeResult::err(0.into(), E_INVALID_INPUT_SIZE));
    }

    let cost = gas_params.base + (gas_params.per_byte * NumBytes::new(192_u64));

    let mut data: Vec<u8> = Vec::new();
    data.extend(versioned_hash);
    data.extend(x);
    data.extend(y);
    data.extend(commitment);
    data.extend(proof);
    let env = Env::default();

    match kzg_point_evaluation::run(&data.into(), 100_000_000_000, &env) {
        Ok((_, output)) => {
            let (output_f, output_b) = output.split_at(32);
            let output_f = Value::vector_u8(output_f.to_vec());
            let output_b = Value::vector_u8(output_b.to_vec());
            Ok(NativeResult::ok(cost, smallvec![output_f, output_b]))
        }

        Err(_) => Ok(NativeResult::err(cost, E_POINT_EVALUATION_FAILED)),
    }
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
    pub modexp: FromBytesGasParameters,
    pub ec_recover: FromBytesGasParameters,
    pub ec_add: FromBytesGasParameters,
    pub ec_mul: FromBytesGasParameters,
    pub ec_pairing: FromBytesGasParameters,
    pub blake2f: FromBytesGasParameters,
    pub point_evaluation: FromBytesGasParameters,
}

impl GasParameters {
    pub fn zeros() -> Self {
        Self {
            modexp: FromBytesGasParameters::zeros(),
            ec_recover: FromBytesGasParameters::zeros(),
            ec_add: FromBytesGasParameters::zeros(),
            ec_mul: FromBytesGasParameters::zeros(),
            ec_pairing: FromBytesGasParameters::zeros(),
            blake2f: FromBytesGasParameters::zeros(),
            point_evaluation: FromBytesGasParameters::zeros(),
        }
    }
}

pub fn make_all(gas_params: GasParameters) -> impl Iterator<Item = (String, NativeFunction)> {
    let natives = [
        ("modexp", make_native(gas_params.modexp, native_modexp)),
        (
            "ec_recover",
            make_native(gas_params.ec_recover, native_ec_recover),
        ),
        ("ec_add", make_native(gas_params.ec_add, native_ec_add)),
        ("ec_mul", make_native(gas_params.ec_mul, native_ec_mul)),
        (
            "ec_pairing",
            make_native(gas_params.ec_pairing, native_ec_pairing),
        ),
        ("blake2f", make_native(gas_params.blake2f, native_blake2f)),
        (
            "point_evaluation",
            make_native(gas_params.point_evaluation, native_point_evaluation),
        ),
    ];

    make_module_natives(natives)
}
