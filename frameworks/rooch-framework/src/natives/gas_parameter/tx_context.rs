// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::natives::gas_parameter::native::MUL;
use moveos_stdlib::natives::moveos_stdlib::tx_context::GasParameters;

crate::natives::gas_parameter::native::define_gas_parameters_for_natives!(GasParameters, "tx_context", [
    [.borrow_inner.base, "borrow_inner.base", 500 * MUL],
    [.borrow_mut_inner.base, "borrow_mut_inner.base", 1000 * MUL],
]);
