// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::natives::gas_parameter::native::MUL;
use crate::natives::rooch_framework::crypto::ecdsa_k1_recoverable::GasParameters;

crate::natives::gas_parameter::native::define_gas_parameters_for_natives!(GasParameters, "ecdsa_k1_recoverable", [
    [.verify.base, "verify.base", (5 + 1) * MUL],
    [.ecrecover.base, "ecrecover.base", (5 + 1) * MUL],
    [.decompress_pubkey.base, "decompress_pubkey.base", (5 + 1) * MUL],
]);
