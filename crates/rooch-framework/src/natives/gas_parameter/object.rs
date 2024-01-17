// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::natives::gas_parameter::native::MUL;
use moveos_stdlib::natives::moveos_stdlib::object::GasParameters;

crate::natives::gas_parameter::native::define_gas_parameters_for_natives!(GasParameters, "object", [
    [.as_ref_inner.base, "as_ref_inner.base", 500 * MUL],
    [.as_mut_ref_inner.base, "as_mut_ref_inner.base", 1000 * MUL],
]);
