// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::natives::gas_parameter::native::MUL;
use moveos_stdlib::natives::moveos_stdlib::evm::GasParameters;

crate::natives::gas_parameter::native::define_gas_parameters_for_natives!(GasParameters, "evm", [
    [.modexp.base, "modexp.base", 1000 * MUL],
    [.modexp.per_byte, "modexp.per_byte", 30 * MUL],
    [.ec_recover.base, "ec_recover.base", 1000 * MUL],
    [.ec_recover.per_byte, "ec_recover.per_byte", 30 * MUL],
    [.ec_add.base, "ec_add.base", 1000 * MUL],
    [.ec_add.per_byte, "ec_add.per_byte", 30 * MUL],
    [.ec_mul.base, "ec_mul.base", 1000 * MUL],
    [.ec_mul.per_byte, "ec_mul.per_byte", 30 * MUL],
    [.ec_pairing.base, "ec_pairing.base", 1000 * MUL],
    [.ec_pairing.per_byte, "ec_pairing.per_byte", 30 * MUL],
    [.blake2f.base, "blake2f.base", 1000 * MUL],
    [.blake2f.per_byte, "blake2f.per_byte", 30 * MUL],
    [.point_evaluation.base, "point_evaluation.base", 1000 * MUL],
    [.point_evaluation.per_byte, "point_evaluation.per_byte", 30 * MUL],
]);
