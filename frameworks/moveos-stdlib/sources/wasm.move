// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

module moveos_std::wasm {
    use std::string;
    use std::option::{Self,Option};
    use moveos_std::features;

    struct WASMInstance {
        id: u64
    }

    public fun get_instance_id(instance: &WASMInstance): u64 {
        instance.id
    }

    public fun create_wasm_instance(bytecode: vector<u8>): WASMInstance {
        std::debug::print(&string::utf8(b"create_wasm_instance debug 1"));
        features::ensure_wasm_enabled();
        std::debug::print(&string::utf8(b"create_wasm_instance debug 2"));

        let (instance_id, error_code) = native_create_wasm_instance(bytecode);
        std::debug::print(&string::utf8(b"create_wasm_instance debug 3"));
        std::debug::print(&error_code);
        
        if (error_code > 0) {
            std::debug::print(&string::utf8(b"create_wasm_instance_error:"));
            std::debug::print(&error_code);
        };

        assert!(error_code == 0, error_code);
        std::debug::print(&string::utf8(b"create_wasm_instance debug 4"));

        WASMInstance {id: instance_id }
    }

    public fun create_wasm_instance_option(bytecode: vector<u8>): Option<WASMInstance> {
        features::ensure_wasm_enabled();

        let (instance_id, error_code) = native_create_wasm_instance(bytecode);
        if (error_code > 0) {
            return option::none()
        };

        option::some(WASMInstance {id: instance_id })
    }

    public fun create_cbor_values(value: vector<vector<u8>>): vector<u8> {
        native_create_cbor_values(value)
    }

    public fun add_length_with_data(value: vector<u8>): vector<u8> {
        native_add_length_with_data(value)
    }

    public fun create_memory_wasm_args(instance: &mut WASMInstance, func_name: vector<u8>, args: vector<vector<u8>>): vector<u64> {
        native_create_wasm_args_in_memory(instance.id, func_name, args)
    }

    public fun execute_wasm_function(instance: &mut WASMInstance, func_name: vector<u8>, args: vector<u64>): u64 {
        features::ensure_wasm_enabled();

        let (ret_val, error_code) = native_execute_wasm_function(instance.id, func_name, args);
        assert!(error_code == 0, error_code);

        ret_val
    }

    public fun execute_wasm_function_option(instance: &mut WASMInstance, func_name: vector<u8>, args: vector<u64>): Option<u64> {
        features::ensure_wasm_enabled();
        
        let (ret_val, error_code) = native_execute_wasm_function(instance.id, func_name, args);
        if (error_code > 0) {
            return option::none()
        };

        option::some(ret_val)
    }

    public fun read_data_length(instance: &WASMInstance, data_ptr: u64): u32 {
        native_read_data_length(instance.id, data_ptr)
    }

    public fun read_data_from_heap(instance: &WASMInstance, data_ptr: u32, data_length: u32): vector<u8> {
        native_read_data_from_heap(instance.id, data_ptr, data_length)
    }

    public fun release_wasm_instance(instance: WASMInstance): bool {
        native_release_wasm_instance(instance)
    }

    native fun native_create_wasm_instance(bytecodes: vector<u8>): (u64, u64);

    native fun native_create_cbor_values(value: vector<vector<u8>>): vector<u8>;

    native fun native_add_length_with_data(value: vector<u8>): vector<u8>;

    native fun native_create_wasm_args_in_memory(instance_id: u64, func_name: vector<u8>, args_bytes: vector<vector<u8>>): vector<u64>;

    native fun native_execute_wasm_function(instance_id: u64, func_name: vector<u8>, args: vector<u64>): (u64, u64);

    native fun native_read_data_length(instance_id: u64, data_ptr: u64): u32;

    native fun native_read_data_from_heap(instance_id: u64, data_ptr: u32, data_length: u32): vector<u8>;

    native fun native_release_wasm_instance(instance: WASMInstance): bool;
}
