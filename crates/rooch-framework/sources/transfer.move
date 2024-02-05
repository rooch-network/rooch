// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

module rooch_framework::transfer {
    use moveos_std::object_id::ObjectID;
    use moveos_std::context::{Self, Context};
    use moveos_std::object::{Self};
    use rooch_framework::account;
    use rooch_framework::account_coin_store;
    use rooch_framework::multichain_address;
    use rooch_framework::address_mapping;

    /// Transfer `amount` of coins `CoinType` from `from` to `to`.
    /// This public entry function requires the `CoinType` to have `key` and `store` abilities.
    public entry fun transfer_coin<CoinType: key + store>(
        ctx: &mut Context,
        from: &signer,
        to: address,
        amount: u256,
    ) {
        if(!account::exists_at(ctx, to)) {
            account::create_account(ctx, to);
        };

        account_coin_store::transfer<CoinType>(ctx, from, to, amount)
    }

    /// Transfer `amount` of coins `CoinType` from `from` to a MultiChainAddress.
    /// The MultiChainAddress is represented by `multichain_id` and `raw_address`.
    /// This public entry function requires the `CoinType` to have `key` and `store` abilities.
    public entry fun transfer_coin_to_multichain_address<CoinType: key + store>(
        ctx: &mut Context,
        from: &signer,
        multichain_id: u64,
        raw_address: vector<u8>,
        amount: u256,
    ) {
        let maddress = multichain_address::new(multichain_id, raw_address);
        let to = address_mapping::resolve_or_generate(ctx, maddress);
        if(!account::exists_at(ctx, to)) {
            account::create_account(ctx, to);
            address_mapping::bind_no_check(ctx, to, maddress);
        };
        account_coin_store::transfer<CoinType>(ctx, from, to, amount)
    }

    /// Transfer `from` owned `Object<T>` to `to` account.
    /// TODO: Currently, we can not pass the `Object<T>` argument by value, so, we use `ObjectID` instead.
    /// After the `Object<T>` argument can be passed by value, we should change the argument type to `Object<T>`.
    public entry fun transfer_object<T: key + store>(ctx: &mut Context, from: &signer, to: address, object_id: ObjectID) {
        if(!account::exists_at(ctx, to)) {
            account::create_account(ctx, to);
        };
        let obj = object::take_object<T>(from, object_id);
        object::transfer(obj, to);
    }
}
