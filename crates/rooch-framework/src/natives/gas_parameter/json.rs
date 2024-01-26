// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::natives::gas_parameter::native::MUL;
use moveos_stdlib::natives::moveos_stdlib::json::GasParameters;

crate::natives::gas_parameter::native::define_gas_parameters_for_natives!(GasParameters, "json", [
    [.from_bytes.base, "from_bytes.base", 1000 * MUL],
    [.from_bytes.per_byte_in_str, "from_bytes.per_byte_in_str", 20 * MUL],
]);
