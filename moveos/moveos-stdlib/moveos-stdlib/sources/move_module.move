// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

/// `move_module` provides some basic functions for handle Move module in Move.
module moveos_std::move_module {
    use std::vector;
    use std::error;
    use std::string::String;

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
        MoveModule {
            byte_codes,
        }
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

    /// Remap addresses in module binary where the length of
    /// `old_addresses` must equal to that of `new_addresses`.
    public fun remap_module_addresses(
        modules: vector<MoveModule>,
        old_addresses: vector<address>,
        new_addresses: vector<address>,
    ): vector<MoveModule> {
        assert!(
            vector::length(&old_addresses) == vector::length(&new_addresses), 
            error::invalid_argument(ErrorLengthNotMatch)
        );
        let bytes_vec = vector::empty<vector<u8>>();
        let i = 0u64;
        let len = vector::length(&modules);
        while (i < len) {
            vector::push_back(&mut bytes_vec, vector::pop_back(&mut modules).byte_codes);
            i = i + 1;
        };
        let remapped_bytes = remap_module_addresses_inner(bytes_vec, old_addresses, new_addresses);
        // let remapped_bytes = remap_module_addresses_inner(bytes_vec);
        let remapped_modules = vector::empty<MoveModule>();
        i = 0u64;
        let len = vector::length(&remapped_bytes);
        while (i < len) {
            vector::push_back(&mut remapped_modules, MoveModule {
                byte_codes: vector::pop_back(&mut remapped_bytes),
            });
            i = i + 1;
        };
        vector::destroy_empty(remapped_bytes);
        remapped_modules
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

    /// Native function to remap addresses in module binary where the length of
    /// `old_addresses` must equal to that of `new_addresses`.
    native fun remap_module_addresses_inner(
        bytes: vector<vector<u8>>,
        old_addresses: vector<address>,
        new_addresses: vector<address>,
    ): vector<vector<u8>>;


    #[test_only]
    use std::debug;
    #[test_only]
    use std::signer;
    #[test_only]
    use moveos_std::context;

    #[test(account=@0x42)]
    fun test_get_module_name(account: &signer) {
        let addr = signer::address_of(account);
        let ctx = context::new_test_context(addr);
        // The following is the bytes and hex of the compiled module: example/counter/sources/counter.move
        // with account 0x42
        let module_bytes: vector<u8> = x"a11ceb0b060000000b010006020608030e26043406053a32076c7d08e9014006a902220acb02050cd002560da6030200000101010200030c00020400000005000100000600010000070201000008030400010907080108010a09010108010b0a0b0108040605060606010708010002070801060c0106080101030107080001080002070801050107090003070801060c090002060801050106090007636f756e7465720f6163636f756e745f73746f7261676507636f6e7465787407436f756e74657207436f6e7465787408696e63726561736509696e6372656173655f04696e69740576616c756511676c6f62616c5f626f72726f775f6d75740e676c6f62616c5f6d6f76655f746f0d676c6f62616c5f626f72726f77000000000000000000000000000000000000000000000000000000000000004200000000000000000000000000000000000000000000000000000000000000020520000000000000000000000000000000000000000000000000000000000000004200020108030001040001030b0011010201010000050d0b00070038000c010a01100014060100000000000000160b010f0015020200000001060b000b0106000000000000000012003801020301000001060b000700380210001402000000";
        let m: MoveModule = Self::new(module_bytes);
        let name = Self::module_name(&m);
        debug::print(&name);
        context::drop_test_context(ctx);
    }

    #[test(account=@0x42)]
    fun test_verify_modules(account: &signer) {
        let addr = signer::address_of(account);
        let ctx = context::new_test_context(addr);
        // The following is the bytes and hex of the compiled module: example/counter/sources/counter.move
        // with account 0x42
        let module_bytes: vector<u8> = x"a11ceb0b060000000b010006020608030e26043406053a32076c7d08e9014006a902220acb02050cd002560da6030200000101010200030c00020400000005000100000600010000070201000008030400010907080108010a09010108010b0a0b0108040605060606010708010002070801060c0106080101030107080001080002070801050107090003070801060c090002060801050106090007636f756e7465720f6163636f756e745f73746f7261676507636f6e7465787407436f756e74657207436f6e7465787408696e63726561736509696e6372656173655f04696e69740576616c756511676c6f62616c5f626f72726f775f6d75740e676c6f62616c5f6d6f76655f746f0d676c6f62616c5f626f72726f77000000000000000000000000000000000000000000000000000000000000004200000000000000000000000000000000000000000000000000000000000000020520000000000000000000000000000000000000000000000000000000000000004200020108030001040001030b0011010201010000050d0b00070038000c010a01100014060100000000000000160b010f0015020200000001060b000b0106000000000000000012003801020301000001060b000700380210001402000000";
        let m: MoveModule = Self::new(module_bytes);
        let modules = vector::singleton(m);
        let (module_names, _module_names_with_init_fn) = Self::sort_and_verify_modules(&modules, addr);
        debug::print(&module_names);
        context::drop_test_context(ctx);  
    }

    #[test(account=@0x1314)]
    #[expected_failure(abort_code = 0x10001, location = Self)]
    fun test_address_mismatch_failure(account: &signer) {
        let addr = signer::address_of(account);
        let ctx = context::new_test_context(addr);
        // The following is the bytes and hex of the compiled module: example/counter/sources/counter.move
        // with account 0x42
        let module_bytes: vector<u8> = x"a11ceb0b060000000b010006020608030e26043406053a32076c7d08e9014006a902220acb02050cd002560da6030200000101010200030c00020400000005000100000600010000070201000008030400010907080108010a09010108010b0a0b0108040605060606010708010002070801060c0106080101030107080001080002070801050107090003070801060c090002060801050106090007636f756e7465720f6163636f756e745f73746f7261676507636f6e7465787407436f756e74657207436f6e7465787408696e63726561736509696e6372656173655f04696e69740576616c756511676c6f62616c5f626f72726f775f6d75740e676c6f62616c5f6d6f76655f746f0d676c6f62616c5f626f72726f77000000000000000000000000000000000000000000000000000000000000004200000000000000000000000000000000000000000000000000000000000000020520000000000000000000000000000000000000000000000000000000000000004200020108030001040001030b0011010201010000050d0b00070038000c010a01100014060100000000000000160b010f0015020200000001060b000b0106000000000000000012003801020301000001060b000700380210001402000000";
        let m: MoveModule = Self::new(module_bytes);
        let modules = vector::singleton(m);
        let (module_names, _module_names_with_init_fn) = Self::sort_and_verify_modules(&modules, addr);
        debug::print(&module_names);
        context::drop_test_context(ctx);  
    }

    #[test(account=@0x1314)]
    fun test_remap_address(account: &signer) {
        let addr = signer::address_of(account);
        let ctx = context::new_test_context(addr);
        // The following is the bytes and hex of the compiled module: example/counter/sources/counter.move
        // with account 0x42
        let module_bytes: vector<u8> = x"a11ceb0b060000000b010006020608030e26043406053a32076c7d08e9014006a902220acb02050cd002560da6030200000101010200030c00020400000005000100000600010000070201000008030400010907080108010a09010108010b0a0b0108040605060606010708010002070801060c0106080101030107080001080002070801050107090003070801060c090002060801050106090007636f756e7465720f6163636f756e745f73746f7261676507636f6e7465787407436f756e74657207436f6e7465787408696e63726561736509696e6372656173655f04696e69740576616c756511676c6f62616c5f626f72726f775f6d75740e676c6f62616c5f6d6f76655f746f0d676c6f62616c5f626f72726f77000000000000000000000000000000000000000000000000000000000000004200000000000000000000000000000000000000000000000000000000000000020520000000000000000000000000000000000000000000000000000000000000004200020108030001040001030b0011010201010000050d0b00070038000c010a01100014060100000000000000160b010f0015020200000001060b000b0106000000000000000012003801020301000001060b000700380210001402000000";
        let m: MoveModule = Self::new(module_bytes);
        let modules = vector::singleton(m);
        let new_addresses = vector::singleton(addr);
        let old_addresses = vector::singleton(@0x42);
        let remapped_modules = Self::remap_module_addresses(modules, old_addresses, new_addresses);
        // In `sort_and_verify_modules`, addresses of modules are ensured to be the same with signer address
        // So if the remapping is failed, the verification will fail
        let (module_names, _module_names_with_init_fn) = Self::sort_and_verify_modules(&remapped_modules, addr);
        debug::print(&module_names);
        context::drop_test_context(ctx);  
    }

}