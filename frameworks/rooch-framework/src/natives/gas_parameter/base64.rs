// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use moveos_stdlib::natives::moveos_stdlib::base64::GasParameters;

crate::natives::gas_parameter::native::define_gas_parameters_for_natives!(GasParameters, "base64", [
    [.encode.base, optional "encode.base", 0],
    [.encode.per_byte, optional "encode.per_byte", 0],
    [.decode.base, optional "decode.base", 0],
    [.decode.per_byte, optional "decode.per_byte", 0],
]);
