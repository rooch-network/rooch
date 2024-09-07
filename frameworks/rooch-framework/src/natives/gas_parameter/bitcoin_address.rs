// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::natives::gas_parameter::native::MUL;
use crate::natives::rooch_framework::bitcoin_address::GasParameters;

crate::natives::gas_parameter::native::define_gas_parameters_for_natives!(GasParameters, "bitcoin_address", [
    [.parse.base, "parse.base", 1000 * MUL],
    [.parse.per_byte, "parse.per_byte", 30 * MUL],
    [.verify_bitcoin_address_with_public_key.base, optional "verify_bitcoin_address_with_public_key.base", 1000 * MUL],
    [.verify_bitcoin_address_with_public_key.per_byte, optional "verify_bitcoin_address_with_public_key.per_byte", 30 * MUL],
    [.derive_bitcoin_taproot_address.base, optional "derive_bitcoin_taproot_address.base", 1000 * MUL],
    [.derive_bitcoin_taproot_address.per_byte, optional "derive_bitcoin_taproot_address.per_byte", 30 * MUL],
]);
