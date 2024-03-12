// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

/// Notes:
/// If the entry functions argument type is changed or the bytecode of the module is changed, and the tests are not updated, the tests will fail.
/// Please update the tests as follows:
/// 1. Compile the example/counter with `./target/debug/rooch move test -p examples/counter -d`
/// 2. Run the following command to get the bytecode of the compiled module: `xxd -c 99999 -p examples/counter/build/counter/bytecode_modules/counter.mv`
/// 3. Copy the bytecode of the compiled module from the output of the above command, and update the `module_bytes` variable in the tests below.
module rooch_examples::publish {
    use std::vector;
    use std::signer;
    use moveos_std::move_module::{Self, MoveModule};
    

    public entry fun publish_modules_entry( account: &signer, module_bytes: vector<u8>) {
        let m: MoveModule = move_module::new(module_bytes);
        let module_store = move_module::borrow_mut_module_store();
        move_module::publish_modules(module_store, account, vector::singleton(m));
    }

    public entry fun publish_counter_example( account: &signer) {
        // The following is the bytes and hex of the compiled module: example/counter/sources/counter.move
        // with account 0x42
        let module_bytes: vector<u8> = x"a11ceb0b060000000b010004020408030c26043206053832076a7308dd0140069d02220abf02050cc402560d9a03020000010100020c00010300000004000100000500010000060201000007030400010807080108010909010108010a0a0b0108040605060606010708010002070801060c0106080101030107080001080002070801050107090003070801060c090002060801050106090007636f756e74657207636f6e7465787407436f756e74657207436f6e7465787408696e63726561736509696e6372656173655f04696e69740576616c756513626f72726f775f6d75745f7265736f75726365106d6f76655f7265736f757263655f746f0f626f72726f775f7265736f75726365000000000000000000000000000000000000000000000000000000000000004200000000000000000000000000000000000000000000000000000000000000020520000000000000000000000000000000000000000000000000000000000000004200020107030001040001030b0011010201010000050d0b00070038000c010a01100014060100000000000000160b010f0015020200000001060b000b0106000000000000000012003801020301000001060b000700380210001402000000";
        let modules = vector::singleton(move_module::new(module_bytes));
        let old_address = @0x42;
        let new_address = signer::address_of(account);
        let remapped_modules = move_module::binding_module_address(modules, old_address, new_address);
        let module_store = move_module::borrow_mut_module_store();
        move_module::publish_modules(module_store, account, remapped_modules);
    }
}
