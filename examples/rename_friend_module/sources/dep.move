// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

module rooch_examples::dep {
    use rooch_examples::rename_friend;
    use moveos_std::account;

    struct Value has key {
      value:u64,
    }

    fun init() {
        let value = rename_friend::value();
        let signer = moveos_std::signer::module_signer<Value>();
        account::move_resource_to(&signer, Value{ value: value });
    }
}
