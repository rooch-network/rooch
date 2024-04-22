// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

module rooch_nursery::bitseed {
    use std::string;
    use std::string::String;
    use std::vector;
    use std::option;

    use moveos_std::object::{Self, ObjectID};
    use moveos_std::string_utils::{parse_u64, parse_u8};
    use moveos_std::simple_map;
    use moveos_std::simple_map::SimpleMap;
    use moveos_std::wasm;
    use moveos_std::table::{Self, Table};

    use bitcoin_move::types::{Self, Transaction};
    use bitcoin_move::utxo;
    use bitcoin_move::ord::{Self, Inscription};

    const BIT_SEED_DEPLOY: vector<u8> = b"bitseed_deploy";
    const BIT_SEED_MINT: vector<u8> = b"bitseed_mint";

    friend bitcoin_move::genesis;

    struct Bitseed has key {}

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

    struct BitseedCoinInfo has store, copy{
        tick: String,
        max: u256,
        lim: u256,
        dec: u64,
        supply: u256,
    }

    struct BitseedStore has key{
        coins: Table<String, BitseedCoinInfo>,
    }

    public(friend) fun genesis_init(_genesis_account: &signer){
        let bitseed_store = BitseedStore{
            coins: table::new(),
        };

        let obj = object::new_named_object(bitseed_store);
        object::to_shared(obj);
    }

    public fun bitseed_deploy_key(): vector<u8> {
        BIT_SEED_DEPLOY
    }

    public fun bitseed_mint_key(): vector<u8> {
        BIT_SEED_MINT
    }

    public fun is_bitseed(inscription: &Inscription) : bool {
        let metaprotocol = ord::metaprotocol(inscription);
        option::is_some<String>(&metaprotocol) && option::borrow(&metaprotocol) == &string::utf8(b"bitseed")
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

        let function_name = b"inscribe_generate";

        let arg = pack_inscribe_generate_args(deploy_args, seed, user_input);
        let arg_with_length = wasm::add_length_with_data(arg);

        let arg_list = vector::empty<vector<u8>>();
        vector::push_back(&mut arg_list, arg_with_length);
        let memory_args_list = wasm::create_memory_wasm_args(&mut wasm_instance, function_name, arg_list);

        let ret_val = wasm::execute_wasm_function(&mut wasm_instance, function_name, memory_args_list);

        let ret_data_length = wasm::read_data_length(&wasm_instance, ret_val);
        let ret_data = wasm::read_data_from_heap(&wasm_instance, (ret_val as u32) + 4, ret_data_length);

        wasm::release_wasm_instance(wasm_instance);
        ret_data
    }

    public fun inscribe_verify(wasm_bytes: vector<u8>, deploy_args: vector<u8>,
                               seed: vector<u8>, user_input: vector<u8>, attributes_output: vector<u8>): bool {
        let wasm_instance = wasm::create_wasm_instance(wasm_bytes);

        let function_name = b"inscribe_verify";

        let buffer = pack_inscribe_generate_args(deploy_args, seed, user_input);
        let arg_with_length = wasm::add_length_with_data(buffer);

        let arg_list = vector::empty<vector<u8>>();
        vector::push_back(&mut arg_list, arg_with_length);
        vector::push_back(&mut arg_list, attributes_output);
        let memory_args_list = wasm::create_memory_wasm_args(&mut wasm_instance, function_name, arg_list);

        let ret_val = wasm::execute_wasm_function(&mut wasm_instance, function_name, memory_args_list);

        wasm::release_wasm_instance(wasm_instance);
        if (ret_val == 0 ) {
            true
        } else {
            false
        }
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

    fun pack_inscribe_generate_args(_deploy_args: vector<u8>, _seed: vector<u8>, _user_input: vector<u8>): vector<u8>{
        //TODO
        abort 0
    }

    native fun native_pack_inscribe_generate_args(
        deploy_args: vector<u8>, deploy_args_key: vector<u8>,
        seed: vector<u8>, seed_key: vector<u8>,
        user_input: vector<u8>, user_input_key: vector<u8>,
    ): vector<u8>;


    // ==== Process Bitseed ==== //
    public fun process(tx: &Transaction) {
        let txid = types::tx_id(tx);
        let txoutput = types::tx_output(tx);
        let idx = 0;
        let txoutput_len = vector::length(txoutput);
        while(idx < txoutput_len){
            let vout = (idx as u32);
            let output_point = types::new_outpoint(txid, vout);
            let utxo_obj = utxo::borrow_utxo(output_point);
            let utxo = object::borrow(utxo_obj);
            let seals = utxo::get_seals<Inscription>(utxo);

            // Track the Inscription via SatPoint
            let j = 0;
            let seals_len = vector::length<ObjectID>(&seals);
            while(j < seals_len){
                let seal_object_id = *vector::borrow(&seals, j);
                let inscription_obj = object::borrow_object<Inscription>(seal_object_id);
                let inscription = object::borrow(inscription_obj);
                process_inscription(tx, inscription);

                j = j + 1;
            };

            idx = idx + 1;
        };
    }

    public fun process_inscription(tx: &Transaction, inscription: &Inscription) {
        let txid = types::tx_id(tx);
        let index = ord::index(inscription);
        let inscription_id = ord::new_inscription_id(txid, index);

        if (is_bitseed(inscription)) {
            // TODO parse inscription.meta and valid op
            ord::seal_metaprotocol_validity<Bitseed>(inscription_id, true, option::none());
        };
    }
}
