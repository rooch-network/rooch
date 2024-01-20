// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::natives::gas_parameter::native::MUL;
use moveos_stdlib::natives::moveos_stdlib::type_info::GasParameters;

crate::natives::gas_parameter::native::define_gas_parameters_for_natives!(GasParameters, "type_info", [
    [.type_of.base, "type_of.base", 1000 * MUL],
    [.type_of.per_byte_in_str, "type_of.per_byte_in_str", 20 * MUL],
]);
