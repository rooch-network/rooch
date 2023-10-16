// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

/// Notes:
/// If the entry functions argument type is changed or the bytecode of the module is changed, and the tests are not updated, the tests will fail.
/// Please update the tests as follows:
/// 1. Compile the example/counter with `./target/debug/rooch move test -p examples/counter -d`
/// 2. Run the following command to get the bytecode of the compiled module: `xxd -c 0 -p examples/counter/build/counter/bytecode_modules/counter.mv`
/// 3. Copy the bytecode of the compiled module from the output of the above command, and update the `module_bytes` variable in the tests below.
module rooch_examples::publish {
    use std::vector;
    use std::signer;
    use moveos_std::move_module::{Self, MoveModule};
    use moveos_std::context::Context;
    use moveos_std::account_storage;

    #[test_only]
    use moveos_std::context;
    #[test_only]
    use std::debug;

    public entry fun publish_modules_entry(ctx: &mut Context,  account: &signer, module_bytes: vector<u8>) {
        account_storage::ensure_account_storage(ctx, signer::address_of(account));
        let m: MoveModule = move_module::new(module_bytes);
        account_storage::publish_modules(ctx, account, vector::singleton(m));
    }

    #[test(account=@0x42)]
    fun test_get_module_name(account: &signer) {
        let addr = signer::address_of(account);
        let ctx = context::new_test_context(addr);
        // The following is the bytes and hex of the compiled module: example/counter/sources/counter.move
        // with account 0x42
        let module_bytes: vector<u8> = x"a11ceb0b060000000b010006020608030e26043406053a32076c7d08e9014006a902220acb02050cd002560da6030200000101010200030c00020400000005000100000600010000070201000008030400010907080108010a09010108010b0a0b0108040605060606010708010002070801060c0106080101030107080001080002070801050107090003070801060c090002060801050106090007636f756e7465720f6163636f756e745f73746f7261676507636f6e7465787407436f756e74657207436f6e7465787408696e63726561736509696e6372656173655f04696e69740576616c756511676c6f62616c5f626f72726f775f6d75740e676c6f62616c5f6d6f76655f746f0d676c6f62616c5f626f72726f77000000000000000000000000000000000000000000000000000000000000004200000000000000000000000000000000000000000000000000000000000000020520000000000000000000000000000000000000000000000000000000000000004200020108030001040001030b0011010201010000050d0b00070038000c010a01100014060100000000000000160b010f0015020200000001060b000b0106000000000000000012003801020301000001060b000700380210001402000000";
        let m: MoveModule = move_module::new(module_bytes);
        let name = move_module::module_name(&m);
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
        let m: MoveModule = move_module::new(module_bytes);
        let modules = vector::singleton(m);
        let (module_names, _module_names_with_init_fn) = move_module::sort_and_verify_modules(&modules, addr);
        debug::print(&module_names);
        context::drop_test_context(ctx);  
    }

    #[test(account=@0x42)]
    fun test_publish_modules(account: &signer) {
        let addr = signer::address_of(account);
        let ctx = context::new_test_context(addr);
        account_storage::create_account_storage(&mut ctx, addr);
        // The following is the bytes and hex of the compiled module: example/counter/sources/counter.move
        // with account 0x42       
        let module_bytes: vector<u8> = x"a11ceb0b060000000b010006020608030e26043406053a32076c7d08e9014006a902220acb02050cd002560da6030200000101010200030c00020400000005000100000600010000070201000008030400010907080108010a09010108010b0a0b0108040605060606010708010002070801060c0106080101030107080001080002070801050107090003070801060c090002060801050106090007636f756e7465720f6163636f756e745f73746f7261676507636f6e7465787407436f756e74657207436f6e7465787408696e63726561736509696e6372656173655f04696e69740576616c756511676c6f62616c5f626f72726f775f6d75740e676c6f62616c5f6d6f76655f746f0d676c6f62616c5f626f72726f77000000000000000000000000000000000000000000000000000000000000004200000000000000000000000000000000000000000000000000000000000000020520000000000000000000000000000000000000000000000000000000000000004200020108030001040001030b0011010201010000050d0b00070038000c010a01100014060100000000000000160b010f0015020200000001060b000b0106000000000000000012003801020301000001060b000700380210001402000000";
        let m: MoveModule = move_module::new(module_bytes);
        account_storage::publish_modules(&mut ctx, account, vector::singleton(m));
        context::drop_test_context(ctx);  
    }
}
