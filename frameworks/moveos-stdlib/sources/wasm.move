// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

module moveos_std::wasm {
    
    use std::option::{Self,Option};
    use moveos_std::features;

    struct WASMInstance {
        id: u64
    }

    public fun get_instance_id(instance: &WASMInstance): u64 {
        instance.id
    }

    public fun create_wasm_instance(bytecode: vector<u8>): WASMInstance {
        features::ensure_wasm_enabled();

        let (instance_id, error_code) = native_create_wasm_instance(bytecode);
        assert!(error_code == 0, error_code);

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

    #[test_only] 
    use std::vector;

    #[test]
    fun test_trap() {
      features::init_and_enable_all_features_for_test();

      let wasm_code: vector<u8> = b"(module (func (export \"div_s\") (param $x i32) (param $y i32) (result i32) (i32.div_s (local.get $x) (local.get $y))))";

      // 1. create wasm VM instance (required step)
      let wasm_instance = create_wasm_instance(wasm_code);
    //   std::debug::print(&std::std::string::utf8(b"wasm_instance:"));
    //   std::debug::print(&wasm_instance);

      // 2. run 10/0
      let function_name = b"div_s";
      let arg_list = vector::empty<u64>();
      vector::push_back(&mut arg_list, 10u64);
      vector::push_back(&mut arg_list, 0u64);
    //   std::debug::print(&arg_list);

      let ret_val_option = execute_wasm_function_option(&mut wasm_instance, function_name, arg_list);
      assert!(option::is_none(&ret_val_option), 1);

      // 3. release the wasm VM instance (required step)
      release_wasm_instance(wasm_instance);
    }

    #[test]
    fun test_unconditional_jump_loop() {
        // Enable all features for testing
        features::init_and_enable_all_features_for_test();

        // Define WASM code with an unconditional jump loop
        let wasm_code: vector<u8> = b"(module (func $run_forever (loop $loop (br $loop))) (export \"run_forever\" (func $run_forever)))";

        // 1. Create WASM VM instance (required step)
        let wasm_instance = create_wasm_instance(wasm_code);
        // std::debug::print(&std::string::utf8(b"wasm_instance:"));
        // std::debug::print(&wasm_instance);

        // 2. Execute the function that runs forever
        let function_name = b"run_forever";
        let arg_list = vector::empty<u64>();
        // std::debug::print(&std::string::utf8(b"arg_list:"));
        // std::debug::print(&arg_list);

        // Execute the function and check if it returns None (indicating an infinite loop)
        let ret_val_option = execute_wasm_function_option(&mut wasm_instance, function_name, arg_list);
        assert!(option::is_none(&ret_val_option), 1);

        // 3. Release the WASM VM instance (required step)
        release_wasm_instance(wasm_instance);
    }

    #[test]
    fun test_conditional_jump_loop() {
        // Enable all features for testing
        features::init_and_enable_all_features_for_test();

        // Define WASM code with a conditional jump loop that always evaluates to true
        let wasm_code: vector<u8> = b"(module (func $run_forever (loop $loop (i32.const 1) (br_if $loop))) (export \"run_forever\" (func $run_forever)))";

        // 1. Create WASM VM instance (required step)
        let wasm_instance = create_wasm_instance(wasm_code);
        // std::debug::print(&std::string::utf8(b"wasm_instance:"));
        // std::debug::print(&wasm_instance);

        // 2. Execute the function that runs forever
        let function_name = b"run_forever";
        let arg_list = vector::empty<u64>();
        // std::debug::print(&std::string::utf8(b"arg_list:"));
        // std::debug::print(&arg_list);

        // Execute the function and check if it returns None (indicating an infinite loop)
        let ret_val_option = execute_wasm_function_option(&mut wasm_instance, function_name, arg_list);
        assert!(option::is_none(&ret_val_option), 1);

        // 3. Release the WASM VM instance (required step)
        release_wasm_instance(wasm_instance);
    }

    #[test]
    fun test_recursive_call_loop() {
        // Enable all features for testing
        features::init_and_enable_all_features_for_test();

        // Define WASM code with a recursive call that never terminates
        let wasm_code: vector<u8> = b"(module (func $run_forever (call $run_forever)) (export \"run_forever\" (func $run_forever)))";

        // 1. Create WASM VM instance (required step)
        let wasm_instance = create_wasm_instance(wasm_code);
        // std::debug::print(&std::string::utf8(b"wasm_instance:"));
        // std::debug::print(&wasm_instance);

        // 2. Execute the function that runs forever
        let function_name = b"run_forever";
        let arg_list = vector::empty<u64>();
        // std::debug::print(&std::string::utf8(b"arg_list:"));
        // std::debug::print(&arg_list);

        // Execute the function and check if it returns None (indicating an infinite loop)
        let ret_val_option = execute_wasm_function_option(&mut wasm_instance, function_name, arg_list);
        assert!(option::is_none(&ret_val_option), 1);

        // 3. Release the WASM VM instance (required step)
        release_wasm_instance(wasm_instance);
    }

    #[test]
    fun test_counter_overflow_loop() {
        // Enable all features for testing
        features::init_and_enable_all_features_for_test();

        // Define WASM code with a counter overflow loop
        let wasm_code: vector<u8> = b"(module (func $run_forever (local $i i32) (loop $loop (local.set $i (i32.add (local.get $i) (i32.const 1))) (br_if $loop (i32.ne (local.get $i) (i32.const 0))))) (export \"run_forever\" (func $run_forever)))";

        // 1. Create WASM VM instance (required step)
        let wasm_instance = create_wasm_instance(wasm_code);
        // std::debug::print(&std::string::utf8(b"wasm_instance:"));
        // std::debug::print(&wasm_instance);

        // 2. Execute the function that runs forever
        let function_name = b"run_forever";
        let arg_list = vector::empty<u64>();
        // std::debug::print(&std::string::utf8(b"arg_list:"));
        // std::debug::print(&arg_list);

        // Execute the function and check if it returns None (indicating an infinite loop)
        let ret_val_option = execute_wasm_function_option(&mut wasm_instance, function_name, arg_list);
        assert!(option::is_none(&ret_val_option), 1);

        // 3. Release the WASM VM instance (required step)
        release_wasm_instance(wasm_instance);
    }

    #[test]
    fun test_always_true_condition_loop() {
        // Enable all features for testing
        features::init_and_enable_all_features_for_test();

        // Define WASM code with a loop that always evaluates to true
        let wasm_code: vector<u8> = b"(module (func $run_forever (loop $loop (if (i32.const 1) (then (br $loop))))) (export \"run_forever\" (func $run_forever)))";

        // 1. Create WASM VM instance (required step)
        let wasm_instance = create_wasm_instance(wasm_code);
        // std::debug::print(&std::string::utf8(b"wasm_instance:"));
        // std::debug::print(&wasm_instance);

        // 2. Execute the function that runs forever
        let function_name = b"run_forever";
        let arg_list = vector::empty<u64>();
        // std::debug::print(&std::string::utf8(b"arg_list:"));
        // std::debug::print(&arg_list);

        // Execute the function and check if it returns None (indicating an infinite loop)
        let ret_val_option = execute_wasm_function_option(&mut wasm_instance, function_name, arg_list);
        assert!(option::is_none(&ret_val_option), 1);

        // 3. Release the WASM VM instance (required step)
        release_wasm_instance(wasm_instance);
    }
}
