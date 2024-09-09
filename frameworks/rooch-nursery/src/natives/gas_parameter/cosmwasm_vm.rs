// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::natives::cosmwasm_vm::GasParameters;

rooch_framework::natives::gas_parameter::native::define_gas_parameters_for_natives!(GasParameters, "cosmwasm_vm", [
    [.create_instance.base, "create_instance.base", 10000],
    [.destroy_instance.base, "destroy_instance.base", 10000],
]);
