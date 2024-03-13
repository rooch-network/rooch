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
    

    public entry fun publish_modules_entry(account: &signer, module_bytes: vector<u8>) {
        let m: MoveModule = move_module::new(module_bytes);
        let module_store = move_module::borrow_mut_module_store();
        move_module::publish_modules(module_store, account, vector::singleton(m));
    }

    public entry fun publish_counter_example(account: &signer) {
        // The following is the bytes and hex of the compiled module: example/counter/sources/counter.move
        // with account 0x42
        let module_bytes: vector<u8> = x"a11ceb0b060000000b010006020604030a35043f0605451f0764af010893026006f302220a9503050c9a03770d91040200000101020200030c00000400000000050000000006010000000701000000080000000009000200020a05060108020b07000108010c020800020d050901080604070409040001060c01030107080001080001050107090002060c0900010a0c0106090007636f756e74657209756e69745f74657374076163636f756e7407436f756e74657208696e63726561736509696e6372656173655f04696e69740d696e69745f666f725f7465737410756e69745f746573745f706f69736f6e0576616c756513626f72726f775f6d75745f7265736f75726365106d6f76655f7265736f757263655f746f1a6372656174655f7369676e6572735f666f725f74657374696e670f626f72726f775f7265736f7572636500000000000000000000000000000000000000000000000000000000000000420000000000000000000000000000000000000000000000000000000000000001000000000000000000000000000000000000000000000000000000000000000205200000000000000000000000000000000000000000000000000000000000000042000201090300010400000211010201010000030c070038000c000a00100014060100000000000000160b000f0015020200000000050b0006000000000000000012003801020301000000050b000600000000000000001200380102040000000004060000000000000000110801020501000000050700380210001402000000";
        let modules = vector::singleton(move_module::new(module_bytes));
        let old_address = @0x42;
        let new_address = signer::address_of(account);
        let remapped_modules = move_module::binding_module_address(modules, old_address, new_address);
        let module_store = move_module::borrow_mut_module_store();
        move_module::publish_modules(module_store, account, remapped_modules);
    }
}
