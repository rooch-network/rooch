// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::natives::rooch_framework::crypto::ecdsa_r1::GasParameters;

crate::natives::gas_parameter::native::define_gas_parameters_for_natives!(GasParameters, "ecdsa_r1", [
    [.verify.base, optional "verify.base", 0],
    [.verify.per_byte, optional "verify.per_byte", 0],
]);
