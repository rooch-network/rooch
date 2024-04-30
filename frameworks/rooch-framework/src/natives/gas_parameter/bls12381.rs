// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::natives::gas_parameter::native::MUL;
use moveos_stdlib::natives::moveos_stdlib::bls12381::GasParameters;

crate::natives::gas_parameter::native::define_gas_parameters_for_natives!(GasParameters, "bls12381", [
    [.bls12381_min_sig_verify.base, "bls12381_min_sig_verify.base", 1000 * MUL],
    [.bls12381_min_sig_verify.per_byte, "bls12381_min_sig_verify.per_byte", 30 * MUL],
    [.bls12381_min_pk_verify.base, "bls12381_min_pk_verify.base", 1000 * MUL],
    [.bls12381_min_pk_verify.per_byte, "bls12381_min_pk_verify.per_byte", 30 * MUL],
]);
