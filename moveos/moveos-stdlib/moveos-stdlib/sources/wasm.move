module moveos_std::wasm {
    use moveos_std::context::Context;

    public fun create_wasm_instance(_ctx: &Context, bytecode: vector<u8>): u64 {
        native_create_wasm_instance(bytecode)
    }

    native fun native_create_wasm_instance(bytecodes: vector<u8>): u64;
}
