// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use rooch_framework::natives::gas_parameter::native::MUL;
use crate::natives::ord::GasParameters;

rooch_framework::natives::gas_parameter::native::define_gas_parameters_for_natives!(GasParameters, "ord", [
    [.from_witness.base, "from_witness.base", (5 + 1) * MUL],
]);
