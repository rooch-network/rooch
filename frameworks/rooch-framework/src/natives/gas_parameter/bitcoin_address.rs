// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::natives::gas_parameter::native::MUL;
use crate::natives::rooch_framework::bitcoin_address::GasParameters;

crate::natives::gas_parameter::native::define_gas_parameters_for_natives!(GasParameters, "bitcoin_address", [
    [.verify_bitcoin_address_with_public_key.base, "verify_bitcoin_address_with_public_key.base", 1000 * MUL],
    [.verify_bitcoin_address_with_public_key.per_byte, "verify_bitcoin_address_with_public_key.per_byte", 30 * MUL],
    [.new.base, "parse.base", 1000 * MUL],
    [.new.per_byte, "parse.per_byte", 30 * MUL],
]);
