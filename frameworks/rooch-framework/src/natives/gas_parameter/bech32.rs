// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::natives::gas_parameter::native::MUL;
use moveos_stdlib::natives::moveos_stdlib::bech32::GasParameters;

crate::natives::gas_parameter::native::define_gas_parameters_for_natives!(GasParameters, "bech32", [
    [.encode.base, "encode.base", 1000 * MUL],
    [.encode.per_byte, "encode.per_byte", 30 * MUL],
    [.segwit_encode.base, "segwit_encode.base", 1000 * MUL],
    [.segwit_encode.per_byte, "segwit_encode.per_byte", 30 * MUL],
    [.decode.base, "decode.base", 1000 * MUL],
    [.decode.per_byte, "decode.per_byte", 30 * MUL],
    [.segwit_decode.base, "segwit_decode.base", 1000 * MUL],
    [.segwit_decode.per_byte, "segwit_decode.per_byte", 30 * MUL],
]);
