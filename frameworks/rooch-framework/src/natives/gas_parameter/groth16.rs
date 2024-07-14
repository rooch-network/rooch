// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::natives::gas_parameter::native::MUL;
use moveos_stdlib::natives::moveos_stdlib::groth16::GasParameters;

crate::natives::gas_parameter::native::define_gas_parameters_for_natives!(GasParameters, "groth16", [
    [.prepare_verifying_key_internal.base, "prepare_verifying_key_internal.base", 1000 * MUL],
    [.prepare_verifying_key_internal.per_byte, "prepare_verifying_key_internal.per_byte", 30 * MUL],
    [.verify_groth16_proof_internal.base, "verify_groth16_proof_internal.base", 1000 * MUL],
    [.verify_groth16_proof_internal.per_byte, "verify_groth16_proof_internal.per_byte", 30 * MUL],
]);
