// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

module moveos_std::genesis {

    use moveos_std::object;
    use moveos_std::account_storage;
    use moveos_std::context::Context;

    const ErrorGenesisInit: u64 = 1;

    fun init(_ctx: &mut Context){
        // init_genesis_account_storage(ctx, @std);
        // init_genesis_account_storage(ctx, @moveos_std);
    }

    fun init_genesis_account_storage(_ctx: &mut Context, account: address) {
        let account_storage = account_storage::create_account_storage(account);
        let object_id = object::address_to_object_id(account);
        let obj = object::new_with_id(object_id, account_storage);
        account_storage::transfer(obj, account);
    }
}
