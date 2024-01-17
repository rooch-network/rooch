// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::natives::gas_parameter::native::MUL;
use crate::natives::rooch_framework::crypto::ed25519::GasParameters;

crate::natives::gas_parameter::native::define_gas_parameters_for_natives!(GasParameters, "ed25519", [
    [.verify.base, "verify.base", (5 + 1) * MUL],
    [.verify.per_byte, "verify.per_byte", (5 + 1) * MUL],
]);
