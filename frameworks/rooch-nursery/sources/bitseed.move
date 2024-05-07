// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

<<<<<<< HEAD:frameworks/rooch-nursery/sources/bitseed.move
module rooch_nursery::bitseed {
    use std::string;
    use std::string::String;
=======
module bitcoin_move::bitseed {
>>>>>>> 5e2be180... feat: check SFT valid:frameworks/bitcoin-move/sources/bitseed.move
    use std::vector;
    use std::option::{Self, Option};
    use std::string::{Self, String};

    use moveos_std::object::{Self, ObjectID};
    use moveos_std::string_utils::{parse_u64, parse_u8};
    use moveos_std::simple_map::{Self, SimpleMap};
    use moveos_std::wasm;
    use moveos_std::table::{Self, Table};
    use moveos_std::cbor;

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

    fun is_bitseed(inscription: &Inscription) : bool {
        let metaprotocol = ord::metaprotocol(inscription);
        option::is_some<String>(&metaprotocol) && option::borrow(&metaprotocol) == &string::utf8(b"bitseed")
    }

    fun get_SFT_op(metadata: &SimpleMap<String,vector<u8>>) : Option<std::string::String> {
        let op_key = string::utf8(b"op");

        if (simple_map::contains_key(metadata, &op_key)) {
            let op_bytes = simple_map::borrow(metadata, &op_key);
            return cbor::from_cbor_option<std::string::String>(*op_bytes)
        };

        return option::none()
    }

    fun get_SFT_tick(metadata: &SimpleMap<String,vector<u8>>) : Option<std::string::String> {
        let key = string::utf8(b"tick");

        if (simple_map::contains_key(metadata, &key)) {
            let bytes = simple_map::borrow(metadata, &key);
            return cbor::from_cbor_option<std::string::String>(*bytes)
        };

        return option::none()
    }

    fun get_SFT_amount(metadata: &SimpleMap<String,vector<u8>>) : Option<u64> {
        let key = string::utf8(b"amount");

        if (simple_map::contains_key(metadata, &key)) {
            let bytes = simple_map::borrow(metadata, &key);
            return cbor::from_cbor_option<u64>(*bytes)
        };

        return option::none()
    }

    fun get_SFT_attributes(metadata: &SimpleMap<String,vector<u8>>) : SimpleMap<String,vector<u8>> {
        let key = string::utf8(b"attributes");

        if (simple_map::contains_key(metadata, &key)) {
            let bytes = simple_map::borrow(metadata, &key);
            return cbor::to_map(*bytes)
        };

        return simple_map::new()
    }

    fun get_SFT_string_attribute(attributes: &SimpleMap<String,vector<u8>>, name: vector<u8>) : Option<std::string::String> {
        let key = string::utf8(name);

        if (simple_map::contains_key(attributes, &key)) {
            let bytes = simple_map::borrow(attributes, &key);
            return cbor::from_cbor_option<std::string::String>(*bytes)
        };

        return option::none()
    }

    fun get_SFT_bytes_attribute(attributes: &SimpleMap<String,vector<u8>>, name: vector<u8>) : Option<vector<u8>> {
        let key = string::utf8(name);

        if (simple_map::contains_key(attributes, &key)) {
            let bytes = simple_map::borrow(attributes, &key);
            return option::some(*bytes)
        };

        return option::none()
    }

    fun is_valid_bitseed(metadata: &SimpleMap<String,vector<u8>>) : (bool, Option<String>) {
        let tick = get_SFT_tick(metadata);
        if (option::is_none(&tick)) {
            return (false, option::some(std::string::utf8(b"metadata.tick is required")))
        };

        let tick_len = std::string::length(option::borrow(&tick));
        if (tick_len < 4 || tick_len > 32) {
            return (false, option::some(std::string::utf8(b"metadata.tick must be 4-32 characters")))
        };

        let amount = get_SFT_amount(metadata);
        if (option::is_none(&amount)) {
            return (false, option::some(std::string::utf8(b"metadata.amount is required")))
        };

        (true, option::none<String>())
    }

    fun is_valid_bitseed_deploy(metadata: &SimpleMap<String,vector<u8>>) : (bool, Option<String>) {
        let (is_valid, reason) = is_valid_bitseed(metadata);
        if (!is_valid) {
            return (false, reason)
        };

        let attributes = get_SFT_attributes(metadata);

        let generator = get_SFT_string_attribute(&attributes, b"generator");
        if (option::is_none(&generator)) {
            simple_map::drop(attributes);
            return (false, option::some(std::string::utf8(b"metadata.attributes.generator is required")))
        };

        let (is_valid, reason) = is_valid_generator_uri(option::borrow(&generator));
        if (!is_valid) {
            simple_map::drop(attributes);
            return (false, reason)
        };

        simple_map::drop(attributes);
        (true, option::none<String>())
    }

    fun is_valid_generator_uri(generator_uri: &String) : (bool, Option<String>) {
        let index = string::index_of(generator_uri, &std::string::utf8(b"/inscription/"));
        if (index != 0) {
            return (false, option::some(std::string::utf8(b"metadata.attributes.generator not start with /inscription/")))
        };

        let inscription_id_str = string::sub_string(generator_uri, vector::length(&b"/inscription/"), string::length(generator_uri));
        let inscription_id = ord::parse_inscription_id(inscription_id_str);
        if (!ord::exists_inscription(&inscription_id)) {
            return (false, option::some(std::string::utf8(b"metadata.attributes.generator inscription not exists")))
        };

        (true, option::none<String>())
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
            let metadata_bytes = ord::metadata(inscription);
            let metadata = cbor::to_map(metadata_bytes);

            let op = get_SFT_op(&metadata);
            if (option::is_some(&op)) {
                if (option::borrow(&op) == &string::utf8(b"deploy")) {
                    let (is_valid, reason) = is_valid_bitseed_deploy(&metadata);
                    ord::seal_metaprotocol_validity<Bitseed>(inscription_id, is_valid, reason);
                } else if (option::borrow(&op) == &string::utf8(b"mint")) {
                    ord::seal_metaprotocol_validity<Bitseed>(inscription_id, true, option::none());
                } else if (option::borrow(&op) == &string::utf8(b"split")) {
                    ord::seal_metaprotocol_validity<Bitseed>(inscription_id, true, option::none());
                } else if (option::borrow(&op) == &string::utf8(b"merge")) {
                    ord::seal_metaprotocol_validity<Bitseed>(inscription_id, true, option::none());
                } else {
                    ord::seal_metaprotocol_validity<Bitseed>(inscription_id, false, option::some(string::utf8(b"invalid op")));
                }
            } else {
                ord::seal_metaprotocol_validity<Bitseed>(inscription_id, false, option::some(string::utf8(b"op not found")));
            };

            simple_map::drop(metadata)
        }
    }

    #[test]
    fun test_is_valid_bitseed_deploy_ok(){
        let metadata_bytes = x"a4626f70666465706c6f79647469636b646d6f766566616d6f756e741903e86a61747472696275746573a366726570656174016967656e657261746f72784f2f696e736372697074696f6e2f3666353534373563653635303534616138333731643631386432313764613863396137363463656364616634646562636263653864363331326665366234643869306b6465706c6f795f617267738178377b22686569676874223a7b2274797065223a2272616e6765222c2264617461223a7b226d696e223a312c226d6178223a313030307d7d7d";
        let metadata = cbor::to_map(metadata_bytes);
        let (is_valid, reason) = is_valid_bitseed_deploy(&metadata);
        simple_map::drop(metadata);

        assert!(is_valid, 1);
        assert!(option::is_none(&reason), 1);
    }

    #[test]
    fun test_is_valid_bitseed_deploy_fail_with_tick_not_found(){
        let metadata_bytes = x"a4626f70666465706c6f79647469636bf766616d6f756e74016a61747472696275746573a16967656e657261746f72784f2f696e736372697074696f6e2f653839633162343830356538626235303236323038373632326263656662383533343232356364376138633264343832366433366630633161653333303831316931";
        let metadata = cbor::to_map(metadata_bytes);
        let (is_valid, reason) = is_valid_bitseed_deploy(&metadata);
        simple_map::drop(metadata);

        assert!(!is_valid, 1);
        assert!(option::borrow(&reason) == &std::string::utf8(b"metadata.tick is required"), 1);
    }

    #[test]
    fun test_is_valid_bitseed_deploy_fail_with_tick_too_short(){
        let metadata_bytes = x"a4626f70666465706c6f79647469636b6378787866616d6f756e74016a61747472696275746573a16967656e657261746f72784f2f696e736372697074696f6e2f653839633162343830356538626235303236323038373632326263656662383533343232356364376138633264343832366433366630633161653333303831316931";
        let metadata = cbor::to_map(metadata_bytes);
        let (is_valid, reason) = is_valid_bitseed_deploy(&metadata);
        simple_map::drop(metadata);

        assert!(!is_valid, 1);
        assert!(option::borrow(&reason) == &std::string::utf8(b"metadata.tick must be 4-32 characters"), 1);
    }

    #[test]
    fun test_is_valid_bitseed_deploy_fail_with_tick_too_long(){
        let metadata_bytes = x"a4626f70666465706c6f79647469636b78227878787878787878787878787878787878787878787878787878787878787878787866616d6f756e74016a61747472696275746573a16967656e657261746f72784f2f696e736372697074696f6e2f653839633162343830356538626235303236323038373632326263656662383533343232356364376138633264343832366433366630633161653333303831316931";
        let metadata = cbor::to_map(metadata_bytes);
        let (is_valid, reason) = is_valid_bitseed_deploy(&metadata);
        simple_map::drop(metadata);

        assert!(!is_valid, 1);
        assert!(option::borrow(&reason) == &std::string::utf8(b"metadata.tick must be 4-32 characters"), 1);
    }

    #[test]
    fun test_is_valid_bitseed_deploy_fail_with_amount_not_found(){
        let metadata_bytes = x"a4626f70666465706c6f79647469636b68746573745469636b66616d6f756e74f76a61747472696275746573a16967656e657261746f72784f2f696e736372697074696f6e2f653839633162343830356538626235303236323038373632326263656662383533343232356364376138633264343832366433366630633161653333303831316931";
        let metadata = cbor::to_map(metadata_bytes);
        let (is_valid, reason) = is_valid_bitseed_deploy(&metadata);
        simple_map::drop(metadata);

        assert!(!is_valid, 1);
        assert!(option::borrow(&reason) == &std::string::utf8(b"metadata.amount is required"), 1);
    }


    #[test]
    fun test_is_valid_bitseed_deploy_fail_with_generator_not_found(){
        let metadata_bytes = x"a4626f70666465706c6f79647469636b68746573745469636b66616d6f756e74016a61747472696275746573a0";
        let metadata = cbor::to_map(metadata_bytes);
        let (is_valid, reason) = is_valid_bitseed_deploy(&metadata);
        simple_map::drop(metadata);

        assert!(!is_valid, 1);
        assert!(option::borrow(&reason) == &std::string::utf8(b"metadata.attributes.generator is required"), 1);
    }

    #[test]
    fun test_is_valid_bitseed_deploy_fail_with_generator_uri_not_start_with_generator(){
        let metadata_bytes = x"a4626f70666465706c6f79647469636b646d6f766566616d6f756e74016a61747472696275746573a16967656e657261746f7278472f7878782f653839633162343830356538626235303236323038373632326263656662383533343232356364376138633264343832366433366630633161653333303831316931";
        let metadata = cbor::to_map(metadata_bytes);
        let (is_valid, reason) = is_valid_bitseed_deploy(&metadata);
        simple_map::drop(metadata);

        assert!(!is_valid, 1);
        assert!(option::borrow(&reason) == &std::string::utf8(b"metadata.attributes.generator not start with /inscription/"), 1);
    }
}
