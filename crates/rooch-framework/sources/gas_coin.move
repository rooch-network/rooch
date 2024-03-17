// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

/// This module defines Rooch Gas Coin.
module rooch_framework::gas_coin {
    use std::string;
    use moveos_std::signer;
    
    use moveos_std::object::{Self, Object};
    use moveos_std::account;
    use rooch_framework::timestamp;
    use rooch_framework::coin::{Self, Coin, CoinInfo};
    use rooch_framework::account_coin_store;

    friend rooch_framework::genesis;
    friend rooch_framework::transaction_validator;

    //TODO should we allow user to transfer gas coin?
    //If not, we can remove `store` ability from GasCoin.
    struct GasCoin has key, store {}

    /// Record the last time when faucet is called for each address.
    struct FaucetRecord has key, store {
        last_time: u64
    }

    /// Faucet interval in seconds
    const FAUCET_INTERVAL: u64 = 24 * 60 * 60; // 1 day

    /// Faucet too frequently
    const ErrorFaucetTooFrequently: u64 = 1; 

    public fun balance(addr: address): u256 {
        account_coin_store::balance<GasCoin>(addr)
    }

    fun borrow_mut_coin_info() : &mut Object<CoinInfo<GasCoin>> {
        let signer = signer::module_signer<GasCoin>();
        let coin_info_id = coin::coin_info_id<GasCoin>();
        object::borrow_mut_object<CoinInfo<GasCoin>>(&signer, coin_info_id)
    }

    fun mint(amount: u256): Coin<GasCoin> {
        let coin_info = borrow_mut_coin_info();
        coin::mint_extend<GasCoin>(coin_info, amount)
    }

    #[test_only]
    public fun mint_for_test(amount: u256) : Coin<GasCoin> {
        mint(amount)
    }

    public fun burn(coin: Coin<GasCoin>) {
        let coin_info = borrow_mut_coin_info(); 
        coin::burn_extend<GasCoin>(coin_info, coin);
    }

    /// deduct gas coin from the given account.
    public(friend) fun deduct_gas(addr: address, amount: u256):Coin<GasCoin> {
        account_coin_store::withdraw_extend<GasCoin>(addr, amount)
    }

    /// Mint gas coin to the given account.
    public(friend) fun faucet(addr: address, amount: u256) {
        let coin = mint(amount);
        account_coin_store::deposit_extend<GasCoin>(addr, coin);
    }

    #[test_only]
    public fun faucet_for_test(addr: address, amount: u256) {
        faucet(addr, amount);
    }

    public entry fun faucet_entry(account: &signer) { 
        //100 RGC
        let amount = 100_000_000_000_000_000_000u256;
        let addr = signer::address_of(account);

        if (account::exists_resource<FaucetRecord>(addr)) {
            let record = account::borrow_mut_resource<FaucetRecord>(addr);
            assert!(timestamp::now_seconds() - record.last_time >= FAUCET_INTERVAL, ErrorFaucetTooFrequently);
            record.last_time = timestamp::now_seconds();
        } else {
            account::move_resource_to(account, FaucetRecord {
                last_time: timestamp::now_seconds()
            });
        };

        faucet(addr, amount);
    }

    /// Can only be called during genesis to initialize the Rooch coin.
    public(friend) fun genesis_init(_genesis_account: &signer){
        let coin_info_obj = coin::register_extend<GasCoin>(
            string::utf8(b"Rooch Gas Coin"),
            string::utf8(b"RGC"),
            18, // decimals
        );
        object::transfer(coin_info_obj, @rooch_framework)
    }
}
