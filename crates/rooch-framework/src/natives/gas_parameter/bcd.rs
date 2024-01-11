// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::natives::gas_parameter::native::MUL;
use crate::natives::rooch_framework::bcs::GasParameters as RoochFrameGasParameters;
use moveos_stdlib::natives::moveos_stdlib::bcs::GasParameters;

crate::natives::gas_parameter::native::define_gas_parameters_for_natives!(GasParameters, "bcs", [
    [.from_bytes.base, "from_bytes.base", (5 + 1) * MUL],
    [.from_bytes.per_byte_deserialize, "from_bytes.per_byte_deserialize", (5 + 1) * MUL],
]);

crate::natives::gas_parameter::native::define_gas_parameters_for_natives!(RoochFrameGasParameters, "bcs", [
    [.from_bytes.base, "from_bytes.base", (5 + 1) * MUL],
    [.from_bytes.per_byte_deserialize, "from_bytes.per_byte_deserialize", (5 + 1) * MUL],
]);
