// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::natives::gas_parameter::native::MUL;
use moveos_stdlib::natives::moveos_stdlib::base58::GasParameters;

crate::natives::gas_parameter::native::define_gas_parameters_for_natives!(GasParameters, "base58", [
    [.encoding.base, "encoding.base", 1000 * MUL],
    [.encoding.per_byte, "encoding.per_byte", 30 * MUL],
    [.checksum_encoding.base, "checksum_encoding.base", 1000 * MUL],
    [.checksum_encoding.per_byte, "checksum_encoding.per_byte", 30 * MUL],
    [.decoding.base, "decoding.base", 1000 * MUL],
    [.decoding.per_byte, "decoding.per_byte", 30 * MUL],
    [.checksum_decoding.base, "checksum_decoding.base", 1000 * MUL],
    [.checksum_decoding.per_byte, "checksum_decoding.per_byte", 30 * MUL],
]);
