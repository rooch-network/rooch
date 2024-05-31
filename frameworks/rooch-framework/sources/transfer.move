// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

module rooch_framework::transfer {
    
    use std::option;
    use std::string::String;
    use moveos_std::object::ObjectID;
    use moveos_std::object;
    use rooch_framework::account_coin_store;
    use rooch_framework::multichain_address;
    use rooch_framework::address_mapping;
    use rooch_framework::bitcoin_address;

    const ErrorAddressMappingNotExists: u64 = 1;

    /// Transfer `amount` of coins `CoinType` from `from` to `to`.
    /// This public entry function requires the `CoinType` to have `key` and `store` abilities.
    public entry fun transfer_coin<CoinType: key + store>(
        from: &signer,
        to: address,
        amount: u256,
    ) {
        account_coin_store::transfer<CoinType>(from, to, amount)
    }

    /// Transfer `amount` of coins `CoinType` from `from` to a Bitcoin Address.
    public entry fun transfer_coin_to_bitcoin_address<CoinType: key + store>(
        from: &signer,
        to: String,
        amount: u256,
    ) {
        let btc_address = bitcoin_address::from_string(&to);
        let rooch_address = bitcoin_address::to_rooch_address(&btc_address);
        account_coin_store::transfer<CoinType>(from, rooch_address, amount)
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
        let to_opt = address_mapping::resolve(maddress);
        assert!(option::is_some(&to_opt), ErrorAddressMappingNotExists);
        let to = option::destroy_some(to_opt);
        account_coin_store::transfer<CoinType>(from, to, amount)
    }

    /// Transfer `from` owned `Object<T>` to `to` account.
    /// TODO: Currently, we can not pass the `Object<T>` argument by value, so, we use `ObjectID` instead.
    /// After the `Object<T>` argument can be passed by value, we should change the argument type to `Object<T>`.
    public entry fun transfer_object<T: key + store>(from: &signer, to: address, object_id: ObjectID) {
        let obj = object::take_object<T>(from, object_id);
        object::transfer(obj, to);
    }

    /// Transfer `from` owned `Object<T>` to a Bitcoin Address.
    public entry fun transfer_object_to_bitcoin_address<T: key + store>(
        from: &signer, 
        to: String, 
        object_id: ObjectID) {
        let btc_address = bitcoin_address::from_string(&to);
        let rooch_address = bitcoin_address::to_rooch_address(&btc_address);
        let obj = object::take_object<T>(from, object_id);
        object::transfer(obj, rooch_address);
    }
}
