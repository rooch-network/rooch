// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::natives::gas_parameter::native::MUL;
use moveos_stdlib::natives::moveos_stdlib::object::ListFieldsGasParameters;

crate::natives::gas_parameter::native::define_gas_parameters_for_natives!(ListFieldsGasParameters, "native_list_field_keys", [
    [.list_field_keys.base, optional "list_field_keys.base", 1000 * MUL],
    [.list_field_keys.per_byte, optional "list_field_keys.per_byte", 150 * MUL],
]);
