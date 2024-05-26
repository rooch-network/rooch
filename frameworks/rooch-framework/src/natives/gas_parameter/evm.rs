// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::natives::gas_parameter::native::MUL;
use moveos_stdlib::natives::moveos_stdlib::evm::GasParameters;

crate::natives::gas_parameter::native::define_gas_parameters_for_natives!(GasParameters, "evm", [
    [.ec_add.base, "ec_add.base", 1000 * MUL],
    [.ec_add.per_byte, "ec_add.per_byte", 30 * MUL],
    [.ec_pairing.base, "ec_pairing.base", 1000 * MUL],
    [.ec_pairing.per_byte, "ec_pairing.per_byte", 30 * MUL],
]);
