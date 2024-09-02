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

    /// RGas is the symbol of Rooch Gas Coin
    //If not, we can remove `store` ability from RGas.
    struct RGas has key, store {}

    const DECIMALS: u8 = 8;
    public fun decimals() : u8 {
        DECIMALS
    }

    public fun balance(addr: address): u256 {
        account_coin_store::balance<RGas>(addr)
    }

    fun borrow_mut_coin_info() : &mut Object<CoinInfo<RGas>> {
        let signer = signer::module_signer<RGas>();
        let coin_info_id = coin::coin_info_id<RGas>();
        object::borrow_mut_object<CoinInfo<RGas>>(&signer, coin_info_id)
    }

    fun mint(amount: u256): Coin<RGas> {
        let coin_info = borrow_mut_coin_info();
        coin::mint_extend<RGas>(coin_info, amount)
    }

    #[test_only]
    public fun mint_for_test(amount: u256) : Coin<RGas> {
        mint(amount)
    }

    public fun burn(coin: Coin<RGas>) {
        let coin_info = borrow_mut_coin_info(); 
        coin::burn_extend<RGas>(coin_info, coin);
    }

    /// deduct gas coin from the given account.
    public(friend) fun deduct_gas(addr: address, amount: u256):Coin<RGas> {
        account_coin_store::withdraw_extend<RGas>(addr, amount)
    }

    /// Mint gas coin to the given account.
    public(friend) fun faucet(addr: address, amount: u256) {
        let coin = mint(amount);
        account_coin_store::deposit_extend<RGas>(addr, coin);
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
        let coin_info_obj = coin::register_extend<RGas>(
            string::utf8(b"Rooch Gas Coin"),
            string::utf8(b"RGAS"),
            DECIMALS, // decimals
        );
        object::transfer(coin_info_obj, @rooch_framework);
    }
}
