// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

module coins::private_coin {

    use std::string;
    use moveos_std::object_id;
    use moveos_std::signer;
    
    use moveos_std::object::{Self, Object};
    use rooch_framework::coin::{Self, Coin, CoinInfo};
    use rooch_framework::coin_store::{Self, CoinStore};
    use rooch_framework::account_coin_store;

    const ErrorTransferAmountTooLarge: u64 = 1;

    /// This Coin has no `store` ability, 
    /// so it can not be operate via `account_coin_store::transfer`, `account_coin_store::deposit` and `account_coin_store::withdraw`
    struct PRC has key {}

    struct Treasury has key {
        coin_store: Object<CoinStore<PRC>>
    }

    fun init() {
        let coin_info_obj = coin::register_extend<PRC>(
            
            string::utf8(b"Private Coin"),
            string::utf8(b"PRC"),
            1,
        );
        object::transfer(coin_info_obj, @coins);
        let coin_store = coin_store::create_coin_store_extend<PRC>();
        let treasury_obj = object::new_named_object( Treasury { coin_store });
        object::transfer_extend(treasury_obj, @coins);
    }

    /// Provide a faucet to give out coins to users
    /// In a real world scenario, the coins should be given out in the application business logic.
    public entry fun faucet(account: &signer) {
        let account_addr = signer::address_of(account);
        let coin_signer = signer::module_signer<Treasury>();
        let coin_info_obj = object::borrow_mut_object<CoinInfo<PRC>>(&coin_signer, coin::coin_info_id<PRC>());
        let coin = coin::mint_extend<PRC>(coin_info_obj, 10000);
        account_coin_store::deposit_extend(account_addr, coin);
    }

    /// This function shows how to use `coin::transfer_extend` to define a custom transfer logic
    /// This transfer function limits the amount of transfer to 10000, and take 1% of the amount as fee
    public entry fun transfer(from: &signer, to_addr: address, amount: u256) {
        assert!(amount <= 10000u256, ErrorTransferAmountTooLarge);
        let from_addr = signer::address_of(from);
        let fee_amount = amount / 100u256;
        if (fee_amount > 0u256) {
            let fee = account_coin_store::withdraw_extend<PRC>(from_addr, fee_amount);
            deposit_to_treaury(fee);
        };
        account_coin_store::transfer_extend<PRC>(from_addr, to_addr, amount);
    }

    fun deposit_to_treaury(coin: Coin<PRC>) {
        let treasury_object_id = object_id::named_object_id<Treasury>();
        let treasury_obj = object::borrow_mut_object_extend<Treasury>(treasury_object_id);
        coin_store::deposit_extend(&mut object::borrow_mut(treasury_obj).coin_store, coin);
    }
}
