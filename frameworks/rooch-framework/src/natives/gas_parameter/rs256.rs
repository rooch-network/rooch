// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::natives::rooch_framework::crypto::rs256::GasParameters;

crate::natives::gas_parameter::native::define_gas_parameters_for_natives!(GasParameters, "rs256", [
    [.verify.base, optional "verify.base", 0],
    [.verify.per_byte, optional "verify.per_byte", 0],
    [.verify_prehash.base, optional "verify_prehash.base", 0],
    [.verify_prehash.per_byte, optional "verify_prehash.per_byte", 0],
]);
