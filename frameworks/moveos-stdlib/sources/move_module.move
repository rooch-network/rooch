// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

/// `move_module` wraps module bytes and provides some basic functions for handle Move module in Move.
module moveos_std::move_module {
    use std::vector;
    use std::string::{Self, String};
    use moveos_std::features;

    friend moveos_std::module_store;
    
    /// Module address is not the same as the signer
    const ErrorAddressNotMatchWithSigner: u64 = 1;
    /// Module verification error
    const ErrorModuleVerificationError: u64 = 2;
    /// Module incompatible with the old ones.
    const ErrorModuleIncompatible: u64 = 3;
    /// Vector length not match
    const ErrorLengthNotMatch: u64 = 4;
    
    struct MoveModule has copy, store, drop {
        byte_codes: vector<u8>,
    }

    public fun new(byte_codes: vector<u8>) : MoveModule {
        //TODO quick check the Magic number to test if it is Move bytecode
        MoveModule {
            byte_codes,
        }
    }

    public fun new_batch(byte_codes_batch: vector<vector<u8>>): vector<MoveModule> {
        let modules = vector::empty<MoveModule>();
        let i = 0u64;
        let len = vector::length(&byte_codes_batch);
        while (i < len) {
            vector::push_back(&mut modules, MoveModule {
                byte_codes: vector::pop_back(&mut byte_codes_batch),
            });
            i = i + 1;
        };
        vector::destroy_empty(byte_codes_batch);
        vector::reverse(&mut modules);
        modules
    }

    public(friend) fun into_byte_codes_batch(modules: vector<MoveModule>): vector<vector<u8>> {
        let bytes_vec = vector::empty<vector<u8>>();
        let i = 0u64;
        let len = vector::length(&modules);
        while (i < len) {
            vector::push_back(&mut bytes_vec, vector::pop_back(&mut modules).byte_codes);
            i = i + 1;
        };    
        vector::destroy_empty(modules);
        vector::reverse(&mut bytes_vec);
        bytes_vec
    }

    public fun module_id(move_module: &MoveModule): String {
        module_id_inner(&move_module.byte_codes)
    }

    /// Sort modules by dependency order and then verify. 
    /// Return their names and names of the modules with init function if sorted dependency order.
    /// This function will ensure the module's bytecode is valid and the module id is matching the module object address.
    /// Return
    ///     1. Module names of all the modules. Order of names is not matching the input, but sorted by module dependency order
    ///     2. Module names of the modules with init function.
    ///     3. Indices in input modules of each sorted modules.
    public fun sort_and_verify_modules(
        modules: &vector<MoveModule>, account_address: address
    ): (vector<String>, vector<String>, vector<u64>) {
        let bytes_vec = vector::empty<vector<u8>>();
        let i = 0u64;
        let len = vector::length(modules);
        while (i < len) {
            vector::push_back(&mut bytes_vec, vector::borrow(modules, i).byte_codes);
            i = i + 1;
        };
        sort_and_verify_modules_inner(bytes_vec, account_address)
    }

    /// Check module compatibility when upgrading
    /// Abort if the new module is not compatible with the old module.
    public fun check_comatibility(new_module: &MoveModule, old_module: &MoveModule) {
        check_compatibililty_inner(new_module.byte_codes, old_module.byte_codes);
    }

    // TODO: add more tests
    /// Binding given module's address to the new address
    public fun binding_module_address(
        modules: vector<MoveModule>,
        old_address: address,
        new_address: address,
    ): vector<MoveModule> {
        features::ensure_module_template_enabled();

        let bytes_vec = into_byte_codes_batch(modules);

        let old_addresses = vector::singleton(old_address);
        let new_addresses = vector::singleton(new_address);
        
        let rebinded_bytes = replace_address_identifiers(bytes_vec, old_addresses, new_addresses);
        let rebinded_bytes = replace_addresses_constant(rebinded_bytes, old_addresses, new_addresses);
        
        new_batch(rebinded_bytes)
    }

    // TODO: add tests
    /// Replace given module's identifier to the new ones
    public fun replace_module_identiner (
        modules: vector<MoveModule>,
        old_names: vector<String>,
        new_names: vector<String>,
    ): vector<MoveModule> {
        features::ensure_module_template_enabled();

        assert!(
            vector::length(&old_names) == vector::length(&new_names),
            ErrorLengthNotMatch
        );
        let bytes_vec = into_byte_codes_batch(modules);        
        let rebinded_bytes = replace_identifiers(bytes_vec, old_names, new_names);        
        new_batch(rebinded_bytes)
    }

    // TODO: add more tests
    /// Replace given struct's identifier to the new ones
    public fun replace_struct_identifier(
        modules: vector<MoveModule>,
        old_names: vector<String>,
        new_names: vector<String>,
    ): vector<MoveModule> {
        features::ensure_module_template_enabled();
        replace_module_identiner(modules, old_names, new_names)
    }

    // TODO: add more tests
    /// Replace given string constant to the new ones
    public fun replace_constant_string(
        modules: vector<MoveModule>,
        old_strings: vector<String>,
        new_strings: vector<String>,
    ): vector<MoveModule> {
        features::ensure_devnet_enabled();
        assert!(
            vector::length(&old_strings) == vector::length(&new_strings),
            ErrorLengthNotMatch
        );
        let bytes_vec = into_byte_codes_batch(modules);

        let old_str_bytes = vector::empty<vector<u8>>();
        let new_str_bytes = vector::empty<vector<u8>>();
        let i = 0u64;
        let len = vector::length(&old_strings);
        while (i < len) {
            vector::push_back(&mut old_str_bytes, *string::bytes(vector::borrow(&old_strings, i)));
            vector::push_back(&mut new_str_bytes, *string::bytes(vector::borrow(&new_strings, i)));
            i = i + 1;
        };
        let rebinded_bytes = replace_bytes_constant(bytes_vec, old_str_bytes, new_str_bytes);
        new_batch(rebinded_bytes)
    }

    // TODO: add more tests
    /// Replace given address constant to the new ones
    public fun replace_constant_address(
        modules: vector<MoveModule>,
        old_addresses: vector<address>,
        new_addresses: vector<address>,
    ): vector<MoveModule> {
        features::ensure_module_template_enabled();

        assert!(
            vector::length(&old_addresses) == vector::length(&new_addresses),
            ErrorLengthNotMatch
        );
        let bytes_vec = into_byte_codes_batch(modules);
        let rebinded_bytes = replace_addresses_constant(bytes_vec, old_addresses, new_addresses);
        new_batch(rebinded_bytes)
    }

    // TODO: add more tests
    /// Replace given u8 constant to the new ones
    public fun replace_constant_u8(
        modules: vector<MoveModule>,
        old_u8s: vector<u8>,
        new_u8s: vector<u8>,
    ): vector<MoveModule> {
        features::ensure_devnet_enabled();
        assert!(
            vector::length(&old_u8s) == vector::length(&new_u8s),
            ErrorLengthNotMatch
        );
        let bytes_vec = into_byte_codes_batch(modules);
        let rebinded_bytes = replace_u8_constant(bytes_vec, old_u8s, new_u8s);
        new_batch(rebinded_bytes)
    }

    // TODO: add more tests
    /// Replace given u64 constant to the new ones
    public fun replace_constant_u64(
        modules: vector<MoveModule>,
        old_u64s: vector<u64>,
        new_u64s: vector<u64>,
    ): vector<MoveModule> {
        features::ensure_devnet_enabled();
        assert!(
            vector::length(&old_u64s) == vector::length(&new_u64s),
            ErrorLengthNotMatch
        );
        let bytes_vec = into_byte_codes_batch(modules);
        let rebinded_bytes = replace_u64_constant(bytes_vec, old_u64s, new_u64s);
        new_batch(rebinded_bytes)
    }

    // TODO: add more tests
    /// Replace given u256 constant to the new ones
    public fun replace_constant_u256(
        modules: vector<MoveModule>,
        old_u256s: vector<u256>,
        new_u256s: vector<u256>,
    ): vector<MoveModule> {
        features::ensure_devnet_enabled();
        assert!(
            vector::length(&old_u256s) == vector::length(&new_u256s),
            ErrorLengthNotMatch
        );
        let bytes_vec = into_byte_codes_batch(modules);
        let rebinded_bytes = replace_u256_constant(bytes_vec, old_u256s, new_u256s);
        new_batch(rebinded_bytes)
    }

    native fun module_id_inner(byte_codes: &vector<u8>): String;

    native public fun module_id_from_name(account: address, name: String): String;

    /// Sort modules by dependency order and then verify. 
    /// Return
    ///  The first vector is the module names of all the modules.
    ///  The second vector is the module names of the modules with init function.
    ///  The third vector is the indices in input modules of each sorted modules.
    native public(friend) fun sort_and_verify_modules_inner(modules: vector<vector<u8>>, account_address: address): (vector<String>, vector<String>, vector<u64>);
    
    /// Request to call the init functions of the given modules
    /// module_ids: ids of modules which have a init function
    native public(friend) fun request_init_functions(module_ids: vector<String>);

    native fun check_compatibililty_inner(new_bytecodes: vector<u8>, old_bytecodes: vector<u8>);

    /// Native function to replace addresses identifier in module binary where the length of
    /// `old_addresses` must equal to that of `new_addresses`.  
    native public(friend) fun replace_address_identifiers(
        bytes: vector<vector<u8>>,
        old_addresses: vector<address>,
        new_addresses: vector<address>,
    ): vector<vector<u8>>;

    /// Native function to replace the name identifier `old_name` to `new_name` in module binary.
    native public(friend) fun replace_identifiers(
        bytes: vector<vector<u8>>,
        old_idents: vector<String>,
        new_idents: vector<String>,
    ): vector<vector<u8>>;

    /// Native function to replace constant addresses in module binary where the length of
    /// `old_addresses` must equal to that of `new_addresses`.    
    native public(friend) fun replace_addresses_constant(
        bytes: vector<vector<u8>>,
        old_addresses: vector<address>,
        new_addresses: vector<address>,
    ): vector<vector<u8>>;

    /// Native function to replace constant bytes in module binary where the length of
    /// `old_bytes` must equal to that of `new_bytes`.    
    native public(friend) fun replace_bytes_constant(
        bytes: vector<vector<u8>>,
        old_bytes: vector<vector<u8>>,
        new_bytes: vector<vector<u8>>,
    ): vector<vector<u8>>;

    /// Native function to replace constant u8 in module binary where the length of
    /// `old_u8s` must equal to that of `new_u8s`.    
    native public(friend) fun replace_u8_constant(
        bytes: vector<vector<u8>>,
        old_u8s: vector<u8>,
        new_u8s: vector<u8>,
    ): vector<vector<u8>>;

    /// Native function to replace constant u64 in module binary where the length of
    /// `old_u64s` must equal to that of `new_u64s`.    
    native public(friend) fun replace_u64_constant(
        bytes: vector<vector<u8>>,
        old_u64s: vector<u64>,
        new_u64s: vector<u64>,
    ): vector<vector<u8>>;

    /// Native function to replace constant u256 in module binary where the length of
    /// `old_u256s` must equal to that of `new_u256s`.    
    native public(friend) fun replace_u256_constant(
        bytes: vector<vector<u8>>,
        old_u256s: vector<u256>,
        new_u256s: vector<u256>,
    ): vector<vector<u8>>;

    #[test_only]
    use moveos_std::signer;

    //The following is the bytes and hex of the compiled module: example/counter/sources/counter.move with account 0x42
    // Run the follow commands to get the bytecode of the module
    //./target/debug/rooch move build -p examples/counter -d
    //xxd -c 99999 -p examples/counter/build/counter/bytecode_modules/counter.mv
    #[test_only]
    const COUNTER_MV_BYTES: vector<u8> = x"a11ceb0b060000000b01000402040403082b04330605391c07557908ce0140068e02220ab002050cb502640d9903020000010100020c000003000000000400000000050100000006010000000700020001080506010c01090700010c010a0508010c0504060407040001060c01030107080001080001050107090002060c09000106090007636f756e746572076163636f756e7407436f756e74657208696e63726561736509696e6372656173655f04696e69740d696e69745f666f725f746573740576616c756513626f72726f775f6d75745f7265736f75726365106d6f76655f7265736f757263655f746f0f626f72726f775f7265736f757263650000000000000000000000000000000000000000000000000000000000000042000000000000000000000000000000000000000000000000000000000000000205200000000000000000000000000000000000000000000000000000000000000042000201070300010400000211010201010000030c070038000c000a00100014060100000000000000160b000f0015020200000000050b0006000000000000000012003801020301000000050b0006000000000000000012003801020401000000050700380210001402000000";

    #[test]
    fun test_get_module_id() {
        let module_bytes = COUNTER_MV_BYTES;
        let m: MoveModule = Self::new(module_bytes);
        let name = Self::module_id(&m);
        assert!(name == string::utf8(b"0x42::counter"), 101);
    }

    #[test(account=@0x42)]
    fun test_verify_modules(account: &signer) {
        let addr = signer::address_of(account);
        let module_bytes = COUNTER_MV_BYTES;
        let m: MoveModule = Self::new(module_bytes);
        let modules = vector::singleton(m);
        let (module_names, _module_names_with_init_fn, _indices) = Self::sort_and_verify_modules(&modules, addr);
        assert!(vector::length(&module_names)==1, 101);
        assert!(vector::borrow(&module_names, 0) == &string::utf8(b"counter"), 102);
    }

    #[test(account=@0x1314)]
    #[expected_failure(abort_code = 1, location = Self)]
    fun test_address_mismatch_failure(account: &signer) {
        let addr = signer::address_of(account);
        let module_bytes = COUNTER_MV_BYTES;
        let m: MoveModule = Self::new(module_bytes);
        let modules = vector::singleton(m);
        let (_module_names, _module_names_with_init_fn, _indices) = Self::sort_and_verify_modules(&modules, addr);
    }
    
    #[test(account=@0x42)]
    fun test_module_template(account: &signer) {
        features::init_feature_store_for_test();
        features::change_feature_flags_for_test(
            vector[features::get_module_template_feature(), features::get_devnet_feature()], 
            vector[]
        );
        let addr = signer::address_of(account);
        // The following is the bytes of module `examples/coins/sources/fixed_supply_coin.move` with account 0x42
        let ref_bytes: vector<u8> = x"a11ceb0b060000000b01000e020e24033250048201140596019c0107b202940208c604800106c605410a8706110c9806710d890702000001010202020303040305030600070c000008080002090c010001060d08010801050e0001080105130c01080101140700000a000100000b010100030f0304000210060701080611090a010c04120b01010c01150d0e0005160f1001080517110a010802181301010806190114010c06121501010c021a16130108021b130101080305040805080708080809120a080b080c050d0502060c070b020108010002050b0401080001060c010501080101070b020109000107090001080002070b02010b030109000f010b0401090002050b04010900030b040108000b02010b050108000b02010b03010800010a02010806030806080602010b02010b0501090002070b02010b050109000f010b05010800010b02010900010b02010b0301090002070b02010b030109000b040109000109001166697865645f737570706c795f636f696e06737472696e67066f626a656374067369676e6572126163636f756e745f636f696e5f73746f726504636f696e0a636f696e5f73746f726503465343085472656173757279064f626a6563740666617563657404696e69740b64756d6d795f6669656c6409436f696e53746f726504436f696e0a616464726573735f6f660a626f72726f775f6d7574087769746864726177076465706f73697408436f696e496e666f06537472696e6704757466380f72656769737465725f657874656e640b6d696e745f657874656e6409746f5f66726f7a656e116372656174655f636f696e5f73746f7265106e65775f6e616d65645f6f626a65637409746f5f73686172656400000000000000000000000000000000000000000000000000000000000000420000000000000000000000000000000000000000000000000000000000000001000000000000000000000000000000000000000000000000000000000000000200000000000000000000000000000000000000000000000000000000000000030201010f2000b4f9e4300000000000000000000000000000000000000000000000000000000a021211466978656420537570706c7920436f696e0a0204034653430002010c01010201060b02010b0301080000010400020d0b0011020c020b0138000f004a102700000000000000000000000000000000000000000000000000000000000038010c030b020b03380202010000000c170702110607031106070038030c010d01070138040c000b01380538060c020d020b0038070b0212013808380902010000";

        // The following is the bytes of compiled module: examples/module_template/template/sources/fixed_supply_coin_template.move
        //rooch move build -p examples/module_template/template
        //xxd -c 99999 -p examples/module_template/template/build/template/bytecode_modules/coin_module_identifier_placeholder.mv
        let module_bytes: vector<u8> = x"a11ceb0b060000000b01000e020e24033250048201140596019c0107b202c40208f604800106f605590acf06110ce006710dd10702000001010202020303040305030600070c000008080002090c010001060d08010801050e0001080105130c01080101140700000a000100000b010100030f0304000210060701080611090a010c04120b01010c01150d0e0005160f1001080517110a010802181301010806190114010c06121501010c021a16130108021b130101080305040805080708080809120a080b080c050d0502060c070b020108010002050b0401080001060c010501080101070b020109000107090001080002070b02010b030109000f010b0401090002050b04010900030b040108000b02010b050108000b02010b03010800010a02010806030806080602010b02010b0501090002070b02010b050109000f010b05010800010b02010900010b02010b0301090002070b02010b030109000b0401090001090022636f696e5f6d6f64756c655f6964656e7469666965725f706c616365686f6c64657206737472696e67066f626a656374067369676e6572126163636f756e745f636f696e5f73746f726504636f696e0a636f696e5f73746f726522434f494e5f5354525543545f4944454e5449464945525f504c414345484f4c444552085472656173757279064f626a6563740666617563657404696e69740b64756d6d795f6669656c6409436f696e53746f726504436f696e0a616464726573735f6f660a626f72726f775f6d7574087769746864726177076465706f73697408436f696e496e666f06537472696e6704757466380f72656769737465725f657874656e640b6d696e745f657874656e6409746f5f66726f7a656e116372656174655f636f696e5f73746f7265106e65775f6e616d65645f6f626a65637409746f5f736861726564deadeadeadeadeadeadeadeadeadeadeadeadeadeadeadeadeadeadeadeadead0000000000000000000000000000000000000000000000000000000000000001000000000000000000000000000000000000000000000000000000000000000200000000000000000000000000000000000000000000000000000000000000030201de0f20800283b61c0000000000000000000000000000000000000000000000000000000a021615434f494e5f4e414d455f504c414345484f4c4445520a021817434f494e5f53594d424f4c5f504c414345484f4c4445520002010c01010201060b02010b0301080000010400020d0b0011020c020b0138000f004a102700000000000000000000000000000000000000000000000000000000000038010c030b020b03380202010000000c170702110607031106070038030c010d01070138040c000b01380538060c020d020b0038070b0212013808380902010000";
        
        // replace symbol and name
        let modules = vector::singleton(Self::new(module_bytes));
        let new_strings = vector::empty<String>();
        vector::push_back(&mut new_strings, string::utf8(b"FSC"));
        vector::push_back(&mut new_strings, string::utf8(b"Fixed Supply Coin"));
        let old_strings = vector::empty<String>();
        vector::push_back(&mut old_strings, string::utf8(b"COIN_SYMBOL_PLACEHOLDER"));
        vector::push_back(&mut old_strings, string::utf8(b"COIN_NAME_PLACEHOLDER"));
        let modules = Self::replace_constant_string(modules, old_strings, new_strings);

        let new_names = vector::empty<String>();
        vector::push_back(&mut new_names, std::string::utf8(b"fixed_supply_coin"));
        vector::push_back(&mut new_names, std::string::utf8(b"FSC"));
        let old_names = vector::empty<String>();
        vector::push_back(&mut old_names, std::string::utf8(b"coin_module_identifier_placeholder"));
        vector::push_back(&mut old_names, std::string::utf8(b"COIN_STRUCT_IDENTIFIER_PLACEHOLDER"));
        let modules = Self::replace_module_identiner(modules, old_names, new_names);

        let new_address = addr;
        let old_address = @0xdeadeadeadeadeadeadeadeadeadeadeadeadeadeadeadeadeadeadeadeadead;
        let modules = Self::binding_module_address(modules, old_address, new_address);

        let new_supply = vector::singleton(210_000_000_000u256);
        let old_supply = vector::singleton(123_321_123_456u256);
        let modules = Self::replace_constant_u256(modules, old_supply, new_supply);

        let new_decimal = vector::singleton(1u8);
        let old_decimal = vector::singleton(222u8);
        let modules = Self::replace_constant_u8(modules, old_decimal, new_decimal);

        let module_bytes = vector::borrow(&modules, 0).byte_codes;
        // compare the remapped modules bytes
        assert!(std::compare::cmp_bcs_bytes(&module_bytes, &ref_bytes) == 0u8, 1);
    }


    #[test(_account=@0x42)]
    #[expected_failure(abort_code = 2, location = moveos_std::features)]
    fun test_module_template_with_feature_disabled(_account: &signer) {
        features::init_feature_store_for_test();
        // The following is the bytes of compiled module: examples/module_template/template/sources/fixed_supply_coin_template.move
        //rooch move build -p examples/module_template/template
        //xxd -c 99999 -p examples/module_template/template/build/template/bytecode_modules/coin_module_identifier_placeholder.mv
        let module_bytes: vector<u8> = x"a11ceb0b060000000b01000e020e24033250048201140596019c0107b202c40208f604800106f605590acf06110ce006710dd10702000001010202020303040305030600070c000008080002090c010001060d08010801050e0001080105130c01080101140700000a000100000b010100030f0304000210060701080611090a010c04120b01010c01150d0e0005160f1001080517110a010802181301010806190114010c06121501010c021a16130108021b130101080305040805080708080809120a080b080c050d0502060c070b020108010002050b0401080001060c010501080101070b020109000107090001080002070b02010b030109000f010b0401090002050b04010900030b040108000b02010b050108000b02010b03010800010a02010806030806080602010b02010b0501090002070b02010b050109000f010b05010800010b02010900010b02010b0301090002070b02010b030109000b0401090001090022636f696e5f6d6f64756c655f6964656e7469666965725f706c616365686f6c64657206737472696e67066f626a656374067369676e6572126163636f756e745f636f696e5f73746f726504636f696e0a636f696e5f73746f726522434f494e5f5354525543545f4944454e5449464945525f504c414345484f4c444552085472656173757279064f626a6563740666617563657404696e69740b64756d6d795f6669656c6409436f696e53746f726504436f696e0a616464726573735f6f660a626f72726f775f6d7574087769746864726177076465706f73697408436f696e496e666f06537472696e6704757466380f72656769737465725f657874656e640b6d696e745f657874656e6409746f5f66726f7a656e116372656174655f636f696e5f73746f7265106e65775f6e616d65645f6f626a65637409746f5f736861726564deadeadeadeadeadeadeadeadeadeadeadeadeadeadeadeadeadeadeadeadead0000000000000000000000000000000000000000000000000000000000000001000000000000000000000000000000000000000000000000000000000000000200000000000000000000000000000000000000000000000000000000000000030201de0f20800283b61c0000000000000000000000000000000000000000000000000000000a021615434f494e5f4e414d455f504c414345484f4c4445520a021817434f494e5f53594d424f4c5f504c414345484f4c4445520002010c01010201060b02010b0301080000010400020d0b0011020c020b0138000f004a102700000000000000000000000000000000000000000000000000000000000038010c030b020b03380202010000000c170702110607031106070038030c010d01070138040c000b01380538060c020d020b0038070b0212013808380902010000";
        let modules = vector::singleton(Self::new(module_bytes));

        let new_names = vector::empty<String>();
        vector::push_back(&mut new_names, std::string::utf8(b"fixed_supply_coin"));
        vector::push_back(&mut new_names, std::string::utf8(b"FSC"));
        let old_names = vector::empty<String>();
        vector::push_back(&mut old_names, std::string::utf8(b"coin_module_identifier_placeholder"));
        vector::push_back(&mut old_names, std::string::utf8(b"COIN_STRUCT_IDENTIFIER_PLACEHOLDER"));
        let _modules = Self::replace_module_identiner(modules, old_names, new_names);
    }
}

