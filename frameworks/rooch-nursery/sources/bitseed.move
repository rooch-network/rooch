// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

module rooch_nursery::bitseed {
    use std::vector;
    use std::option::{Self, Option};
    use std::string::{Self, String};
    use std::bcs;
    
    use moveos_std::address;
    use moveos_std::hash;
    use moveos_std::hex;
    use moveos_std::object::{Self, Object};
    use moveos_std::string_utils;
    use moveos_std::simple_map::{Self, SimpleMap};
    use moveos_std::wasm;
    use moveos_std::table::{Self, Table};
    use moveos_std::cbor;

    use bitcoin_move::types;
    use bitcoin_move::ord::{Self, Inscription, InscriptionID, MetaprotocolValidity};
    use bitcoin_move::bitcoin;

    const BIT_SEED_DEPLOY: vector<u8> = b"bitseed_deploy";
    const BIT_SEED_MINT: vector<u8> = b"bitseed_mint";
    const BIT_SEED_GENERATOR_TICK: vector<u8> = b"generator";

    friend rooch_nursery::genesis;

    struct Bitseed has key {}

    struct BitseedCoinInfo has store, copy, drop {
        tick: String,
        generator: Option<InscriptionID>,
        max: u64,
        repeat: u64,
        has_user_input: bool,
        deploy_args: Option<vector<u8>>,
        supply: u64,
    }

    struct BitseedStore has key{
        coins: Table<String, BitseedCoinInfo>,
    }

    public(friend) fun genesis_init(_genesis_account: &signer){
        let bitseed_store = BitseedStore{
            coins: table::new(),
        };

        // init built-in generator tick
        let tick = string::utf8(BIT_SEED_GENERATOR_TICK);
        let coin_info = BitseedCoinInfo{ 
            tick: tick, 
            generator: option::none(),
            max: 1000000u64,
            repeat: 0,
            has_user_input: false,
            deploy_args: option::none(),
            supply: 0,
        };
        table::add(&mut bitseed_store.coins, tick, coin_info);

        let obj = object::new_named_object(bitseed_store);
        object::to_shared(obj);
    }

    fun borrow_store() : &mut BitseedStore {
        let bitseed_store_object_id = object::named_object_id<BitseedStore>();
        let bitseed_store_obj = object::borrow_mut_object_shared<BitseedStore>(bitseed_store_object_id);
        object::borrow_mut(bitseed_store_obj)
    }

    #[test_only]
    fun init_bitseed_store_for_test(_genesis_account: &signer) {
        genesis_init(_genesis_account)
    }

    public fun bitseed_deploy_key(): vector<u8> {
        BIT_SEED_DEPLOY
    }

    public fun bitseed_mint_key(): vector<u8> {
        BIT_SEED_MINT
    }

    public fun get_coin_info(bitseed_store_obj:&Object<BitseedStore>, tick: &String) : Option<BitseedCoinInfo> {
        let tick = string_utils::to_lower_case(tick);
        let bitseed_store = object::borrow(bitseed_store_obj);

        if (table::contains(&bitseed_store.coins, tick)) {
            option::some(*table::borrow(&bitseed_store.coins, tick))
        } else {
            option::none()
        }
    }

    public fun coin_info_tick(self: &BitseedCoinInfo): String {
        self.tick
    }

    public fun coin_info_generator(self: &BitseedCoinInfo): Option<InscriptionID> {
        self.generator
    }

    public fun coin_info_max(self: &BitseedCoinInfo): u64 {
        self.max
    }

    public fun coin_info_repeat(self: &BitseedCoinInfo): u64 {
        self.repeat
    }

    public fun coin_info_has_user_input(self: &BitseedCoinInfo): bool {
        self.has_user_input
    }

    public fun coin_info_deploy_args_option(self: &BitseedCoinInfo): Option<vector<u8>> {
        self.deploy_args
    }

    public fun coin_info_deploy_args(self: &BitseedCoinInfo): vector<u8> {
        if (option::is_some(&self.deploy_args)) {
            *option::borrow(&self.deploy_args)
        } else {
            vector::empty()
        }
    }

    public fun coin_info_supply(self: &BitseedCoinInfo): u64 {
        self.supply
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

    fun get_SFT_tick_option(metadata: &SimpleMap<String,vector<u8>>) : Option<std::string::String> {
        let key = string::utf8(b"tick");

        if (simple_map::contains_key(metadata, &key)) {
            let bytes = simple_map::borrow(metadata, &key);
            return cbor::from_cbor_option<std::string::String>(*bytes)
        };

        return option::none()
    }

    fun get_SFT_tick(metadata: &SimpleMap<String,vector<u8>>) : std::string::String {
        let tick_option = get_SFT_tick_option(metadata);
        option::destroy_some(tick_option)
    }

    fun get_SFT_amount_option(metadata: &SimpleMap<String,vector<u8>>) : Option<u64> {
        let key = string::utf8(b"amount");

        if (simple_map::contains_key(metadata, &key)) {
            let bytes = simple_map::borrow(metadata, &key);
            return cbor::from_cbor_option<u64>(*bytes)
        };

        return option::none()
    }

    fun get_SFT_amount(metadata: &SimpleMap<String,vector<u8>>) : u64 {
        let amount_option = get_SFT_amount_option(metadata);
        option::destroy_some(amount_option)
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

    fun get_SFT_u64_attribute(attributes: &SimpleMap<String,vector<u8>>, name: vector<u8>) : Option<u64> {
        let key = string::utf8(name);

        if (simple_map::contains_key(attributes, &key)) {
            let bytes = simple_map::borrow(attributes, &key);
            return cbor::from_cbor_option<u64>(*bytes)
        };

        return option::none()
    }

    fun get_SFT_bool_attribute(attributes: &SimpleMap<String,vector<u8>>, name: vector<u8>) : Option<bool> {
        let key = string::utf8(name);

        if (simple_map::contains_key(attributes, &key)) {
            let bytes = simple_map::borrow(attributes, &key);
            return cbor::from_cbor_option<bool>(*bytes)
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
        let tick = get_SFT_tick_option(metadata);
        if (option::is_none(&tick)) {
            return (false, option::some(std::string::utf8(b"metadata.tick is required")))
        };

        let tick_len = std::string::length(option::borrow(&tick));
        if (tick_len < 4 || tick_len > 32) {
            return (false, option::some(std::string::utf8(b"metadata.tick must be 4-32 characters")))
        };

        let amount = get_SFT_amount_option(metadata);
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
        let inscription_id_option = ord::parse_inscription_id(&inscription_id_str);
        if (option::is_none(&inscription_id_option)) {
            return (false, option::some(std::string::utf8(b"metadata.attributes.generator inscription_id parse fail")))
        };

        let inscription_id = option::extract(&mut inscription_id_option);
        if (!ord::exists_inscription(inscription_id)) {
            return (false, option::some(std::string::utf8(b"metadata.attributes.generator inscription not exists")))
        };

        if (!ord::exists_metaprotocol_validity(inscription_id)) {
            return (false, option::some(std::string::utf8(b"metadata.attributes.generator inscription metaprotocol validity not exists")))
        };

        let metaprotocol_validity = ord::borrow_metaprotocol_validity(inscription_id);

        let is_match = ord::metaprotocol_validity_protocol_match<Bitseed>(metaprotocol_validity);
        if (!is_match) {
            return (false, option::some(std::string::utf8(b"metadata.attributes.generator inscription metaprotocol validity protocol not match")))
        };

        let is_valid = ord::metaprotocol_validity_is_valid(metaprotocol_validity);
        if (!is_valid) {
            return (false, option::some(std::string::utf8(b"metadata.attributes.generator inscription metaprotocol validity not valid")))
        };

        (true, option::none<String>())
    }

    fun deploy_tick(metadata: &SimpleMap<String,vector<u8>>): (bool, Option<String>){
        let bitseed_store = borrow_store();

        let tick = get_SFT_tick(metadata);
        let tick = string_utils::to_lower_case(&tick);

        if(table::contains(&bitseed_store.coins, tick)){
            return (false, option::some(string::utf8(b"bitseed tick already exists")))
        };
        
        let max = get_SFT_amount(metadata);
        let attributes = get_SFT_attributes(metadata);

        let repeat = 0u64;
        let repeat_option = get_SFT_u64_attribute(&attributes, b"repeat");
        if (option::is_some(&repeat_option)) {
            repeat = option::destroy_some(repeat_option);
        };

        let generator_uri = option::destroy_some(get_SFT_string_attribute(&attributes, b"generator"));
        let inscription_id_str = string::sub_string(&generator_uri, vector::length(&b"/inscription/"), string::length(&generator_uri));
        let inscription_id_option = ord::parse_inscription_id(&inscription_id_str);
        let inscription_id = option::destroy_some(inscription_id_option);

        let has_user_input = false;
        let has_user_input_option = get_SFT_bool_attribute(&attributes, b"has_user_input");
        if (option::is_some(&has_user_input_option)) {
            has_user_input = option::destroy_some(has_user_input_option);
        };

        let deploy_args = get_SFT_bytes_attribute(&attributes, b"deploy_args");

        let coin_info = BitseedCoinInfo{ 
            tick, 
            generator: option::some(inscription_id),
            max,
            repeat,
            has_user_input,
            deploy_args,
            supply: 0,
        };

        table::add(&mut bitseed_store.coins, tick, coin_info);

        simple_map::drop(attributes);
        (true, option::none<String>())
    }
    
    fun is_valid_bitseed_mint(metadata: &SimpleMap<String,vector<u8>>, seed: vector<u8>) : (bool, Option<String>) {
        let (is_valid, reason) = is_valid_bitseed(metadata);
        if (!is_valid) {
            return (false, reason)
        };

        let tick = get_SFT_tick(metadata);
        let attributes = get_SFT_attributes(metadata);
        let amount = get_SFT_amount(metadata);

        let bitseed_store_object_id = object::named_object_id<BitseedStore>();
        let brc20_store_obj = object::borrow_mut_object_shared<BitseedStore>(bitseed_store_object_id);
        let coin_info_option = get_coin_info(brc20_store_obj, &tick);
        if (option::is_none(&coin_info_option)) {
            simple_map::drop(attributes);
            return (false, option::some(std::string::utf8(b"tick not deploy")))
        };

        let coin_info = option::destroy_some(coin_info_option);

        let max = coin_info_max(&coin_info);
        let supply = coin_info_supply(&coin_info);
        let has_user_input = coin_info_has_user_input(&coin_info);
        let deploy_args = coin_info_deploy_args(&coin_info);

        if (supply + amount > max) {
            simple_map::drop(attributes);
            return (false, option::some(std::string::utf8(b"maximum supply exceeded")))
        };

        let user_input = vector::empty();
        if (has_user_input) {
            let user_input_option = get_SFT_bytes_attribute(&attributes, b"user_input");
            if (option::is_none(&user_input_option)) {
                simple_map::drop(attributes);
                return (false, option::some(std::string::utf8(b"metadata.attributes.user_input is required")))
            };

            user_input = *option::borrow(&user_input_option);
        };

        let generator_inscription_id_option = coin_info_generator(&coin_info);
        if (option::is_none(&generator_inscription_id_option)) {
            simple_map::drop(attributes);
            return (true, option::none<String>())
        };

        let generator_inscription_id = option::destroy_some(generator_inscription_id_option);
        if (!ord::exists_metaprotocol_validity(generator_inscription_id)) {
            simple_map::drop(attributes);
            return (false, option::some(std::string::utf8(b"generator_inscription_id is not validity bitseed")))
        };

        let generator_txid = ord::inscription_id_txid(&generator_inscription_id);
        let generator_index = ord::inscription_id_index(&generator_inscription_id);
        let inscription_obj = ord::borrow_inscription(generator_txid, generator_index);

        let inscrption = object::borrow(inscription_obj);
        let wasm_bytes = ord::body(inscrption);

        let attributes_bytes = simple_map::borrow(metadata, &string::utf8(b"attributes"));

        let (is_valid, reason) = inscribe_verify(wasm_bytes, deploy_args, seed, user_input, *attributes_bytes);
        if (!is_valid) {
            simple_map::drop(attributes);
            return (false, reason)
        };

        simple_map::drop(attributes);
        (true, option::none<String>())
    }

    public fun inscribe_verify(wasm_bytes: vector<u8>, deploy_args: vector<u8>,
                               seed: vector<u8>, user_input: vector<u8>, attributes_output: vector<u8>): (bool, Option<String>) {
        let wasm_instance_option = wasm::create_wasm_instance_option(wasm_bytes);
        if (option::is_none(&wasm_instance_option)) {
            option::destroy_none(wasm_instance_option);
            return (false, option::some(std::string::utf8(b"create wasm instance fail")))
        };

        let wasm_instance = option::destroy_some(wasm_instance_option);
        let function_name = b"inscribe_verify";

        let buffer = pack_inscribe_generate_args(deploy_args, seed, user_input);
        let arg_with_length = wasm::add_length_with_data(buffer);

        let arg_list = vector::empty<vector<u8>>();
        vector::push_back(&mut arg_list, arg_with_length);
        vector::push_back(&mut arg_list, attributes_output);
        let memory_args_list = wasm::create_memory_wasm_args(&mut wasm_instance, function_name, arg_list);

        let ret_val_option = wasm::execute_wasm_function_option(&mut wasm_instance, function_name, memory_args_list);

        wasm::release_wasm_instance(wasm_instance);

        if (option::is_none(&ret_val_option)) {
            option::destroy_none(ret_val_option);
            return (false, option::some(std::string::utf8(b"inscribe verify execute_wasm_function fail")))
        };

        let ret_val = option::destroy_some(ret_val_option);
        if (ret_val != 1 ) {
            return (false, option::some(std::string::utf8(b"inscribe verify fail")))
        };

        (true, option::none<String>())
    }

    #[data_struct]
    struct InscribeGenerateArgs has copy, drop, store {
        attrs: vector<u8>,
        seed: std::string::String,
        user_input: std::string::String,
    }

    fun pack_inscribe_generate_args(deploy_args: vector<u8>, seed: vector<u8>, user_input: vector<u8>): vector<u8>{
        let args = InscribeGenerateArgs{
            attrs: deploy_args,
            seed: string::utf8(seed),
            user_input: string::utf8(user_input)
        };

        cbor::to_cbor(&args)
    }

    fun generate_seed_from_inscription(inscription: &Inscription): vector<u8> {
        let inscription_txid = ord::txid(inscription);
        let tx_option = bitcoin::get_tx(inscription_txid);
        if (option::is_none(&tx_option)) {
            return vector::empty()
        };

        let tx = option::destroy_some(tx_option);
        let input = types::tx_input(&tx);
        let index = ord::index(inscription);
        let txin = vector::borrow(input, (index as u64));
        let outpoint = types::txin_previous_output(txin);

        let txid = types::outpoint_txid(outpoint);
        let vout = types::outpoint_vout(outpoint);

        let seed_tx_option = bitcoin::get_tx(txid);
        if (option::is_none(&seed_tx_option)) {
            return vector::empty()
        };

        let seed_height_option = bitcoin::get_tx_height(txid);
        if (option::is_none(&seed_height_option)) {
            return vector::empty()
        };

        let seed_height = *option::borrow(&seed_height_option);

        let block_header_option = bitcoin::get_block_by_height(seed_height);
        if (option::is_none(&block_header_option)) {
            return vector::empty()
        };

        let block_header = option::borrow(&block_header_option);
        let block_hash = types::merkle_root(block_header);

        let buf = vector::empty();
        vector::append(&mut buf, address::to_bytes(&block_hash));
        vector::append(&mut buf, address::to_bytes(&txid));
        vector::append(&mut buf, bcs::to_bytes(&vout)); //TODO vout to le_bytes
        hex::encode(hash::sha3_256(buf))
    }

    // ==== Process Bitseed Entry ==== //
    public fun process_inscription(inscription: &Inscription) {
        let txid = ord::txid(inscription);
        let index = ord::index(inscription);
        let inscription_id = ord::new_inscription_id(txid, index);

        if (is_bitseed(inscription)) {
            let metadata_bytes = ord::metadata(inscription);
            let metadata = cbor::to_map(metadata_bytes);

            let op = get_SFT_op(&metadata);
            if (option::is_some(&op)) {
                if (option::borrow(&op) == &string::utf8(b"deploy")) {
                    let (is_valid, reason) = is_valid_bitseed_deploy(&metadata);
                    if (!is_valid) {
                        ord::seal_metaprotocol_validity<Bitseed>(inscription_id, is_valid, reason);

                        simple_map::drop(metadata);
                        return ()
                    };

                    let (ok, reason) = deploy_tick(&metadata);
                    if (!ok) {
                        ord::seal_metaprotocol_validity<Bitseed>(inscription_id, false, reason);

                        simple_map::drop(metadata);
                        return ()
                    };

                    ord::seal_metaprotocol_validity<Bitseed>(inscription_id, true, option::none());
                } else if (option::borrow(&op) == &string::utf8(b"mint")) {
                    let seed = generate_seed_from_inscription(inscription);
                    let (is_valid, reason) = is_valid_bitseed_mint(&metadata, seed);
                    ord::seal_metaprotocol_validity<Bitseed>(inscription_id, is_valid, reason);
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

    public fun view_validity(inscription_id_str: String) : Option<MetaprotocolValidity> {
        let inscription_id_option = ord::parse_inscription_id(&inscription_id_str);
        if (option::is_none(&inscription_id_option)) {
            return option::none()
        };

        let inscription_id = option::destroy_some(inscription_id_option);
        if (!ord::exists_metaprotocol_validity(inscription_id)) {
            return option::none()
        };

        let validity = ord::borrow_metaprotocol_validity(inscription_id);

        option::some(*validity)
    }

    #[test_only]
    struct TestProtocol has key {}

    #[test(genesis_account=@0x4)]
    fun test_is_valid_bitseed_deploy_ok(genesis_account: &signer){
        let (_test_address, test_inscription_id) = ord::setup_inscription_for_test(genesis_account);
        ord::seal_metaprotocol_validity<Bitseed>(test_inscription_id, true, option::none());

        let metadata_bytes = x"a4626f70666465706c6f79647469636b646d6f766566616d6f756e74016a61747472696275746573a16967656e657261746f72784f2f696e736372697074696f6e2f373764666332666535393834313962303036343163323936313831613936636631363934333639376635373334383062303233623737636365383261646132316930";
        let metadata = cbor::to_map(metadata_bytes);
        let (is_valid, reason) = is_valid_bitseed_deploy(&metadata);
        simple_map::drop(metadata);

        assert!(is_valid, 1);
        assert!(option::is_none(&reason), 1);
    }

    #[test]
    fun test_is_valid_bitseed_deploy_fail_for_tick_not_found(){
        let metadata_bytes = x"a4626f70666465706c6f79647469636bf766616d6f756e74016a61747472696275746573a16967656e657261746f72784f2f696e736372697074696f6e2f653839633162343830356538626235303236323038373632326263656662383533343232356364376138633264343832366433366630633161653333303831316931";
        let metadata = cbor::to_map(metadata_bytes);
        let (is_valid, reason) = is_valid_bitseed_deploy(&metadata);
        simple_map::drop(metadata);

        assert!(!is_valid, 1);
        assert!(option::borrow(&reason) == &std::string::utf8(b"metadata.tick is required"), 1);
    }

    #[test]
    fun test_is_valid_bitseed_deploy_fail_for_tick_too_short(){
        let metadata_bytes = x"a4626f70666465706c6f79647469636b6378787866616d6f756e74016a61747472696275746573a16967656e657261746f72784f2f696e736372697074696f6e2f653839633162343830356538626235303236323038373632326263656662383533343232356364376138633264343832366433366630633161653333303831316931";
        let metadata = cbor::to_map(metadata_bytes);
        let (is_valid, reason) = is_valid_bitseed_deploy(&metadata);
        simple_map::drop(metadata);

        assert!(!is_valid, 1);
        assert!(option::borrow(&reason) == &std::string::utf8(b"metadata.tick must be 4-32 characters"), 1);
    }

    #[test]
    fun test_is_valid_bitseed_deploy_fail_for_tick_too_long(){
        let metadata_bytes = x"a4626f70666465706c6f79647469636b78227878787878787878787878787878787878787878787878787878787878787878787866616d6f756e74016a61747472696275746573a16967656e657261746f72784f2f696e736372697074696f6e2f653839633162343830356538626235303236323038373632326263656662383533343232356364376138633264343832366433366630633161653333303831316931";
        let metadata = cbor::to_map(metadata_bytes);
        let (is_valid, reason) = is_valid_bitseed_deploy(&metadata);
        simple_map::drop(metadata);

        assert!(!is_valid, 1);
        assert!(option::borrow(&reason) == &std::string::utf8(b"metadata.tick must be 4-32 characters"), 1);
    }

    #[test]
    fun test_is_valid_bitseed_deploy_fail_for_amount_not_found(){
        let metadata_bytes = x"a4626f70666465706c6f79647469636b68746573745469636b66616d6f756e74f76a61747472696275746573a16967656e657261746f72784f2f696e736372697074696f6e2f653839633162343830356538626235303236323038373632326263656662383533343232356364376138633264343832366433366630633161653333303831316931";
        let metadata = cbor::to_map(metadata_bytes);
        let (is_valid, reason) = is_valid_bitseed_deploy(&metadata);
        simple_map::drop(metadata);

        assert!(!is_valid, 1);
        assert!(option::borrow(&reason) == &std::string::utf8(b"metadata.amount is required"), 1);
    }


    #[test]
    fun test_is_valid_bitseed_deploy_fail_for_generator_not_found(){
        let metadata_bytes = x"a4626f70666465706c6f79647469636b68746573745469636b66616d6f756e74016a61747472696275746573a0";
        let metadata = cbor::to_map(metadata_bytes);
        let (is_valid, reason) = is_valid_bitseed_deploy(&metadata);
        simple_map::drop(metadata);

        assert!(!is_valid, 1);
        assert!(option::borrow(&reason) == &std::string::utf8(b"metadata.attributes.generator is required"), 1);
    }

    #[test]
    fun test_is_valid_bitseed_deploy_fail_for_generator_uri_not_start_with_generator(){
        let metadata_bytes = x"a4626f70666465706c6f79647469636b646d6f766566616d6f756e74016a61747472696275746573a16967656e657261746f7278472f7878782f653839633162343830356538626235303236323038373632326263656662383533343232356364376138633264343832366433366630633161653333303831316931";
        let metadata = cbor::to_map(metadata_bytes);
        let (is_valid, reason) = is_valid_bitseed_deploy(&metadata);
        simple_map::drop(metadata);

        assert!(!is_valid, 1);
        assert!(option::borrow(&reason) == &std::string::utf8(b"metadata.attributes.generator not start with /inscription/"), 1);
    }

    #[test]
    fun test_is_valid_bitseed_deploy_fail_for_parse_inscription_id_fail(){
        let metadata_bytes = x"a4626f70666465706c6f79647469636b646d6f766566616d6f756e74016a61747472696275746573a16967656e657261746f72784f2f696e736372697074696f6e2f377864666332666535393834313962303036343163323936313831613936636631363934333639376635373334383062303233623737636365383261646132316930";
        let metadata = cbor::to_map(metadata_bytes);
        let (is_valid, reason) = is_valid_bitseed_deploy(&metadata);
        simple_map::drop(metadata);

        assert!(!is_valid, 1);
        assert!(option::borrow(&reason) == &std::string::utf8(b"metadata.attributes.generator inscription_id parse fail"), 1);
    }

    #[test(genesis_account=@0x4)]
    fun test_is_valid_bitseed_deploy_fail_for_inscription_no_exists(genesis_account: &signer){
        ord::init_for_test(genesis_account);

        let metadata_bytes = x"a4626f70666465706c6f79647469636b646d6f766566616d6f756e74016a61747472696275746573a16967656e657261746f72784f2f696e736372697074696f6e2f373764666332666535393834313962303036343163323936313831613936636631363934333639376635373334383062303233623737636365383261646132316930";
        let metadata = cbor::to_map(metadata_bytes);
        let (is_valid, reason) = is_valid_bitseed_deploy(&metadata);
        simple_map::drop(metadata);

        assert!(!is_valid, 1);
        assert!(option::borrow(&reason) == &std::string::utf8(b"metadata.attributes.generator inscription not exists"), 1);
    }

    #[test(genesis_account=@0x4)]
    fun test_is_valid_bitseed_deploy_fail_for_inscription_metaprotocol_validity_not_exists(genesis_account: &signer){
        let (_test_address, _test_inscription_id) = ord::setup_inscription_for_test(genesis_account);

        let metadata_bytes = x"a4626f70666465706c6f79647469636b646d6f766566616d6f756e74016a61747472696275746573a16967656e657261746f72784f2f696e736372697074696f6e2f373764666332666535393834313962303036343163323936313831613936636631363934333639376635373334383062303233623737636365383261646132316930";
        let metadata = cbor::to_map(metadata_bytes);
        let (is_valid, reason) = is_valid_bitseed_deploy(&metadata);
        simple_map::drop(metadata);

        assert!(!is_valid, 1);
        assert!(option::borrow(&reason) == &std::string::utf8(b"metadata.attributes.generator inscription metaprotocol validity not exists"), 1);
    }

    #[test(genesis_account=@0x4)]
    fun test_is_valid_bitseed_deploy_fail_for_inscription_metaprotocol_validity_protocol_not_match(genesis_account: &signer){
        let (_test_address, test_inscription_id) = ord::setup_inscription_for_test(genesis_account);
        ord::seal_metaprotocol_validity<TestProtocol>(test_inscription_id, true, option::none());

        let metadata_bytes = x"a4626f70666465706c6f79647469636b646d6f766566616d6f756e74016a61747472696275746573a16967656e657261746f72784f2f696e736372697074696f6e2f373764666332666535393834313962303036343163323936313831613936636631363934333639376635373334383062303233623737636365383261646132316930";
        let metadata = cbor::to_map(metadata_bytes);
        let (is_valid, reason) = is_valid_bitseed_deploy(&metadata);
        simple_map::drop(metadata);

        assert!(!is_valid, 1);
        assert!(option::borrow(&reason) == &std::string::utf8(b"metadata.attributes.generator inscription metaprotocol validity protocol not match"), 1);
    }

    #[test(genesis_account=@0x4)]
    fun test_is_valid_bitseed_deploy_fail_for_inscription_metaprotocol_validity_not_valid(genesis_account: &signer){
        let (_test_address, test_inscription_id) = ord::setup_inscription_for_test(genesis_account);
        ord::seal_metaprotocol_validity<Bitseed>(test_inscription_id, false, option::none());

        let metadata_bytes = x"a4626f70666465706c6f79647469636b646d6f766566616d6f756e74016a61747472696275746573a16967656e657261746f72784f2f696e736372697074696f6e2f373764666332666535393834313962303036343163323936313831613936636631363934333639376635373334383062303233623737636365383261646132316930";
        let metadata = cbor::to_map(metadata_bytes);
        let (is_valid, reason) = is_valid_bitseed_deploy(&metadata);
        simple_map::drop(metadata);

        assert!(!is_valid, 1);
        assert!(option::borrow(&reason) == &std::string::utf8(b"metadata.attributes.generator inscription metaprotocol validity not valid"), 1);
    }

    #[test(genesis_account=@0x4)]
    fun test_deploy_tick_ok(genesis_account: &signer){
        genesis_init(genesis_account);

        let (_test_address, test_inscription_id) = ord::setup_inscription_for_test(genesis_account);
        ord::seal_metaprotocol_validity<Bitseed>(test_inscription_id, true, option::none());

        let metadata_bytes = x"a4626f70666465706c6f79647469636b646d6f766566616d6f756e74016a61747472696275746573a16967656e657261746f72784f2f696e736372697074696f6e2f373764666332666535393834313962303036343163323936313831613936636631363934333639376635373334383062303233623737636365383261646132316930";
        let metadata = cbor::to_map(metadata_bytes);
        let (ok, reason) = deploy_tick(&metadata);
        simple_map::drop(metadata);

        assert!(ok, 1);
        assert!(option::is_none(&reason), 1);

        let bitseed_store_object_id = object::named_object_id<BitseedStore>();
        let brc20_store_obj = object::borrow_mut_object_shared<BitseedStore>(bitseed_store_object_id);
        let coin_info_option = get_coin_info(brc20_store_obj, &string::utf8(b"move"));
        assert!(option::is_some(&coin_info_option), 1);

        let coin_info = option::destroy_some(coin_info_option);

        // check tick
        let tick = coin_info_tick(&coin_info);
        assert!(tick == string::utf8(b"move"), 2);

        // check generator
        let generator_option = coin_info_generator(&coin_info);
        assert!(option::is_some(&generator_option), 3);

        let generator = option::destroy_some(generator_option);
        let test_txid = @0x21da2ae8cc773b020b4873f597369416cf961a1896c24106b0198459fec2df77;
        let test_index = 0;
        assert!(ord::inscription_id_txid(&generator) == test_txid, 4);
        assert!(ord::inscription_id_index(&generator) == test_index, 5);

        // check max
        let max = coin_info_max(&coin_info);
        assert!(max == 1u64, 6);

        // check repeat
        let repeat = coin_info_repeat(&coin_info);
        assert!(repeat == 0, 7);

        // check has_user_input
        let has_user_input = coin_info_has_user_input(&coin_info);
        assert!(!has_user_input, 8);

        // check deploy_args
        let deploy_args = coin_info_deploy_args_option(&coin_info);
        assert!(option::is_none(&deploy_args), 9)
    }

    #[test_only]
    use moveos_std::features;

    #[test(genesis_account=@0x4)]
    fun test_is_valid_bitseed_mint_fail_with_tick_not_deploy(genesis_account: &signer){
        genesis_init(genesis_account);

        let (_test_address, test_inscription_id) = ord::setup_inscription_for_test(genesis_account);
        ord::seal_metaprotocol_validity<Bitseed>(test_inscription_id, true, option::none());

        let metadata_bytes = x"a4626f70666465706c6f79647469636b646d6f766566616d6f756e74016a61747472696275746573a16967656e657261746f72784f2f696e736372697074696f6e2f373764666332666535393834313962303036343163323936313831613936636631363934333639376635373334383062303233623737636365383261646132316930";
        let metadata = cbor::to_map(metadata_bytes);
        let seed = vector::empty();
        let (is_valid, reason) = is_valid_bitseed_mint(&metadata, seed);
        simple_map::drop(metadata);

        assert!(!is_valid, 1);
        assert!(option::borrow(&reason) == &std::string::utf8(b"tick not deploy"), 1);
    }

    #[test(genesis_account=@0x4)]
    fun test_is_valid_bitseed_mint_fail_with_maximum_supply_exceeded(genesis_account: &signer){
        genesis_init(genesis_account);

        let (_test_address, test_inscription_id) = ord::setup_inscription_for_test(genesis_account);
        ord::seal_metaprotocol_validity<Bitseed>(test_inscription_id, true, option::none());

        let metadata_bytes = x"a4626f70666465706c6f79647469636b646d6f766566616d6f756e74016a61747472696275746573a16967656e657261746f72784f2f696e736372697074696f6e2f373764666332666535393834313962303036343163323936313831613936636631363934333639376635373334383062303233623737636365383261646132316930";
        let metadata = cbor::to_map(metadata_bytes);
        let (is_valid, reason) = is_valid_bitseed_deploy(&metadata);
        
        assert!(is_valid, 1);
        assert!(option::is_none(&reason), 1);

        let (ok, reason) = deploy_tick(&metadata);
        assert!(ok, 1);
        assert!(option::is_none(&reason), 1);

        simple_map::drop(metadata);

        let metadata_bytes = x"a4626f70646d696e74647469636b646d6f766566616d6f756e7418646a61747472696275746573a16967656e657261746f72784f2f696e736372697074696f6e2f377864666332666535393834313962303036343163323936313831613936636631363934333639376635373334383062303233623737636365383261646132316930";
        let metadata = cbor::to_map(metadata_bytes);
        let seed = vector::empty();
        let (is_valid, reason) = is_valid_bitseed_mint(&metadata, seed);
        simple_map::drop(metadata);

        assert!(!is_valid, 1);
        assert!(option::borrow(&reason) == &std::string::utf8(b"maximum supply exceeded"), 1);
    }

    #[test(genesis_account=@0x4)]
    fun test_is_valid_bitseed_mint_fail_with_user_input_is_required(genesis_account: &signer){
        genesis_init(genesis_account);

        let (_test_address, test_inscription_id) = ord::setup_inscription_for_test(genesis_account);
        ord::seal_metaprotocol_validity<Bitseed>(test_inscription_id, true, option::none());

        let metadata_bytes = x"a4626f70666465706c6f79647469636b646d6f766566616d6f756e741927106a61747472696275746573a466726570656174056967656e657261746f72784f2f696e736372697074696f6e2f3737646663326665353938343139623030363431633239363138316139366366313639343336393766353733343830623032336237376363653832616461323169306e6861735f757365725f696e707574f56b6465706c6f795f617267738178377b22686569676874223a7b2274797065223a2272616e6765222c2264617461223a7b226d696e223a312c226d6178223a313030307d7d7d";
        let metadata = cbor::to_map(metadata_bytes);
        let (is_valid, reason) = is_valid_bitseed_deploy(&metadata);

        assert!(is_valid, 1);
        assert!(option::is_none(&reason), 1);

        let (ok, reason) = deploy_tick(&metadata);
        assert!(ok, 1);
        assert!(option::is_none(&reason), 1);

        simple_map::drop(metadata);

        let metadata_bytes = x"a4626f70646d696e74647469636b646d6f766566616d6f756e7418646a61747472696275746573a16967656e657261746f72784f2f696e736372697074696f6e2f377864666332666535393834313962303036343163323936313831613936636631363934333639376635373334383062303233623737636365383261646132316930";
        let metadata = cbor::to_map(metadata_bytes);
        let seed = vector::empty();
        let (is_valid, reason) = is_valid_bitseed_mint(&metadata, seed);
        simple_map::drop(metadata);

        assert!(!is_valid, 1);
        assert!(option::borrow(&reason) == &std::string::utf8(b"metadata.attributes.user_input is required"), 1);
    }

    #[test(genesis_account=@0x4)]
    fun test_is_valid_bitseed_mint_fail_with_wasm_verify_fail(genesis_account: &signer){
        features::init_and_enable_all_features_for_test();
        
        let (_test_address, test_inscription_id) = ord::setup_inscription_for_test(genesis_account);
        ord::seal_metaprotocol_validity<Bitseed>(test_inscription_id, true, option::none());

        init_bitseed_store_for_test(genesis_account);

        let metadata_bytes = x"a4626f70666465706c6f79647469636b646d6f766566616d6f756e741927106a61747472696275746573a466726570656174056967656e657261746f72784f2f696e736372697074696f6e2f3737646663326665353938343139623030363431633239363138316139366366313639343336393766353733343830623032336237376363653832616461323169306e6861735f757365725f696e707574f56b6465706c6f795f617267738178377b22686569676874223a7b2274797065223a2272616e6765222c2264617461223a7b226d696e223a312c226d6178223a313030307d7d7d";
        let metadata = cbor::to_map(metadata_bytes);
        let (is_valid, reason) = is_valid_bitseed_deploy(&metadata);

        assert!(is_valid, 1);
        assert!(option::is_none(&reason), 1);

        let (ok, reason) = deploy_tick(&metadata);
        assert!(ok, 1);
        assert!(option::is_none(&reason), 1);

        simple_map::drop(metadata);

        let metadata_bytes = x"a4626f70646d696e74647469636b646d6f766566616d6f756e74016a61747472696275746573a16a757365725f696e70757463787878";
        let metadata = cbor::to_map(metadata_bytes);
        let seed = vector::empty();
        let (is_valid, reason) = is_valid_bitseed_mint(&metadata, seed);
        simple_map::drop(metadata);

        assert!(!is_valid, 1);
        assert!(option::borrow(&reason) == &std::string::utf8(b"create wasm instance fail"), 1);
    }

    #[test]
    fun test_pack_inscribe_generate_args() {
        let deploy_args = x"8178377b22686569676874223a7b2274797065223a2272616e6765222c2264617461223a7b226d696e223a312c226d6178223a313030307d7d7d";
        let seed = b"0xe4b6de2407ad9455a364ba0227a8591631d1253508bc41f7d1992d218dd29b47";
        let user_input = b"";

        pack_inscribe_generate_args(deploy_args, seed, user_input);
    }
}