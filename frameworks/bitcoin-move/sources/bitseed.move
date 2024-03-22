// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

module bitcoin_move::bitseed {
    use std::vector;
    use bitcoin_move::ord::pack_inscribe_generate_args;
    use moveos_std::wasm;

    public fun inscribe_generate(wasm_bytes: vector<u8>, deploy_args: vector<u8>,
                                 seed: vector<u8>, user_input: vector<u8>): vector<u8> {
        let wasm_instance = wasm::create_wasm_instance(wasm_bytes);
        let wasm_instance_id = wasm::get_instance_id(&wasm_instance);

        let function_name = b"inscribe_generate";

        let arg = pack_inscribe_generate_args(deploy_args, seed, user_input);
        let arg_with_length = wasm::add_length_with_data(arg);

        let arg_list = vector::empty<vector<u8>>();
        vector::push_back(&mut arg_list, arg_with_length);
        let memory_args_list = wasm::create_memory_wasm_args(wasm_instance_id, function_name, arg_list);

        let ret_val = wasm::execute_wasm_function(wasm_instance_id, function_name, memory_args_list);

        let ret_data_length = wasm::read_data_length(wasm_instance_id, ret_val);
        let ret_data = wasm::read_data_from_heap(wasm_instance_id, (ret_val as u32) + 4, ret_data_length);

        wasm::release_wasm_instance(wasm_instance);
        ret_data
    }
}