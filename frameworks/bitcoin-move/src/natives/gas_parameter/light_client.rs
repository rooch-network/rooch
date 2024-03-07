// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::natives::light_client::GasParameters;
use rooch_framework::natives::gas_parameter::native::MUL;

rooch_framework::natives::gas_parameter::native::define_gas_parameters_for_natives!(GasParameters, "light_client", [
    // TODO Can use gas free to avoid gas charge
    [.get_block.base, "get_block.base", (0 + 1) * MUL],
]);
