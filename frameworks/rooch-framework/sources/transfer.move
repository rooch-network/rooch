// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

module rooch_framework::transfer {
    
    use std::option;
    use std::string::String;
    use moveos_std::type_info;
    use moveos_std::object::Object;
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
        address_mapping::bind_bitcoin_address_internal(rooch_address, btc_address);
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
    public entry fun transfer_object<T: key + store>(to: address, obj: Object<T>) {
        object::transfer(obj, to);
    }

    /// Transfer `from` owned `Object<T>` to a Bitcoin Address.
    public entry fun transfer_object_to_bitcoin_address<T: key + store>(
        to: String, 
        obj: Object<T>) {
        let btc_address = bitcoin_address::from_string(&to);
        let rooch_address = bitcoin_address::to_rooch_address(&btc_address);
        address_mapping::bind_bitcoin_address_internal(rooch_address, btc_address);
        object::transfer(obj, rooch_address);
    }


    /// Forward generic transfers to non-generic system if it's available
    public entry fun transfer_coin_v2<CoinType: key + store>(
        from: &signer,
        to: address,
        amount: u256,
    ) {
        if (is_v2_enabled()) {
            // Use V2 transfer
            let coin_type = type_info::type_name<CoinType>();
            transfer_coin_by_type_name(from, to, coin_type, amount);
        } else {
            // Use original generic transfer
            transfer_coin<CoinType>(from, to, amount);
        }
    }

    /// Check if V2 system is enabled
    fun is_v2_enabled(): bool {
        // Logic to check if V2 system is enabled
        // This could be a global flag or based on a feature flag
        true // For simplicity
    }

    /// Direct transfer by coin type name
    public entry fun transfer_coin_by_type_name(
        from: &signer,
        to: address,
        coin_type: String,
        amount: u256,
    ) {
        // // Get coin metadata and info ID
        // let registry = coin::borrow_registry();
        // assert!(object::contains_field(registry, coin_type), ErrorCoinTypeNotRegistered);
        // let metadata: &CoinMetadata = object::borrow_field(registry, coin_type);
        // let coin_info_id = metadata.coin_info_id;
        //
        // // Get or create coin stores
        // let from_addr = signer::address_of(from);
        // let from_store_id = account_coin_store_v2::get_or_create_coin_store(from_addr, coin_info_id, coin_type);
        // let to_store_id = account_coin_store_v2::get_or_create_coin_store(to, coin_info_id, coin_type);
        //
        // // Perform direct transfer
        // coin_store_v2::direct_transfer(from_store_id, to_store_id, amount);

        account_coin_store::transfer_v2(from, to, coin_type, amount);
    }
}
