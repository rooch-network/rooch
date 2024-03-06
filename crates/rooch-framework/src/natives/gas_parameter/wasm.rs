// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use moveos_stdlib::natives::moveos_stdlib::wasm::GasParameters;

crate::natives::gas_parameter::native::define_gas_parameters_for_natives!(GasParameters, "account", [
    [.create_instance_gas_parameter.base_create_instance, "create_instance_gas_parameter.base_create_instance", 10000],
    [.create_instance_gas_parameter.per_byte_instance, "create_instance_gas_parameter.per_byte_instance", 100],
    [.create_args_gas_parameter.base_create_args, "create_args_gas_parameter.base_create_args", 10000],
    [.create_args_gas_parameter.per_byte_args, "create_args_gas_parameter.per_byte_args", 100],
    [.function_execution_gas_parameter.base_create_execution, "function_execution_gas_parameter.base_create_execution", 1000],
    [.function_execution_gas_parameter.per_byte_execution_result, "function_execution_gas_parameter.per_byte_execution_result", 100],
]);