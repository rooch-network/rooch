module moveos_std::wasm {
    use moveos_std::context::Context;

    public fun create_wasm_instance(_ctx: &Context, bytecode: vector<u8>): u64 {
        native_create_wasm_instance(bytecode)
    }

    public fun create_memory_wasm_args(instance_id: u64, func_name: vector<u8>, args: vector<vector<u8>>): vector<u64> {
        native_create_wasm_args_in_memory(instance_id, func_name, args)
    }

    public fun execute_wasm_function(instance_id: u64, func_name: vector<u8>, args: vector<u64>): u64 {
        native_execute_wasm_function(instance_id, func_name, args)
    }

    native fun native_create_wasm_instance(bytecodes: vector<u8>): u64;

    // native func native_create_cbor_values(value: vector<u8>): vector<u8>;

    native fun native_create_wasm_args_in_memory(instance_id: u64, func_name: vector<u8>, args_bytes: vector<vector<u8>>): vector<u64>;

    native fun native_execute_wasm_function(instance_id: u64, func_name: vector<u8>, args: vector<u64>): u64;

    // native native_read_data_from_heap(instance_id: u64, data_ptr: u32): vector<u8>;

    // native native_release_wasm_instance(instance_id: u64);
}
