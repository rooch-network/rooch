// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::natives::cosmwasm_vm::GasParameters;

rooch_framework::natives::gas_parameter::native::define_gas_parameters_for_natives!(GasParameters, "cosmwasm_vm", [
    [.native_create_instance.base, "native_create_instance.base", 10000],
    [.native_destroy_instance.base, "native_destroy_instance.base", 10000],
]);
