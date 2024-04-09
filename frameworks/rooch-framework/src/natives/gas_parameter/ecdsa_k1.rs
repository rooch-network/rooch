// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::natives::gas_parameter::native::MUL;
use crate::natives::rooch_framework::crypto::ecdsa_k1::GasParameters;

crate::natives::gas_parameter::native::define_gas_parameters_for_natives!(GasParameters, "ecdsa_k1", [
    [.ecrecover.base, "ecrecover.base", 1000 * MUL],
    [.decompress_pubkey.base, "decompress_pubkey.base", 1000 * MUL],
    [.verify.base, "verify.base", 1000 * MUL],
    [.verify.per_byte, "verify.per_byte", 30 * MUL],
]);
