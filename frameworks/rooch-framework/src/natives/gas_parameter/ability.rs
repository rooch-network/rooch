// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use moveos_stdlib::natives::moveos_stdlib::ability::GasParameters;

crate::natives::gas_parameter::native::define_gas_parameters_for_natives!(GasParameters, "ability", [
    [.get_abilities.base, optional "decode.base", 0],
    [.get_abilities.per_byte_in_str, optional "decode.per_byte_in_str", 0],
]);
