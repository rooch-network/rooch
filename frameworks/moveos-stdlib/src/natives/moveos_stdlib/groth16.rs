// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use move_binary_format::errors::PartialVMResult;
use move_core_types::gas_algebra::{InternalGas, InternalGasPerByte, NumBytes};
use move_vm_runtime::native_functions::{NativeContext, NativeFunction};
use move_vm_types::{
    loaded_data::runtime_types::Type,
    natives::function::NativeResult,
    pop_arg,
    values::{self, Value, VectorRef},
};
use smallvec::smallvec;
use std::collections::VecDeque;

use crate::natives::helpers::{make_module_natives, make_native};

pub const E_INVALID_CURVE: u64 = 0;
pub const E_INVALID_VERIFYING_KEY: u64 = 1;
pub const E_TOO_MANY_PUBLIC_INPUTS: u64 = 2;

// These must match the corresponding values in sui::groth16::Curve.
pub const BLS12381: u8 = 0;
pub const BN254: u8 = 1;

// We need to set an upper bound on the number of public inputs to avoid a DoS attack
pub const MAX_PUBLIC_INPUTS: usize = 8;

/***************************************************************************************************
 * native fun prepare_verifying_key_internal
 * Implementation of the Move native function `prepare_verifying_key_internal(curve: u8, verifying_key: &vector<u8>): PreparedVerifyingKey`
 * This function has two cost modes depending on the curve being set to `BLS12381` or `BN254`. The core formula is same but constants differ.
 * If curve = 0, we use the `bls12381` cost constants, otherwise we use the `bn254` cost constants.
 *   gas cost: groth16_prepare_verifying_key_cost_base                    | covers various fixed costs in the oper
 * Note: `curve` and `verifying_key` are fixed size, so their costs are included in the base cost.
 **************************************************************************************************/
pub fn native_prepare_verifying_key_internal(
    gas_params: &FromBytesGasParameters,
    _context: &mut NativeContext,
    ty_args: Vec<Type>,
    mut args: VecDeque<Value>,
) -> PartialVMResult<NativeResult> {
    debug_assert!(ty_args.is_empty());
    debug_assert!(args.len() == 2);

    let verifying_key = pop_arg!(args, VectorRef);
    let curve = pop_arg!(args, u8);

    let verifying_key_bytes_ref = verifying_key.as_bytes_ref();

    let base_cost = match curve {
        BLS12381 => {
            52 // TODO: use variable?
        }
        BN254 => {
            52 // TODO: use variable?
        }
        _ => {
            return Ok(NativeResult::err(gas_params.base, E_INVALID_CURVE));
        }
    };

    let result;
    if curve == BLS12381 {
        result = fastcrypto_zkp::bls12381::api::prepare_pvk_bytes(&verifying_key_bytes_ref);
    } else if curve == BN254 {
        result = fastcrypto_zkp::bn254::api::prepare_pvk_bytes(&verifying_key_bytes_ref);
    } else {
        return Ok(NativeResult::err(base_cost.into(), E_INVALID_CURVE));
    }

    match result {
        Ok(pvk) => Ok(NativeResult::ok(
            base_cost.into(),
            smallvec![Value::struct_(values::Struct::pack(vec![
                Value::vector_u8(pvk[0].to_vec()),
                Value::vector_u8(pvk[1].to_vec()),
                Value::vector_u8(pvk[2].to_vec()),
                Value::vector_u8(pvk[3].to_vec())
            ]))],
        )),
        Err(_) => Ok(NativeResult::err(base_cost.into(), E_INVALID_VERIFYING_KEY)),
    }
}

/***************************************************************************************************
 * native fun verify_groth16_proof_internal
 * Implementation of the Move native function `verify_groth16_proof_internal(curve: u8, vk_gamma_abc_g1_bytes: &vector<u8>,
 *                          alpha_g1_beta_g2_bytes: &vector<u8>, gamma_g2_neg_pc_bytes: &vector<u8>, delta_g2_neg_pc_bytes: &vector<u8>,
 *                          public_proof_inputs: &vector<u8>, proof_points: &vector<u8>): bool`
 *
 * This function has two cost modes depending on the curve being set to `BLS12381` or `BN254`. The core formula is same but constants differ.
 * If curve = 0, we use the `bls12381` cost constants, otherwise we use the `bn254` cost constants.
 *   gas cost: groth16_prepare_verifying_key_cost_base                    | covers various fixed costs in the oper
 *              + groth16_verify_groth16_proof_internal_public_input_cost_per_byte
 *                                                   * size_of(public_proof_inputs) | covers the cost of verifying each public input per byte
 *              + groth16_verify_groth16_proof_internal_cost_per_public_input
 *                                                   * num_public_inputs) | covers the cost of verifying each public input per input
 * Note: every other arg is fixed size, so their costs are included in the base cost.
 **************************************************************************************************/
pub fn native_verify_groth16_proof_internal(
    gas_params: &FromBytesGasParameters,
    _context: &mut NativeContext,
    ty_args: Vec<Type>,
    mut args: VecDeque<Value>,
) -> PartialVMResult<NativeResult> {
    debug_assert!(ty_args.is_empty());
    debug_assert!(args.len() == 7);

    let proof_points = pop_arg!(args, VectorRef);
    let public_proof_inputs = pop_arg!(args, VectorRef);
    let delta_g2_neg_pc = pop_arg!(args, VectorRef);
    let gamma_g2_neg_pc = pop_arg!(args, VectorRef);
    let alpha_g1_beta_g2 = pop_arg!(args, VectorRef);
    let vk_gamma_abc_g1 = pop_arg!(args, VectorRef);
    let curve = pop_arg!(args, u8);

    let proof_points_bytes_ref = proof_points.as_bytes_ref();
    let public_proof_inputs_bytes_ref = public_proof_inputs.as_bytes_ref();
    let delta_g2_neg_pc_bytes_ref = delta_g2_neg_pc.as_bytes_ref();
    let gamma_g2_neg_pc_bytes_ref = gamma_g2_neg_pc.as_bytes_ref();
    let alpha_g1_beta_g2_bytes_ref = alpha_g1_beta_g2.as_bytes_ref();
    let vk_gamma_abc_g1_bytes_ref = vk_gamma_abc_g1.as_bytes_ref();

    let (base_cost, cost_per_public_input, num_public_inputs) = match curve {
        BLS12381 => (
            52, // TODO: use variable?
            2,  // TODO: use variable?
            (public_proof_inputs_bytes_ref.len()
                + fastcrypto_zkp::bls12381::conversions::SCALAR_SIZE
                - 1)
                / fastcrypto_zkp::bls12381::conversions::SCALAR_SIZE,
        ),
        BN254 => (
            52, // TODO: use variable?
            2,  // TODO: use variable?
            (public_proof_inputs_bytes_ref.len() + fastcrypto_zkp::bn254::api::SCALAR_SIZE - 1)
                / fastcrypto_zkp::bn254::api::SCALAR_SIZE,
        ),
        _ => {
            return Ok(NativeResult::err(gas_params.base, E_INVALID_CURVE));
        }
    };

    let cost = (gas_params.per_byte * NumBytes::new(public_proof_inputs_bytes_ref.len() as u64))
        + (cost_per_public_input * num_public_inputs as u64).into()
        + base_cost.into();

    let result;
    if curve == BLS12381 {
        if public_proof_inputs_bytes_ref.len()
            > fastcrypto_zkp::bls12381::conversions::SCALAR_SIZE * MAX_PUBLIC_INPUTS
        {
            return Ok(NativeResult::err(cost, E_TOO_MANY_PUBLIC_INPUTS));
        }
        result = fastcrypto_zkp::bls12381::api::verify_groth16_in_bytes(
            &vk_gamma_abc_g1_bytes_ref,
            &alpha_g1_beta_g2_bytes_ref,
            &gamma_g2_neg_pc_bytes_ref,
            &delta_g2_neg_pc_bytes_ref,
            &public_proof_inputs_bytes_ref,
            &proof_points_bytes_ref,
        );
    } else if curve == BN254 {
        if public_proof_inputs_bytes_ref.len()
            > fastcrypto_zkp::bn254::api::SCALAR_SIZE * MAX_PUBLIC_INPUTS
        {
            return Ok(NativeResult::err(cost, E_TOO_MANY_PUBLIC_INPUTS));
        }
        result = fastcrypto_zkp::bn254::api::verify_groth16_in_bytes(
            &vk_gamma_abc_g1_bytes_ref,
            &alpha_g1_beta_g2_bytes_ref,
            &gamma_g2_neg_pc_bytes_ref,
            &delta_g2_neg_pc_bytes_ref,
            &public_proof_inputs_bytes_ref,
            &proof_points_bytes_ref,
        );
    } else {
        return Ok(NativeResult::err(cost, E_INVALID_CURVE));
    }

    Ok(NativeResult::ok(
        cost,
        smallvec![Value::bool(result.unwrap_or(false))],
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
    pub prepare_verifying_key_internal: FromBytesGasParameters,
    pub verify_groth16_proof_internal: FromBytesGasParameters,
}

impl GasParameters {
    pub fn zeros() -> Self {
        Self {
            prepare_verifying_key_internal: FromBytesGasParameters::zeros(),
            verify_groth16_proof_internal: FromBytesGasParameters::zeros(),
        }
    }
}

pub fn make_all(gas_params: GasParameters) -> impl Iterator<Item = (String, NativeFunction)> {
    let natives = [
        (
            "prepare_verifying_key_internal",
            make_native(
                gas_params.prepare_verifying_key_internal,
                native_prepare_verifying_key_internal,
            ),
        ),
        (
            "verify_groth16_proof_internal",
            make_native(
                gas_params.verify_groth16_proof_internal,
                native_verify_groth16_proof_internal,
            ),
        ),
    ];
    make_module_natives(natives)
}
