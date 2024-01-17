// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::natives::gas_parameter::native::MUL;
use crate::natives::rooch_framework::crypto::decoding::GasParameters;

crate::natives::gas_parameter::native::define_gas_parameters_for_natives!(GasParameters, "decoding", [
    [.base58.base, "base58.base", 1000 * MUL],
    [.base58.per_byte, "base58.per_byte", 5 * MUL],
    [.base58check.base, "base58check.base", 1000 * MUL],
    [.base58check.per_byte, "base58check.per_byte", 5 * MUL],
]);
