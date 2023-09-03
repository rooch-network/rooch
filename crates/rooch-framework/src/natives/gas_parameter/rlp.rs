// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::natives::gas_parameter::native::MUL;
use moveos_stdlib::natives::moveos_stdlib::rlp::GasParameters;

crate::natives::gas_parameter::native::define_gas_parameters_for_natives!(GasParameters, "rlp", [
    [.to_bytes.base, "to_bytes.base", (5 + 1) * MUL],
    [.from_bytes.base, "from_bytes.base", (5 + 1) * MUL],
]);
