// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

/// `move_module` provides some basic functions for handle Move module in Move.
module moveos_std::move_module {
    use std::vector;
    use std::string::{Self, String};

    friend moveos_std::account_storage;
    
    /// Module address is not the same as the signer
    const ErrorAddressNotMatchWithSigner: u64 = 1;
    /// Module verification error
    const ErrorModuleVerificationError: u64 = 2;
    /// Module incompatible with the old ones.
    const ErrorModuleIncompatible: u64 = 3;
    /// Vector length not match
    const ErrorLengthNotMatch: u64 = 4;
    
    struct MoveModule has store, drop {
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

    public fun module_name(move_module: &MoveModule): String {
        module_name_inner(&move_module.byte_codes)
    }

    /// Sort modules by dependency order and then verify. 
    /// Return their names and names of the modules with init function if sorted dependency order.
    /// This function will ensure the module's bytecode is valid and the module id is matching the account address.
    /// Return
    ///     1. Module names of all the modules. Order of names is not matching the input, but sorted by module dependency order
    ///     2. Module names of the modules with init function.
    public fun sort_and_verify_modules(modules: &vector<MoveModule>, account_address: address): (vector<String>, vector<String>) {
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

    /// Binding given module's address to the new address
    public fun binding_module_address(
        modules: vector<MoveModule>,
        old_address: address,
        new_address: address,
    ): vector<MoveModule> {
        let bytes_vec = into_byte_codes_batch(modules);

        let old_addresses = vector::singleton(old_address);
        let new_addresses = vector::singleton(new_address);
        
        let rebinded_bytes = replace_address_identifiers(bytes_vec, old_addresses, new_addresses);
        let rebinded_bytes = replace_addresses_constant(rebinded_bytes, old_addresses, new_addresses);
        
        new_batch(rebinded_bytes)
    }

    /// Replace given module's identifier to the new ones
    public fun replace_module_identiner (
        modules: vector<MoveModule>,
        old_names: vector<String>,
        new_names: vector<String>,
    ): vector<MoveModule> {
        assert!(
            vector::length(&old_names) == vector::length(&new_names),
            ErrorLengthNotMatch
        );
        let bytes_vec = into_byte_codes_batch(modules);        
        let rebinded_bytes = replace_identifiers(bytes_vec, old_names, new_names);        
        new_batch(rebinded_bytes)
    }

    /// Replace given struct's identifier to the new ones
    public fun replace_struct_identifier(
        modules: vector<MoveModule>,
        old_names: vector<String>,
        new_names: vector<String>,
    ): vector<MoveModule> {
        replace_module_identiner(modules, old_names, new_names)
    }

    /// Replace given string constant to the new ones
    public fun replace_constant_string(
        modules: vector<MoveModule>,
        old_strings: vector<String>,
        new_strings: vector<String>,
    ): vector<MoveModule> {
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

    /// Replace given address constant to the new ones
    public fun replace_constant_address(
        modules: vector<MoveModule>,
        old_addresses: vector<address>,
        new_addresses: vector<address>,
    ): vector<MoveModule> {
        assert!(
            vector::length(&old_addresses) == vector::length(&new_addresses),
            ErrorLengthNotMatch
        );
        let bytes_vec = into_byte_codes_batch(modules);
        let rebinded_bytes = replace_addresses_constant(bytes_vec, old_addresses, new_addresses);
        new_batch(rebinded_bytes)
    }

    /// Replace given u8 constant to the new ones
    public fun replace_constant_u8(
        modules: vector<MoveModule>,
        old_u8s: vector<u8>,
        new_u8s: vector<u8>,
    ): vector<MoveModule> {
        assert!(
            vector::length(&old_u8s) == vector::length(&new_u8s),
            ErrorLengthNotMatch
        );
        let bytes_vec = into_byte_codes_batch(modules);
        let rebinded_bytes = replace_u8_constant(bytes_vec, old_u8s, new_u8s);
        new_batch(rebinded_bytes)
    }

    /// Replace given u64 constant to the new ones
    public fun replace_constant_u64(
        modules: vector<MoveModule>,
        old_u64s: vector<u64>,
        new_u64s: vector<u64>,
    ): vector<MoveModule> {
        assert!(
            vector::length(&old_u64s) == vector::length(&new_u64s),
            ErrorLengthNotMatch
        );
        let bytes_vec = into_byte_codes_batch(modules);
        let rebinded_bytes = replace_u64_constant(bytes_vec, old_u64s, new_u64s);
        new_batch(rebinded_bytes)
    }

    /// Replace given u256 constant to the new ones
    public fun replace_constant_u256(
        modules: vector<MoveModule>,
        old_u256s: vector<u256>,
        new_u256s: vector<u256>,
    ): vector<MoveModule> {
        assert!(
            vector::length(&old_u256s) == vector::length(&new_u256s),
            ErrorLengthNotMatch
        );
        let bytes_vec = into_byte_codes_batch(modules);
        let rebinded_bytes = replace_u256_constant(bytes_vec, old_u256s, new_u256s);
        new_batch(rebinded_bytes)
    }

    native fun module_name_inner(byte_codes: &vector<u8>): String;

    /// Sort modules by dependency order and then verify. 
    /// Return their names and names of the modules with init function if sorted dependency order.
    native fun sort_and_verify_modules_inner(modules: vector<vector<u8>>, account_address: address): (vector<String>, vector<String>);
    
    /// Request to call the init functions of the given modules
    /// module_names: names of modules which have a init function
    /// account_address: address of all the modules
    native public(friend) fun request_init_functions(module_names: vector<String>, account_address: address);

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
    use std::debug;
    #[test_only]
    use std::signer;

    #[test]
    fun test_get_module_name() {
        // The following is the bytes and hex of the compiled module: example/counter/sources/counter.move
        // with account 0x42
        let module_bytes: vector<u8> = x"a11ceb0b060000000b010004020408030c26043206053832076a7308dd0140069d02220abf02050cc402560d9a03020000010100020c00010300000004000100000500010000060201000007030400010807080108010909010108010a0a0b0108040605060606010708010002070801060c0106080101030107080001080002070801050107090003070801060c090002060801050106090007636f756e74657207636f6e7465787407436f756e74657207436f6e7465787408696e63726561736509696e6372656173655f04696e69740576616c756513626f72726f775f6d75745f7265736f75726365106d6f76655f7265736f757263655f746f0f626f72726f775f7265736f75726365000000000000000000000000000000000000000000000000000000000000004200000000000000000000000000000000000000000000000000000000000000020520000000000000000000000000000000000000000000000000000000000000004200020107030001040001030b0011010201010000050d0b00070038000c010a01100014060100000000000000160b010f0015020200000001060b000b0106000000000000000012003801020301000001060b000700380210001402000000";
        let m: MoveModule = Self::new(module_bytes);
        let name = Self::module_name(&m);
        debug::print(&name);
    }

    #[test(account=@0x42)]
    fun test_verify_modules(account: &signer) {
        let addr = signer::address_of(account);
        // The following is the bytes and hex of the compiled module: example/counter/sources/counter.move
        // with account 0x42
        let module_bytes: vector<u8> = x"a11ceb0b060000000b010004020408030c26043206053832076a7308dd0140069d02220abf02050cc402560d9a03020000010100020c00010300000004000100000500010000060201000007030400010807080108010909010108010a0a0b0108040605060606010708010002070801060c0106080101030107080001080002070801050107090003070801060c090002060801050106090007636f756e74657207636f6e7465787407436f756e74657207436f6e7465787408696e63726561736509696e6372656173655f04696e69740576616c756513626f72726f775f6d75745f7265736f75726365106d6f76655f7265736f757263655f746f0f626f72726f775f7265736f75726365000000000000000000000000000000000000000000000000000000000000004200000000000000000000000000000000000000000000000000000000000000020520000000000000000000000000000000000000000000000000000000000000004200020107030001040001030b0011010201010000050d0b00070038000c010a01100014060100000000000000160b010f0015020200000001060b000b0106000000000000000012003801020301000001060b000700380210001402000000";
        let m: MoveModule = Self::new(module_bytes);
        let modules = vector::singleton(m);
        let (module_names, _module_names_with_init_fn) = Self::sort_and_verify_modules(&modules, addr);
        debug::print(&module_names);
    }

    #[test(account=@0x1314)]
    #[expected_failure(abort_code = 0x10001, location = Self)]
    fun test_address_mismatch_failure(account: &signer) {
        let addr = signer::address_of(account);
        // The following is the bytes and hex of the compiled module: example/counter/sources/counter.move
        // with account 0x42
        let module_bytes: vector<u8> = x"a11ceb0b060000000b010004020408030c26043206053832076a7308dd0140069d02220abf02050cc402560d9a03020000010100020c00010300000004000100000500010000060201000007030400010807080108010909010108010a0a0b0108040605060606010708010002070801060c0106080101030107080001080002070801050107090003070801060c090002060801050106090007636f756e74657207636f6e7465787407436f756e74657207436f6e7465787408696e63726561736509696e6372656173655f04696e69740576616c756513626f72726f775f6d75745f7265736f75726365106d6f76655f7265736f757263655f746f0f626f72726f775f7265736f75726365000000000000000000000000000000000000000000000000000000000000004200000000000000000000000000000000000000000000000000000000000000020520000000000000000000000000000000000000000000000000000000000000004200020107030001040001030b0011010201010000050d0b00070038000c010a01100014060100000000000000160b010f0015020200000001060b000b0106000000000000000012003801020301000001060b000700380210001402000000";
        let m: MoveModule = Self::new(module_bytes);
        let modules = vector::singleton(m);
        let (module_names, _module_names_with_init_fn) = Self::sort_and_verify_modules(&modules, addr);
        debug::print(&module_names);
    }
    
    #[test(account=@0x42)]
    fun test_module_template(account: &signer) {
        let addr = signer::address_of(account);
        // The following is the bytes of module `examples/coins/sources/fixed_supply_coin.move` with account 0x42
        let ref_bytes: vector<u8> = x"a11ceb0b060000000b01001002102803385004880114059c01ac0107c802a40208ec04800106ec05410aad06110cbe06790db707020000010102020203020403050306030700080c0000090800020a0000030b0c010001070f0801080106100001080106150c01080101160700000c000100000d020100041104050003120708010807130a0b010c05140c01010c01170e0f000618101101080619120b0108031a14010108071b0215010c07141601010c021c17140108031d140101080306040905090709080909130a090b090c060d0603070802060c070b03010801000107080202050b0501080001060c010501080101070b030109000107090001080002070b03010b040109000f010b0501090003070802050b05010900030b050108000b03010b060108000b03010b04010800010a02010807040708020807080702010b03010b0601090002070b03010b060109000f010b06010800010b03010900010b03010b0401090002070b03010b040109000b050109000207080209001166697865645f737570706c795f636f696e06737472696e6707636f6e74657874066f626a656374067369676e6572126163636f756e745f636f696e5f73746f726504636f696e0a636f696e5f73746f72650346534308547265617375727907436f6e74657874064f626a6563740666617563657404696e69740b64756d6d795f6669656c6409436f696e53746f726504436f696e0a616464726573735f6f660a626f72726f775f6d7574087769746864726177076465706f73697408436f696e496e666f06537472696e6704757466380f72656769737465725f657874656e640b6d696e745f657874656e6409746f5f66726f7a656e116372656174655f636f696e5f73746f7265106e65775f6e616d65645f6f626a65637409746f5f73686172656400000000000000000000000000000000000000000000000000000000000000420000000000000000000000000000000000000000000000000000000000000001000000000000000000000000000000000000000000000000000000000000000200000000000000000000000000000000000000000000000000000000000000030201010f2000b4f9e4300000000000000000000000000000000000000000000000000000000a021211466978656420537570706c7920436f696e0a0204034653430002010e01010201070b03010b0401080000010400030e0b0111020c030b0238000f004a102700000000000000000000000000000000000000000000000000000000000038010c040b000b030b04380202010000000d1a0a000702110607031106070038030c020d02070138040c010b0238050a0038060c030d030b0138070b000b0312013808380902010000";

        // The following is the bytes of compiled module: examples/module_template/template/sources/fixed_supply_coin_template.move
        //rooch move build -p examples/module_template/template
        //xxd -c 99999 -p examples/module_template/template/build/template/bytecode_modules/coin_module_identifier_placeholder.mv
        let module_bytes: vector<u8> = x"a11ceb0b060000000b01001002102803385004880114059c01ac0107c802d402089c058001069c06590af506110c8607790dff07020000010102020203020403050306030700080c0000090800020a0000030b0c010001070f0801080106100001080106150c01080101160700000c000100000d020100041104050003120708010807130a0b010c05140c01010c01170e0f000618101101080619120b0108031a14010108071b0215010c07141601010c021c17140108031d140101080306040905090709080909130a090b090c060d0603070802060c070b03010801000107080202050b0501080001060c010501080101070b030109000107090001080002070b03010b040109000f010b0501090003070802050b05010900030b050108000b03010b060108000b03010b04010800010a02010807040708020807080702010b03010b0601090002070b03010b060109000f010b06010800010b03010900010b03010b0401090002070b03010b040109000b0501090002070802090022636f696e5f6d6f64756c655f6964656e7469666965725f706c616365686f6c64657206737472696e6707636f6e74657874066f626a656374067369676e6572126163636f756e745f636f696e5f73746f726504636f696e0a636f696e5f73746f726522434f494e5f5354525543545f4944454e5449464945525f504c414345484f4c44455208547265617375727907436f6e74657874064f626a6563740666617563657404696e69740b64756d6d795f6669656c6409436f696e53746f726504436f696e0a616464726573735f6f660a626f72726f775f6d7574087769746864726177076465706f73697408436f696e496e666f06537472696e6704757466380f72656769737465725f657874656e640b6d696e745f657874656e6409746f5f66726f7a656e116372656174655f636f696e5f73746f7265106e65775f6e616d65645f6f626a65637409746f5f736861726564deadeadeadeadeadeadeadeadeadeadeadeadeadeadeadeadeadeadeadeadead0000000000000000000000000000000000000000000000000000000000000001000000000000000000000000000000000000000000000000000000000000000200000000000000000000000000000000000000000000000000000000000000030201de0f20800283b61c0000000000000000000000000000000000000000000000000000000a021615434f494e5f4e414d455f504c414345484f4c4445520a021817434f494e5f53594d424f4c5f504c414345484f4c4445520002010e01010201070b03010b0401080000010400030e0b0111020c030b0238000f004a102700000000000000000000000000000000000000000000000000000000000038010c040b000b030b04380202010000000d1a0a000702110607031106070038030c020d02070138040c010b0238050a0038060c030d030b0138070b000b0312013808380902010000";
        
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
}