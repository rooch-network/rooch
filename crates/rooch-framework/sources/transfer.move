// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

module rooch_framework::transfer {
    use moveos_std::object::ObjectID;
    
    use moveos_std::object::{Self};
    use moveos_std::account;
    use rooch_framework::account as account_entry;
    use rooch_framework::account_coin_store;
    use rooch_framework::multichain_address;
    use rooch_framework::address_mapping;

    /// Transfer `amount` of coins `CoinType` from `from` to `to`.
    /// This public entry function requires the `CoinType` to have `key` and `store` abilities.
    public entry fun transfer_coin<CoinType: key + store>(
        
        from: &signer,
        to: address,
        amount: u256,
    ) {
        if(!account::exists_at(to)) {
            account_entry::create_account(to);
        };

        account_coin_store::transfer<CoinType>(from, to, amount)
    }

    /// Transfer `amount` of coins `CoinType` from `from` to a MultiChainAddress.
    /// The MultiChainAddress is represented by `multichain_id` and `raw_address`.
    /// This public entry function requires the `CoinType` to have `key` and `store` abilities.
    public entry fun transfer_coin_to_multichain_address<CoinType: key + store>(
        
        from: &signer,
        multichain_id: u64,
        raw_address: vector<u8>,
        amount: u256,
    ) {
        let maddress = multichain_address::new(multichain_id, raw_address);
        let to = address_mapping::resolve_or_generate(maddress);
        if(!account::exists_at(to)) {
            account_entry::create_account(to);
            address_mapping::bind_no_check(to, maddress);
        };
        account_coin_store::transfer<CoinType>(from, to, amount)
    }

    /// Transfer `from` owned `Object<T>` to `to` account.
    /// TODO: Currently, we can not pass the `Object<T>` argument by value, so, we use `ObjectID` instead.
    /// After the `Object<T>` argument can be passed by value, we should change the argument type to `Object<T>`.
    public entry fun transfer_object<T: key + store>(from: &signer, to: address, object_id: ObjectID) {
        if(!account::exists_at(to)) {
            account_entry::create_account(to);
        };
        let obj = object::take_object<T>(from, object_id);
        object::transfer(obj, to);
    }
}
