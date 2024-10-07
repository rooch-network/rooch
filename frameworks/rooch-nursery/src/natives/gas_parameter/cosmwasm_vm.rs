// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::natives::cosmwasm_vm::GasParameters;
use rooch_framework::natives::gas_parameter::native::MUL;

rooch_framework::natives::gas_parameter::native::define_gas_parameters_for_natives!(GasParameters, "cosmwasm_vm", [
    [.common.load_base, optional "common.load_base", 1000 * MUL],
    [.common.load_per_byte, optional "common.load_per_byte", 30 * MUL],
    [.common.load_failure, optional "common.load_failure", 300 * MUL],
    [.native_create_instance.base, optional "native_create_instance.base", 1000 * MUL],
    [.native_create_instance.per_byte_wasm, optional "native_create_instance.per_byte_wasm", 30 * MUL],
    [.native_destroy_instance.base, optional "native_destroy_instance.base", 1000 * MUL],
]);
