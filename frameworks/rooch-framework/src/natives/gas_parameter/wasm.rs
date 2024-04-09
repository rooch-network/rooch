// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use moveos_stdlib::natives::moveos_stdlib::wasm::GasParameters;

crate::natives::gas_parameter::native::define_gas_parameters_for_natives!(GasParameters, "wasm", [
    [.create_instance_gas_parameter.base_create_instance, "create_instance_gas_parameter.base_create_instance", 10000],
    [.create_instance_gas_parameter.per_byte_instance, "create_instance_gas_parameter.per_byte_instance", 100],

    [.create_cbor_value_gas_parameter.base, "create_cbor_value_gas_parameter.base", 100],
    [.create_cbor_value_gas_parameter.per_byte, "create_cbor_value_gas_parameter.base", 10000],

    [.add_length_with_data.base, "add_length_with_data.base", 100],
    [.add_length_with_data.per_byte, "add_length_with_data.per_byte", 10000],

    [.create_args_gas_parameter.base_create_args, "create_args_gas_parameter.base_create_args", 10000],
    [.create_args_gas_parameter.per_byte_args, "create_args_gas_parameter.per_byte_args", 100],

    [.function_execution_gas_parameter.base_create_execution, "function_execution_gas_parameter.base_create_execution", 1000],
    [.function_execution_gas_parameter.per_byte_execution_result, "function_execution_gas_parameter.per_byte_execution_result", 100],

    [.read_data_length_gas_parameter.base, "read_data_length_gas_parameter.base", 100],
    [.read_data_length_gas_parameter.per_byte, "read_data_length_gas_parameter.base", 10000],

    [.read_heap_data.base, "read_heap_data.base", 100],
    [.read_heap_data.per_byte, "read_heap_data.per_byte", 10000],

    [.release_wasm_instance.base, "release_wasm_instance.base", 100],
]);
