/// Bitseed inscribe inscription factory
module rooch_nursery::inscribe_factory {

    use std::vector;
    use std::option::{Self, Option};
    use std::string::{Self, String, utf8};
    use std::bcs;
    
    use moveos_std::address;
    use moveos_std::hash;
    use moveos_std::hex;
    use moveos_std::object::{Self, Object};
    use moveos_std::string_utils;
    use moveos_std::simple_map::{Self, SimpleMap};
    use moveos_std::wasm;
    use moveos_std::cbor;
    use moveos_std::result::{Self, Result, err, ok, is_err, as_err};

    use bitcoin_move::types;
    use bitcoin_move::ord::{Self, Inscription};
    use bitcoin_move::bitcoin;

    use rooch_nursery::bitseed::{Self, Bitseed};
    use rooch_nursery::tick_info;

    const BIT_SEED_DEPLOY: vector<u8> = b"bitseed_deploy";
    const BIT_SEED_MINT: vector<u8> = b"bitseed_mint";
    const BIT_SEED_GENERATOR_TICK: vector<u8> = b"generator";

    public fun bitseed_deploy_key(): vector<u8> {
        BIT_SEED_DEPLOY
    }

    public fun bitseed_mint_key(): vector<u8> {
        BIT_SEED_MINT
    }

    fun is_bitseed(inscription: &Inscription) : bool {
        let metaprotocol = ord::metaprotocol(inscription);
        option::is_some<String>(&metaprotocol) && option::borrow(&metaprotocol) == &bitseed::metaprotocol()
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

    fun is_valid_bitseed(metadata: &SimpleMap<String,vector<u8>>) : Result<bool> {
        let tick = get_SFT_tick_option(metadata);
        if (option::is_none(&tick)) {
            return err(b"metadata.tick is required")
        };

        let tick_len = std::string::length(option::borrow(&tick));
        if (tick_len < 4 || tick_len > 32) {
            return err(b"metadata.tick must be 4-32 characters")
        };

        let amount = get_SFT_amount_option(metadata);
        if (option::is_none(&amount)) {
            return err(b"metadata.amount is required")
        };
        ok(true)
    }

    fun is_valid_bitseed_deploy(metadata: &SimpleMap<String,vector<u8>>) : Result<bool> {
        let is_valid_result = is_valid_bitseed(metadata);
        if (is_err(&is_valid_result)) {
            return is_valid_result
        };

        let attributes = get_SFT_attributes(metadata);

        let generator = get_SFT_string_attribute(&attributes, b"generator");
        let factory = get_SFT_string_attribute(&attributes, b"factory");
        if (option::is_none(&generator) && option::is_none(&factory)) {
            simple_map::drop(attributes);
            return err(b"metadata.attributes.generator or metadata.attributes.factory is required")
        };

        if (option::is_some(&generator) && option::is_some(&factory)) {
            simple_map::drop(attributes);
            return err(b"metadata.attributes.generator and metadata.attributes.factory can not exist at the same time")
        };
        if (option::is_some(&generator)) {
            let is_valid_result= is_valid_generator_uri(option::borrow(&generator));
            if (is_err(&is_valid_result)) {
                simple_map::drop(attributes);
                return is_valid_result
            };
        };
        if (option::is_some(&factory)) {
            //TODO check factory
        };

        simple_map::drop(attributes);
        ok(true)
    }

    fun is_valid_generator_uri(generator_uri: &String) : Result<bool> {
        let index = string::index_of(generator_uri, &std::string::utf8(b"/inscription/"));
        if (index != 0) {
            return err(b"metadata.attributes.generator not start with /inscription/")
        };

        let inscription_id_str = string::sub_string(generator_uri, vector::length(&b"/inscription/"), string::length(generator_uri));
        let inscription_id_option = ord::parse_inscription_id(&inscription_id_str);
        if (option::is_none(&inscription_id_option)) {
            return err(b"metadata.attributes.generator inscription_id parse fail")
        };

        let inscription_id = option::extract(&mut inscription_id_option);
        if (!ord::exists_inscription(inscription_id)) {
            return err(b"metadata.attributes.generator inscription not exists")
        };

        if (!ord::exists_metaprotocol_validity(inscription_id)) {
            return err(b"metadata.attributes.generator inscription metaprotocol validity not exists")
        };

        let metaprotocol_validity = ord::borrow_metaprotocol_validity(inscription_id);

        let is_match = ord::metaprotocol_validity_protocol_match<Bitseed>(metaprotocol_validity);
        if (!is_match) {
            return err(b"metadata.attributes.generator inscription metaprotocol validity protocol not match")
        };

        let is_valid = ord::metaprotocol_validity_is_valid(metaprotocol_validity);
        if (!is_valid) {
            return err(b"metadata.attributes.generator inscription metaprotocol validity not valid")
        };
        ok(true)
    }

    fun deploy_tick(metadata: &SimpleMap<String,vector<u8>>): (bool, Option<String>){
    
        let tick = get_SFT_tick(metadata);
        let tick = string_utils::to_lower_case(&tick);
        
        let max = get_SFT_amount(metadata);
        let attributes = get_SFT_attributes(metadata);

        let repeat = 0u64;
        let repeat_option = get_SFT_u64_attribute(&attributes, b"repeat");
        if (option::is_some(&repeat_option)) {
            repeat = option::destroy_some(repeat_option);
        };
        let generator_uri_option = get_SFT_string_attribute(&attributes, b"generator");
        
        let inscription_id_option = if (option::is_some(&generator_uri_option)) {
            let generator_uri = option::destroy_some(generator_uri_option);
            let inscription_id_str = string::sub_string(&generator_uri, vector::length(&b"/inscription/"), string::length(&generator_uri));
            ord::parse_inscription_id(&inscription_id_str)
        }else{
            option::none()
        };

        let factory_option = get_SFT_string_attribute(&attributes, b"factory");
        
        let has_user_input = true;
        let has_user_input_option = get_SFT_bool_attribute(&attributes, b"has_user_input");
        if (option::is_some(&has_user_input_option)) {
            has_user_input = option::destroy_some(has_user_input_option);
        };

        let deploy_args = get_SFT_bytes_attribute(&attributes, b"deploy_args");

        
        tick_info::deploy_tick(bitseed::metaprotocol(), tick, inscription_id_option, factory_option, max, repeat, has_user_input, deploy_args);

        (true, option::none<String>())
    }
    
    fun mint_bitseed(metadata: &SimpleMap<String,vector<u8>>, seed: vector<u8>, content_type: Option<String>, body: vector<u8>) : Result<Object<Bitseed>> {
        let is_valid_result = is_valid_bitseed(metadata);
        if (is_err(&is_valid_result)) {
            return as_err(is_valid_result)
        };

        let tick = get_SFT_tick(metadata);
        let attributes = get_SFT_attributes(metadata);
        let amount = get_SFT_amount(metadata);

        if (!tick_info::is_deployed(bitseed::metaprotocol(), tick)) {
            return err(b"the tick is not deployed")
        };
        let tick_info = tick_info::borrow_tick_info(bitseed::metaprotocol(), tick);
        let has_user_input = tick_info::has_user_input(tick_info);
        let deploy_args = option::destroy_with_default(tick_info::deploy_args(tick_info), vector::empty());

        let user_input = string::utf8(b"");
        if (has_user_input) {
            //TODO handle user input
            let user_input_option = get_SFT_string_attribute(&attributes, b"id");
            if (option::is_none(&user_input_option)) {
                return err(b"metadata.attributes.user_input is required")
            };

            user_input = *option::borrow(&user_input_option);
        };
        // The generator tick has no generator, so we skip the inscribe verify
        if(tick != utf8(BIT_SEED_GENERATOR_TICK)) {
            let generator_inscription_id_option = tick_info::generator(tick_info);
            if (option::is_none(&generator_inscription_id_option)) {
                return err(b"the tick can not mint on Bitcoin")
            };

            let generator_inscription_id = option::destroy_some(generator_inscription_id_option);
            if (!ord::exists_metaprotocol_validity(generator_inscription_id)) {
                return err(b"generator_inscription_id is not validity bitseed")
            };

            let generator_txid = ord::inscription_id_txid(&generator_inscription_id);
            let generator_index = ord::inscription_id_index(&generator_inscription_id);
            let inscription_obj = ord::borrow_inscription(generator_txid, generator_index);

            let inscrption = object::borrow(inscription_obj);
            let wasm_bytes = ord::body(inscrption);

            let (is_valid, reason) = inscribe_verify(wasm_bytes, deploy_args, seed, user_input, metadata, content_type, body);
            if (!is_valid) {
                return result::err_string(option::destroy_with_default(reason, utf8(b"inscribe verify fail")))
            };
        };
        let bitseed_result = tick_info::mint_on_bitcoin(bitseed::metaprotocol(), tick, amount);
        bitseed_result
    }

    public fun inscribe_verify(wasm_bytes: vector<u8>, deploy_args: vector<u8>,
            seed: vector<u8>, user_input: String, metadata: &SimpleMap<String,vector<u8>>, 
            content_type: Option<String>, body: vector<u8>): (bool, Option<String>) {
        let wasm_instance_option = wasm::create_wasm_instance_option(wasm_bytes);
        if (option::is_none(&wasm_instance_option)) {
            option::destroy_none(wasm_instance_option);
            return (false, option::some(std::string::utf8(b"create wasm instance fail")))
        };

        let wasm_instance = option::destroy_some(wasm_instance_option);
        let function_name = b"inscribe_verify";

        let buffer = pack_inscribe_generate_args(deploy_args, seed, user_input);

        let arg_with_length = wasm::add_length_with_data(buffer);

        let amount = get_SFT_amount(metadata);
        let attributes = get_SFT_attributes(metadata);
        let output_buffer = pack_inscribe_output_args(amount, attributes, content_type, body);

        let arg_list = vector::empty<vector<u8>>();
        vector::push_back(&mut arg_list, arg_with_length);
        vector::push_back(&mut arg_list, output_buffer);


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
        attrs: vector<u16>,
        seed: std::string::String,
        user_input: std::string::String,
    }

    fun pack_inscribe_generate_args(deploy_args: vector<u8>, seed: vector<u8>, user_input: String): vector<u8>{
        let attrs = vector::empty();

        let i=0;
        let len = vector::length(&deploy_args);
        while (i < len) {
            vector::push_back(&mut attrs, (*vector::borrow(&deploy_args, i) as u16));
            i = i + 1;
        };

        let seed_hex = hex::encode(seed);
        let args = InscribeGenerateArgs{
            attrs: attrs,
            seed: string::utf8(seed_hex),
            user_input: user_input
        };

        cbor::to_cbor(&args)
    }

    struct InscribeGenerateOutput has store {
        amount: u64,
        attributes: SimpleMap<String,vector<u8>>,
        content: SimpleMap<String,vector<u8>>,
    }

    fun pack_inscribe_output_args(amount: u64, attributes: SimpleMap<String,vector<u8>>, content_type_option: Option<String>, body: vector<u8>): vector<u8>{
        let content = simple_map::new();
        if (vector::length(&body) > 0) {
            if (option::is_some(&content_type_option)) {
                let content_type = option::destroy_some(content_type_option);
                simple_map::add(&mut content, string::utf8(b"content_type"), cbor::to_cbor(&content_type));
            };

            simple_map::add(&mut content, string::utf8(b"body"), cbor::to_cbor(&body));
        };
        
        let output = InscribeGenerateOutput{
            amount: amount,
            attributes: attributes,
            content: content,
        };

        let output_bytes = cbor::to_cbor(&output);

        let InscribeGenerateOutput{amount:_, attributes, content}=output;
        simple_map::drop(attributes);
        simple_map::drop(content);

        output_bytes
    }

    fun generate_seed_from_inscription(inscription: &Inscription): vector<u8> {
        let inscription_txid = ord::txid(inscription);

        // reveal tx
        let reveal_tx_option = bitcoin::get_tx(inscription_txid);
        if (option::is_none(&reveal_tx_option)) {
            return vector::empty()
        };

        let reveal_tx = option::destroy_some(reveal_tx_option);
        let reveal_input = types::tx_input(&reveal_tx);
        let reveal_index = ord::index(inscription);
        let reveal_txin = vector::borrow(reveal_input, (reveal_index as u64));
        let reveal_outpoint = types::txin_previous_output(reveal_txin);

        // commit tx
        let commit_txid = types::outpoint_txid(reveal_outpoint);
        let commit_vout = types::outpoint_vout(reveal_outpoint);
        let commit_tx_option = bitcoin::get_tx(commit_txid);
        if (option::is_none(&commit_tx_option)) {
            return vector::empty()
        };

        let commit_tx = option::destroy_some(commit_tx_option);
        let commit_input = types::tx_input(&commit_tx);
        let commit_txin = vector::borrow(commit_input, (commit_vout as u64));
        let commit_outpoint = types::txin_previous_output(commit_txin);

        // seed tx
        let seed_txid = types::outpoint_txid(commit_outpoint);
        let seed_vout = types::outpoint_vout(commit_outpoint);

        let seed_height_option = bitcoin::get_tx_height(seed_txid);
        if (option::is_none(&seed_height_option)) {
            return vector::empty()
        };

        let seed_height = *option::borrow(&seed_height_option);

        let seed_block_hash_option = bitcoin::get_block_hash_by_height(seed_height);
        if (option::is_none(&seed_block_hash_option)) {
            return vector::empty()
        };

        let seed_block_hash = *option::borrow(&seed_block_hash_option);
        let seed_hex = generate_seed_from_inscription_inner(seed_block_hash, seed_txid, seed_vout);

        seed_hex
    }

    fun generate_seed_from_inscription_inner(block_hash: address, txid: address, vout: u32) : vector<u8> {
        let buf = vector::empty();
        vector::append(&mut buf, address::to_bytes(&block_hash));
        vector::append(&mut buf, address::to_bytes(&txid));
        vector::append(&mut buf, bcs::to_bytes(&vout));
        hash::sha3_256(buf)
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
                    let is_valid_result = is_valid_bitseed_deploy(&metadata);
                    if (is_err(&is_valid_result)) {
                        let reason = result::unwrap_err(is_valid_result);
                        bitseed::seal_metaprotocol_validity(inscription_id, false, option::some(reason));

                        simple_map::drop(metadata);
                        return ()
                    };

                    let (ok, reason) = deploy_tick(&metadata);
                    if (!ok) {
                        bitseed::seal_metaprotocol_validity(inscription_id, false, reason);

                        simple_map::drop(metadata);
                        return ()
                    };

                    bitseed::seal_metaprotocol_validity(inscription_id, true, option::none());
                } else if (option::borrow(&op) == &string::utf8(b"mint")) {
                    let seed = generate_seed_from_inscription(inscription);
                    let content_type = ord::content_type(inscription);
                    let body = ord::body(inscription);

                    let bitseed_result = mint_bitseed(&metadata, seed, content_type, body);
                    if (is_err(&bitseed_result)) {
                        let reason = result::unwrap_err(bitseed_result);
                        bitseed::seal_metaprotocol_validity(inscription_id, false, option::some(reason));

                        simple_map::drop(metadata);
                        return ()
                    }else{
                        let bitseed_obj = result::unwrap(bitseed_result);
                        bitseed::seal_metaprotocol_validity(inscription_id, true, option::none());
                        bitseed::add_metaprotocol_attachment(inscription_id, bitseed_obj);
                    }
                } else if (option::borrow(&op) == &string::utf8(b"split")) {
                    bitseed::seal_metaprotocol_validity(inscription_id, true, option::none());
                } else if (option::borrow(&op) == &string::utf8(b"merge")) {
                    bitseed::seal_metaprotocol_validity(inscription_id, true, option::none());
                } else {
                    bitseed::seal_metaprotocol_validity(inscription_id, false, option::some(string::utf8(b"invalid op")));
                }
            } else {
                bitseed::seal_metaprotocol_validity(inscription_id, false, option::some(string::utf8(b"op not found")));
            };

            simple_map::drop(metadata)
        }
    }

    #[test_only]
    struct TestProtocol has key {}

    #[test(genesis_account=@0x4)]
    fun test_is_valid_bitseed_deploy_ok(genesis_account: &signer){
        let (_test_address, test_inscription_id) = ord::setup_inscription_for_test<Bitseed>(genesis_account, bitseed::metaprotocol());
        bitseed::seal_metaprotocol_validity(test_inscription_id, true, option::none());

        let metadata_bytes = x"a4626f70666465706c6f79647469636b646d6f766566616d6f756e74016a61747472696275746573a16967656e657261746f72784f2f696e736372697074696f6e2f373764666332666535393834313962303036343163323936313831613936636631363934333639376635373334383062303233623737636365383261646132316930";
        let metadata = cbor::to_map(metadata_bytes);
        let is_valid_result = is_valid_bitseed_deploy(&metadata);
        simple_map::drop(metadata);

        assert!(is_ok(&is_valid_result), 1);
        //assert!(option::is_none(&reason), 1);
    }

    #[test]
    fun test_is_valid_bitseed_deploy_fail_for_tick_not_found(){
        let metadata_bytes = x"a4626f70666465706c6f79647469636bf766616d6f756e74016a61747472696275746573a16967656e657261746f72784f2f696e736372697074696f6e2f653839633162343830356538626235303236323038373632326263656662383533343232356364376138633264343832366433366630633161653333303831316931";
        let metadata = cbor::to_map(metadata_bytes);
        let is_valid_result = is_valid_bitseed_deploy(&metadata);
        simple_map::drop(metadata);

        assert!(is_err(&is_valid_result), 1);
        assert!(result::unwrap_err(is_valid_result) == std::string::utf8(b"metadata.tick is required"), 1);
    }

    #[test]
    fun test_is_valid_bitseed_deploy_fail_for_tick_too_short(){
        let metadata_bytes = x"a4626f70666465706c6f79647469636b6378787866616d6f756e74016a61747472696275746573a16967656e657261746f72784f2f696e736372697074696f6e2f653839633162343830356538626235303236323038373632326263656662383533343232356364376138633264343832366433366630633161653333303831316931";
        let metadata = cbor::to_map(metadata_bytes);
        let is_valid_result = is_valid_bitseed_deploy(&metadata);
        simple_map::drop(metadata);

        assert!(is_err(&is_valid_result), 1);
        assert!(result::unwrap_err(is_valid_result) == std::string::utf8(b"metadata.tick must be 4-32 characters"), 1);
    }

    #[test]
    fun test_is_valid_bitseed_deploy_fail_for_tick_too_long(){
        let metadata_bytes = x"a4626f70666465706c6f79647469636b78227878787878787878787878787878787878787878787878787878787878787878787866616d6f756e74016a61747472696275746573a16967656e657261746f72784f2f696e736372697074696f6e2f653839633162343830356538626235303236323038373632326263656662383533343232356364376138633264343832366433366630633161653333303831316931";
        let metadata = cbor::to_map(metadata_bytes);
        let is_valid_result = is_valid_bitseed_deploy(&metadata);
        simple_map::drop(metadata);

        assert!(is_err(&is_valid_result), 1);
        assert!(result::unwrap_err(is_valid_result) == std::string::utf8(b"metadata.tick must be 4-32 characters"), 1);
    }

    #[test]
    fun test_is_valid_bitseed_deploy_fail_for_amount_not_found(){
        let metadata_bytes = x"a4626f70666465706c6f79647469636b68746573745469636b66616d6f756e74f76a61747472696275746573a16967656e657261746f72784f2f696e736372697074696f6e2f653839633162343830356538626235303236323038373632326263656662383533343232356364376138633264343832366433366630633161653333303831316931";
        let metadata = cbor::to_map(metadata_bytes);
        let is_valid_result = is_valid_bitseed_deploy(&metadata);
        simple_map::drop(metadata);

        assert!(is_err(&is_valid_result), 1);
        assert!(result::unwrap_err(is_valid_result) == std::string::utf8(b"metadata.amount is required"), 1);
    }


    #[test]
    fun test_is_valid_bitseed_deploy_fail_for_generator_not_found(){
        let metadata_bytes = x"a4626f70666465706c6f79647469636b68746573745469636b66616d6f756e74016a61747472696275746573a0";
        let metadata = cbor::to_map(metadata_bytes);
        let is_valid_result = is_valid_bitseed_deploy(&metadata);
        simple_map::drop(metadata);
        assert!(is_err(&is_valid_result), 1);
        assert!(result::unwrap_err(is_valid_result) == std::string::utf8(b"metadata.attributes.generator or metadata.attributes.factory is required"), 1);
    }

    #[test]
    fun test_is_valid_bitseed_deploy_fail_for_generator_uri_not_start_with_generator(){
        let metadata_bytes = x"a4626f70666465706c6f79647469636b646d6f766566616d6f756e74016a61747472696275746573a16967656e657261746f7278472f7878782f653839633162343830356538626235303236323038373632326263656662383533343232356364376138633264343832366433366630633161653333303831316931";
        let metadata = cbor::to_map(metadata_bytes);
        let is_valid_result = is_valid_bitseed_deploy(&metadata);
        simple_map::drop(metadata);

        assert!(is_err(&is_valid_result), 1);
        assert!(result::unwrap_err(is_valid_result) == std::string::utf8(b"metadata.attributes.generator not start with /inscription/"), 1);
    }

    #[test]
    fun test_is_valid_bitseed_deploy_fail_for_parse_inscription_id_fail(){
        let metadata_bytes = x"a4626f70666465706c6f79647469636b646d6f766566616d6f756e74016a61747472696275746573a16967656e657261746f72784f2f696e736372697074696f6e2f377864666332666535393834313962303036343163323936313831613936636631363934333639376635373334383062303233623737636365383261646132316930";
        let metadata = cbor::to_map(metadata_bytes);
        let is_valid_result = is_valid_bitseed_deploy(&metadata);
        simple_map::drop(metadata);

        assert!(is_err(&is_valid_result), 1);
        assert!(result::unwrap_err(is_valid_result) == std::string::utf8(b"metadata.attributes.generator inscription_id parse fail"), 1);
    }

    #[test(genesis_account=@0x4)]
    fun test_is_valid_bitseed_deploy_fail_for_inscription_no_exists(genesis_account: &signer){
        ord::init_for_test(genesis_account);

        let metadata_bytes = x"a4626f70666465706c6f79647469636b646d6f766566616d6f756e74016a61747472696275746573a16967656e657261746f72784f2f696e736372697074696f6e2f373764666332666535393834313962303036343163323936313831613936636631363934333639376635373334383062303233623737636365383261646132316930";
        let metadata = cbor::to_map(metadata_bytes);
        let is_valid_result = is_valid_bitseed_deploy(&metadata);
        simple_map::drop(metadata);

        assert!(is_err(&is_valid_result), 1);
        assert!(result::unwrap_err(is_valid_result) == std::string::utf8(b"metadata.attributes.generator inscription not exists"), 1);
    }

    #[test(genesis_account=@0x4)]
    fun test_is_valid_bitseed_deploy_fail_for_inscription_metaprotocol_validity_not_exists(genesis_account: &signer){
        let (_test_address, _test_inscription_id) = ord::setup_inscription_for_test<Bitseed>(genesis_account, bitseed::metaprotocol());

        let metadata_bytes = x"a4626f70666465706c6f79647469636b646d6f766566616d6f756e74016a61747472696275746573a16967656e657261746f72784f2f696e736372697074696f6e2f373764666332666535393834313962303036343163323936313831613936636631363934333639376635373334383062303233623737636365383261646132316930";
        let metadata = cbor::to_map(metadata_bytes);
        let is_valid_result = is_valid_bitseed_deploy(&metadata);
        simple_map::drop(metadata);

        assert!(is_err(&is_valid_result), 1);
        assert!(result::unwrap_err(is_valid_result) == std::string::utf8(b"metadata.attributes.generator inscription metaprotocol validity not exists"), 1);
    }

    #[test(genesis_account=@0x4)]
    fun test_is_valid_bitseed_deploy_fail_for_inscription_metaprotocol_validity_protocol_not_match(genesis_account: &signer){
        let (_test_address, test_inscription_id) = ord::setup_inscription_for_test<TestProtocol>(genesis_account, string::utf8(b"TestProtocol"));
        ord::seal_metaprotocol_validity<TestProtocol>(test_inscription_id, true, option::none());

        let metadata_bytes = x"a4626f70666465706c6f79647469636b646d6f766566616d6f756e74016a61747472696275746573a16967656e657261746f72784f2f696e736372697074696f6e2f373764666332666535393834313962303036343163323936313831613936636631363934333639376635373334383062303233623737636365383261646132316930";
        let metadata = cbor::to_map(metadata_bytes);
        let is_valid_result = is_valid_bitseed_deploy(&metadata);
        simple_map::drop(metadata);

        assert!(is_err(&is_valid_result), 1);
        assert!(result::unwrap_err(is_valid_result) == std::string::utf8(b"metadata.attributes.generator inscription metaprotocol validity protocol not match"), 1);
    }

    #[test(genesis_account=@0x4)]
    fun test_is_valid_bitseed_deploy_fail_for_inscription_metaprotocol_validity_not_valid(genesis_account: &signer){
        let (_test_address, test_inscription_id) = ord::setup_inscription_for_test<Bitseed>(genesis_account, bitseed::metaprotocol());
        bitseed::seal_metaprotocol_validity(test_inscription_id, false, option::none());

        let metadata_bytes = x"a4626f70666465706c6f79647469636b646d6f766566616d6f756e74016a61747472696275746573a16967656e657261746f72784f2f696e736372697074696f6e2f373764666332666535393834313962303036343163323936313831613936636631363934333639376635373334383062303233623737636365383261646132316930";
        let metadata = cbor::to_map(metadata_bytes);
        let is_valid_result = is_valid_bitseed_deploy(&metadata);
        simple_map::drop(metadata);

        assert!(is_err(&is_valid_result), 1);
        assert!(result::unwrap_err(is_valid_result) == std::string::utf8(b"metadata.attributes.generator inscription metaprotocol validity not valid"), 1);
    }

    #[test(genesis_account=@0x4)]
    fun test_deploy_tick_ok(genesis_account: &signer){
        tick_info::init_for_testing();

        let (_test_address, test_inscription_id) = ord::setup_inscription_for_test<Bitseed>(genesis_account, bitseed::metaprotocol());
        bitseed::seal_metaprotocol_validity(test_inscription_id, true, option::none());

        let metadata_bytes = x"a4626f70666465706c6f79647469636b646d6f766566616d6f756e74016a61747472696275746573a16967656e657261746f72784f2f696e736372697074696f6e2f373764666332666535393834313962303036343163323936313831613936636631363934333639376635373334383062303233623737636365383261646132316930";
        let metadata = cbor::to_map(metadata_bytes);
        let (ok, reason) = deploy_tick(&metadata);
        simple_map::drop(metadata);

        assert!(ok, 1);
        assert!(option::is_none(&reason), 1);

        assert!(tick_info::is_deployed(bitseed::metaprotocol(), string::utf8(b"move")), 2);
    
        let tick_info = tick_info::borrow_tick_info(bitseed::metaprotocol(), string::utf8(b"move"));

        // check tick
        let tick = tick_info::tick(tick_info);
        assert!(tick == string::utf8(b"move"), 3);

        // check generator
        let generator_option = tick_info::generator(tick_info);
        assert!(option::is_some(&generator_option), 4);

        let generator = option::destroy_some(generator_option);
        let test_txid = @0x21da2ae8cc773b020b4873f597369416cf961a1896c24106b0198459fec2df77;
        let test_index = 0;
        assert!(ord::inscription_id_txid(&generator) == test_txid, 5);
        assert!(ord::inscription_id_index(&generator) == test_index, 6);

        // check max
        let max = tick_info::max(tick_info);
        assert!(max == 1u64, 6);

        // check repeat
        let repeat = tick_info::repeat(tick_info);
        assert!(repeat == 0, 7);

        // check has_user_input
        let has_user_input = tick_info::has_user_input(tick_info);
        assert!(has_user_input, 8);

        // check deploy_args
        let deploy_args = tick_info::deploy_args(tick_info);
        assert!(option::is_none(&deploy_args), 9)
    }

    #[test_only]
    use moveos_std::features;
    #[test_only]
    use moveos_std::result::{is_ok};

    #[test(genesis_account=@0x4)]
    fun test_is_valid_bitseed_mint_fail_with_tick_not_deploy(genesis_account: &signer){
        tick_info::init_for_testing();

        let (_test_address, test_inscription_id) = ord::setup_inscription_for_test<Bitseed>(genesis_account, bitseed::metaprotocol());
        bitseed::seal_metaprotocol_validity(test_inscription_id, true, option::none());

        let metadata_bytes = x"a4626f70666465706c6f79647469636b646d6f766566616d6f756e74016a61747472696275746573a16967656e657261746f72784f2f696e736372697074696f6e2f373764666332666535393834313962303036343163323936313831613936636631363934333639376635373334383062303233623737636365383261646132316930";
        let metadata = cbor::to_map(metadata_bytes);
        let seed = vector::empty();
        let content_type = option::none();
        let body = vector::empty();
        let bitseed_result = mint_bitseed(&metadata, seed, content_type, body);
        simple_map::drop(metadata);

        assert!(is_err(&bitseed_result), 1);
        assert!(result::unwrap_err(bitseed_result) == std::string::utf8(b"the tick is not deployed"), 1);
    }

    #[test(genesis_account=@0x4)]
    fun test_is_valid_bitseed_mint_fail_with_maximum_supply_exceeded(genesis_account: &signer){
        tick_info::init_for_testing();

        let (_test_address, test_inscription_id) = ord::setup_inscription_for_test<Bitseed>(genesis_account, bitseed::metaprotocol());
        bitseed::seal_metaprotocol_validity(test_inscription_id, true, option::none());

        let metadata_bytes = x"a4626f70666465706c6f79647469636b646d6f766566616d6f756e74016a61747472696275746573a16967656e657261746f72784f2f696e736372697074696f6e2f373764666332666535393834313962303036343163323936313831613936636631363934333639376635373334383062303233623737636365383261646132316930";
        let metadata = cbor::to_map(metadata_bytes);
        let is_valid_result = is_valid_bitseed_deploy(&metadata);
        
        assert!(is_ok(&is_valid_result), 1);

        let (ok, reason) = deploy_tick(&metadata);
        assert!(ok, 1);
        assert!(option::is_none(&reason), 1);

        simple_map::drop(metadata);

        let metadata_bytes = x"a4626f70646d696e74647469636b646d6f766566616d6f756e7418646a61747472696275746573a16967656e657261746f72784f2f696e736372697074696f6e2f377864666332666535393834313962303036343163323936313831613936636631363934333639376635373334383062303233623737636365383261646132316930";
        let metadata = cbor::to_map(metadata_bytes);
        let seed = vector::empty();
        let content_type = option::none();
        let body = vector::empty();
        let mint_result = mint_bitseed(&metadata, seed, content_type, body);
        simple_map::drop(metadata);

        assert!(is_err(&mint_result), 1);
        //std::debug::print(&mint_result);
        //TODO FIXME
        let _err = result::unwrap_err(mint_result);
        //assert!(result::unwrap_err(mint_result) == std::string::utf8(b"maximum supply exceeded"), 1);
    }

    #[test(genesis_account=@0x4)]
    fun test_is_valid_bitseed_mint_fail_with_user_input_is_required(genesis_account: &signer){
        tick_info::init_for_testing();

        let (_test_address, test_inscription_id) = ord::setup_inscription_for_test<Bitseed>(genesis_account, bitseed::metaprotocol());
        bitseed::seal_metaprotocol_validity(test_inscription_id, true, option::none());

        let metadata_bytes = x"a4626f70666465706c6f79647469636b646d6f766566616d6f756e741927106a61747472696275746573a466726570656174056967656e657261746f72784f2f696e736372697074696f6e2f3737646663326665353938343139623030363431633239363138316139366366313639343336393766353733343830623032336237376363653832616461323169306e6861735f757365725f696e707574f56b6465706c6f795f617267738178377b22686569676874223a7b2274797065223a2272616e6765222c2264617461223a7b226d696e223a312c226d6178223a313030307d7d7d";
        let metadata = cbor::to_map(metadata_bytes);
        let is_valid_result = is_valid_bitseed_deploy(&metadata);

        assert!(is_ok(&is_valid_result), 1);

        let (ok, reason) = deploy_tick(&metadata);
        assert!(ok, 1);
        assert!(option::is_none(&reason), 1);

        simple_map::drop(metadata);

        let metadata_bytes = x"a4626f70646d696e74647469636b646d6f766566616d6f756e7418646a61747472696275746573a16967656e657261746f72784f2f696e736372697074696f6e2f377864666332666535393834313962303036343163323936313831613936636631363934333639376635373334383062303233623737636365383261646132316930";
        let metadata = cbor::to_map(metadata_bytes);
        let seed = vector::empty();
        let content_type = option::none();
        let body = vector::empty();
        let mint_result = mint_bitseed(&metadata, seed, content_type, body);
        simple_map::drop(metadata);

        assert!(is_err(&mint_result), 1);
        assert!(result::unwrap_err(mint_result) == std::string::utf8(b"metadata.attributes.user_input is required"), 1);
    }

    
    #[test(genesis_account=@0x4)]
    fun test_is_valid_bitseed_mint_fail_with_wasm_verify_fail(genesis_account: &signer){
        features::init_and_enable_all_features_for_test();
        tick_info::init_for_testing();

        let (_test_address, test_inscription_id) = ord::setup_inscription_for_test<Bitseed>(genesis_account, bitseed::metaprotocol());
        bitseed::seal_metaprotocol_validity(test_inscription_id, true, option::none());

        let metadata_bytes = x"a4626f70666465706c6f79647469636b646d6f766566616d6f756e741927106a61747472696275746573a466726570656174056967656e657261746f72784f2f696e736372697074696f6e2f3737646663326665353938343139623030363431633239363138316139366366313639343336393766353733343830623032336237376363653832616461323169306e6861735f757365725f696e707574f56b6465706c6f795f617267738178377b22686569676874223a7b2274797065223a2272616e6765222c2264617461223a7b226d696e223a312c226d6178223a313030307d7d7d";
        let metadata = cbor::to_map(metadata_bytes);
        let is_valid_result = is_valid_bitseed_deploy(&metadata);

        assert!(is_ok(&is_valid_result), 1);

        let (ok, reason) = deploy_tick(&metadata);
        assert!(ok, 1);
        assert!(option::is_none(&reason), 1);

        simple_map::drop(metadata);

        let metadata_bytes = x"a4626f70646d696e74647469636b646d6f766566616d6f756e74016a61747472696275746573a16a757365725f696e70757463787878";
        let metadata = cbor::to_map(metadata_bytes);
        let seed = vector::empty();
        let content_type = option::none();
        let body = vector::empty();
        let mint_result = mint_bitseed(&metadata, seed, content_type, body);
        simple_map::drop(metadata);

        assert!(is_err(&mint_result), 1);
        let _err = result::unwrap_err(mint_result);
        //std::debug::print();
        //assert!(result::unwrap_err(mint_result) == std::string::utf8(b"wasm verify fail"), 1);
    }

    #[test]
    fun test_pack_inscribe_generate_args() {
        let deploy_args = x"8178377b22686569676874223a7b2274797065223a2272616e6765222c2264617461223a7b226d696e223a312c226d6178223a313030307d7d7d";
        let seed = b"0xe4b6de2407ad9455a364ba0227a8591631d1253508bc41f7d1992d218dd29b47";
        let user_input = string::utf8(b"");

        pack_inscribe_generate_args(deploy_args, seed, user_input);
    }


    #[test]
    fun test_generate_seed_from_inscription_inner() {
        let block_hash = address::from_bytes(x"89753cc1cdc61a89d49d5b267ab8353d4e984e08cda587f54e813add2b6d207c");
        let txid = address::from_bytes(x"1a49883e4248bd8b2e423af8157a1795cd457ece0eb4d1f453266874dc1da262");
        let vout = 1;

        let seed = generate_seed_from_inscription_inner(block_hash, txid, vout);
        let hex_seed = hex::encode(seed);
        assert!(hex_seed == b"1700b4e1d726ef40b2832eb1d5f91fd88d36ddf79eb235789c9b417c997279bc", 1);
    }


    #[test]
    fun test_pack_inscribe_output_args() {
        let amount = 1u64;
        let attributes = simple_map::new();

        let height = 444u64;
        simple_map::add(&mut attributes, string::utf8(b"height"), cbor::to_cbor(&height));

        let id = string::utf8(b"test user input");
        simple_map::add(&mut attributes, string::utf8(b"id"), cbor::to_cbor(&id));

        let content_type = option::some(string::utf8(b"text/plain"));
        let body =x"68656c6c6f20776f726c6421";

        let output_bytes = pack_inscribe_output_args(amount, attributes, content_type, body);
        let output_hex = hex::encode(output_bytes);
        assert!(output_hex == b"a366616d6f756e74016a61747472696275746573a2666865696768741901bc6269646f74657374207573657220696e70757467636f6e74656e74a26c636f6e74656e745f747970656a746578742f706c61696e64626f64794c68656c6c6f20776f726c6421", 1);
    }

    #[test]
    fun test_pack_inscribe_output_args_without_content() {
        let amount = 1u64;
        let attributes = simple_map::new();

        let height = 444u64;
        simple_map::add(&mut attributes, string::utf8(b"height"), cbor::to_cbor(&height));

        let id = string::utf8(b"test user input");
        simple_map::add(&mut attributes, string::utf8(b"id"), cbor::to_cbor(&id));

        let content_type = option::some(string::utf8(b"text/plain"));
        let body = vector::empty();

        let output_bytes = pack_inscribe_output_args(amount, attributes, content_type, body);
        let output_hex = hex::encode(output_bytes);
        assert!(output_hex == b"a366616d6f756e74016a61747472696275746573a2666865696768741901bc6269646f74657374207573657220696e70757467636f6e74656e74a0", 1);
    }
}