// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::natives::gas_parameter::native::MUL;
use moveos_stdlib::natives::moveos_stdlib::bcs::GasParameters;

crate::natives::gas_parameter::native::define_gas_parameters_for_natives!(GasParameters, "bcs", [
    [.from_bytes.base, "from_bytes.base", 1000 * MUL],
    [.from_bytes.per_byte_deserialize, "from_bytes.per_byte_deserialize", 20 * MUL],
]);
