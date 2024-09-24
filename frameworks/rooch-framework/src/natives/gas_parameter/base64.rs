// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::natives::gas_parameter::native::MUL;
use moveos_stdlib::natives::moveos_stdlib::base64::GasParameters;

crate::natives::gas_parameter::native::define_gas_parameters_for_natives!(GasParameters, "base64", [
    [.encode.base, optional "encode.base", 1000 * MUL],
    [.encode.per_byte, optional "encode.per_byte", 30 * MUL],
    [.decode.base, optional "decode.base", 1000 * MUL],
    [.decode.per_byte, optional "decode.per_byte", 30 * MUL],
]);
