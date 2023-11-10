// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::natives::gas_parameter::native::MUL;
use crate::natives::rooch_framework::bcs_friend::GasParameters as RoochFrameGasParameters;
use moveos_stdlib::natives::moveos_stdlib::bcs_friend::GasParameters;

crate::natives::gas_parameter::native::define_gas_parameters_for_natives!(GasParameters, "bcs_friend", [
    [.from_bytes.base, "from_bytes.base", (5 + 1) * MUL],
]);

crate::natives::gas_parameter::native::define_gas_parameters_for_natives!(RoochFrameGasParameters, "bcs_friend", [
    [.from_bytes.base, "from_bytes.base", (5 + 1) * MUL],
]);
