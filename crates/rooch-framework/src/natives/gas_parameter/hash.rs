// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::natives::gas_parameter::native::MUL;
use crate::natives::rooch_framework::crypto::hash::GasParameters;

crate::natives::gas_parameter::native::define_gas_parameters_for_natives!(GasParameters, "hash", [
    [.keccak256.base, "keccak256.base", 1000 * MUL],
    [.keccak256.per_byte, "keccak256.per_byte", 30 * MUL],
    [.blake2b256.base, "blake2b256.base", 1000 * MUL],
    [.blake2b256.per_byte, "blake2b256.per_byte", 30 * MUL],
    [.ripemd160.base, "ripemd160.base", 1000 * MUL],
    [.ripemd160.per_byte, "ripemd160.per_byte", 30 * MUL],
]);
