// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::natives::gas_parameter::native::MUL;
use crate::natives::rooch_framework::bitcoin_address::GasParameters;

crate::natives::gas_parameter::native::define_gas_parameters_for_natives!(GasParameters, "bitcoin_address", [
    [.verify_with_pk.base, "verify_with_pk.base", 1000 * MUL],
    [.verify_with_pk.per_byte, "verify_with_pk.per_byte", 30 * MUL],
]);
