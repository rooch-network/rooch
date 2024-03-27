// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

module bitcoin_move::bitseed {
    use std::string;
    use std::string::String;
    use std::vector;
    use moveos_std::string_utils::{parse_u64, parse_u8};
    use moveos_std::simple_map;
    use moveos_std::simple_map::SimpleMap;
    use moveos_std::wasm;

    const BIT_SEED_DEPLOY: vector<u8> = b"bitseed_deploy";
    const BIT_SEED_MINT: vector<u8> = b"bitseed_mint";

    struct DeployOp has store,copy,drop {
        is_valid: u8,
        from: address,
        to: address,
        tick: String,
        amount: u64,
        attributes_repeat: u8,
        attributes_generator: String,
        attributes_has_user_input: u8,
        attributes_deploy_args: vector<u8>
    }

    struct MintOp has store,copy,drop {
        is_valid: u8,
        from: address,
        to: address,
        tick: String,
        amount: u64,
        attributes: vector<u8>,
        content_type: vector<u8>,
        body: vector<u8>
    }

    public fun bitseed_deploy_key(): vector<u8> {
        BIT_SEED_DEPLOY
    }

    public fun bitseed_mint_key(): vector<u8> {
        BIT_SEED_MINT
    }

    public fun is_bitseed(json_map: &SimpleMap<String,String>) : bool {
        let protocol_key = string::utf8(b"p");
        simple_map::contains_key(json_map, &protocol_key) && simple_map::borrow(json_map, &protocol_key) == &string::utf8(b"bitseed")
    }

    public fun is_bitseed_deploy(json_map: &SimpleMap<String,String>) : bool {
        let op_key = string::utf8(b"op");
        let is_deploy_op = simple_map::contains_key(json_map, &op_key) && simple_map::borrow(json_map, &op_key) == &string::utf8(b"deploy");
        let attributes_generator = *simple_map::borrow(json_map, &string::utf8(b"generator"));
        let attributes_deploy_args = *simple_map::borrow(json_map, &string::utf8(b"deploy_args"));
        is_deploy_op && string::length(&attributes_generator) > 0 && string::length(&attributes_deploy_args) > 0
    }

    public fun is_bitseed_mint(json_map: &SimpleMap<String,String>) : bool {
        let op_key = string::utf8(b"op");
        let is_mint_op = simple_map::contains_key(json_map, &op_key) && simple_map::borrow(json_map, &op_key) == &string::utf8(b"mint");
        let attributes = *simple_map::borrow(json_map, &string::utf8(b"attributes"));
        is_mint_op && string::length(&attributes) > 0
    }

    public fun inscription_to_bitseed_deploy(from: address, to: address, json_map: &SimpleMap<String,String>) : DeployOp {
        let tick = *simple_map::borrow(json_map, &string::utf8(b"tick"));
        let amount_string = *simple_map::borrow(json_map, &string::utf8(b"amount"));
        let amount = parse_u64(&amount_string);
        let repeat_string = *simple_map::borrow(json_map, &string::utf8(b"repeat"));
        let repeat = parse_u8(&repeat_string);
        let generator_string = *simple_map::borrow(json_map, &string::utf8(b"generator"));
        let has_user_input_string = *simple_map::borrow(json_map, &string::utf8(b"has_user_input"));
        let has_user_input = parse_u8(&has_user_input_string);
        let deploy_args_string = *simple_map::borrow(json_map, &string::utf8(b"deploy_args"));
        DeployOp {
            is_valid: 0,
            from,
            to,
            tick,
            amount,
            attributes_repeat: repeat,
            attributes_generator: generator_string,
            attributes_has_user_input: has_user_input,
            attributes_deploy_args: string::into_bytes(deploy_args_string),
        }
    }

    public fun inscription_to_bitseed_mint(from: address, to: address, json_map: &SimpleMap<String,String>) : MintOp {
        let tick = *simple_map::borrow(json_map, &string::utf8(b"tick"));
        let amount_string = *simple_map::borrow(json_map, &string::utf8(b"amount"));
        let amount = parse_u64(&amount_string);
        let attributes_string = *simple_map::borrow(json_map, &string::utf8(b"attributes"));
        let content_type_string = *simple_map::borrow(json_map, &string::utf8(b"content_type"));
        let body_string = *simple_map::borrow(json_map, &string::utf8(b"body"));
        MintOp {
            is_valid: 0,
            from,
            to,
            tick,
            amount,
            attributes: string::into_bytes(attributes_string),
            content_type: string::into_bytes(content_type_string),
            body: string::into_bytes(body_string)
        }
    }

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

    public fun mint_op_is_valid(mint_op: &MintOp): u8 {
        mint_op.is_valid
    }

    public fun mint_op_attributes(mint_op: &MintOp): vector<u8> {
        mint_op.attributes
    }

    public fun deploy_op_generator(deploy_op: &DeployOp): String {
        deploy_op.attributes_generator
    }

    public fun deploy_op_args(deploy_op: &DeployOp): vector<u8> {
        deploy_op.attributes_deploy_args
    }

    fun pack_inscribe_generate_args(deploy_args: vector<u8>, seed: vector<u8>, user_input: vector<u8>): vector<u8>{
        native_pack_inscribe_generate_args(deploy_args, b"attrs", seed, b"seed",
            user_input, b"user_input")
    }

    native fun native_pack_inscribe_generate_args(
        deploy_args: vector<u8>, deploy_args_key: vector<u8>,
        seed: vector<u8>, seed_key: vector<u8>,
        user_input: vector<u8>, user_input_key: vector<u8>,
    ): vector<u8>;
}