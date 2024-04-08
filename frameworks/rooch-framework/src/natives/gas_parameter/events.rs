// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::natives::gas_parameter::native::MUL;
use moveos_stdlib::natives::moveos_stdlib::event::GasParameters;

crate::natives::gas_parameter::native::define_gas_parameters_for_natives!(GasParameters, "events", [
    [.emit.base, "emit.base", (52 + 1) * MUL],
    [.emit.per_byte_in_str, "emit.per_byte_in_str", (5 + 1) * MUL],
]);
