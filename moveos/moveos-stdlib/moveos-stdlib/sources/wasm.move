module moveos_std::wasm {
    use moveos_std::context::Context;
    use std::string;

    public fun create_wasm_instance(_ctx: &Context, bytecode: vector<u8>): u64 {
        native_create_wasm_instance(bytecode)
    }

    public fun create_wasm_args(instance_id: u64, func_name: vector<u8>, args: vector<vector<u8>>): vector<u32> {
        native_create_wasm_args_in_memory(instance_id, func_name, args)
    }

    public fun execute_wasm_instance(_ctx: &Context, instance_id: u64, func_name: string::String, args: vector<u32>): bool {
        native_execute_wasm_function(instance_id, func_name, args)
    }

    native fun native_create_wasm_instance(bytecodes: vector<u8>): u64;

    native fun native_create_wasm_args_in_memory(instance_id: u64, func_name: vector<u8>, args_bytes: vector<vector<u8>>): vector<u32>;

    native fun native_execute_wasm_function(instance_id: u64, func_name: string::String, args: vector<u32>): bool;
}
