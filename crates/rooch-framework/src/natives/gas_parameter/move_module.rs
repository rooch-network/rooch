// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::natives::gas_parameter::native::MUL;
use moveos_stdlib::natives::moveos_stdlib::move_module::GasParameters;

crate::natives::gas_parameter::native::define_gas_parameters_for_natives!(GasParameters, "move_module", [
    [.module_name_inner.base, "module_name_inner.base", (5 + 1) * MUL],
    [.module_name_inner.per_byte_in_str, "module_name_inner.per_byte_in_str", (5 + 1) * MUL],
    [.sort_and_verify_modules_inner.base, "sort_and_verify_modules_inner.base", (5 + 1) * MUL],
    [.sort_and_verify_modules_inner.per_byte, "sort_and_verify_modules_inner.per_byte", (5 + 1) * MUL],
    [.request_init_functions.base, "request_init_functions.base", (5 + 1) * MUL],
    [.request_init_functions.per_byte, "request_init_functions.per_byte", (5 + 1) * MUL],
    [.check_compatibililty_inner.base, "check_compatibililty_inner.base", (5 + 1) * MUL],
    [.check_compatibililty_inner.per_byte, "check_compatibililty_inner.per_byte", (5 + 1) * MUL],
]);
