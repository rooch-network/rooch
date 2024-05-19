// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

/// This module defines Rooch Gas Coin.
module rooch_framework::gas_coin {
    use std::string;
    use moveos_std::signer;
    use moveos_std::object::{Self, Object};
    use rooch_framework::coin::{Self, Coin, CoinInfo};
    use rooch_framework::account_coin_store;
    use rooch_framework::onchain_config;
    use rooch_framework::chain_id;

    friend rooch_framework::genesis;
    friend rooch_framework::transaction_validator;

    //If not, we can remove `store` ability from GasCoin.
    struct GasCoin has key, store {}

    const DECIMALS: u8 = 8;
    public fun decimals() : u8 {
        DECIMALS
    }

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

    /// Entry point for the faucet, anyone can get Gas via this function on local/dev net, otherwise only sequencer account can call this function.
    public entry fun faucet_entry(account: &signer, amount: u256) {
        if(!chain_id::is_local_or_dev()){
            onchain_config::ensure_sequencer(account);
        };
        let addr = signer::address_of(account); 
        faucet(addr, amount);
    }

    /// Can only be called during genesis to initialize the Rooch coin.
    public(friend) fun genesis_init(_genesis_account: &signer){
        let coin_info_obj = coin::register_extend<GasCoin>(
            string::utf8(b"Rooch Gas Coin"),
            string::utf8(b"RGC"),
            DECIMALS, // decimals
        );
        object::transfer(coin_info_obj, @rooch_framework);
    }
}
