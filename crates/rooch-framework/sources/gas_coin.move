// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

/// This module defines Rooch Gas Coin.
module rooch_framework::gas_coin {
    use std::string;
    use std::signer;
    use moveos_std::context::Context;
    use rooch_framework::coin::{Self, Coin};
    use rooch_framework::account_coin_store;

    friend rooch_framework::genesis;
    friend rooch_framework::transaction_validator;

    //TODO should we allow user to transfer gas coin?
    //If not, we can remove `store` ability from GasCoin.
    struct GasCoin has key, store {}

    public fun balance(ctx: &Context, addr: address): u256 {
        account_coin_store::balance<GasCoin>(ctx, addr)
    }

    fun mint(ctx: &mut Context, amount: u256): Coin<GasCoin> {
        coin::mint_extend<GasCoin>(coin::borrow_mut_coin_info_extend<GasCoin>(ctx), amount)
    }

    #[test_only]
    public fun mint_for_test(ctx: &mut Context, amount: u256) : Coin<GasCoin> {
        mint(ctx, amount)
    }

    public fun burn(ctx: &mut Context, coin: Coin<GasCoin>) {
        coin::burn_extend<GasCoin>(coin::borrow_mut_coin_info_extend<GasCoin>(ctx), coin);
    }

    /// deduct gas coin from the given account.
    public(friend) fun deduct_gas(ctx: &mut Context, addr: address, amount: u256):Coin<GasCoin> {
        account_coin_store::withdraw_extend<GasCoin>(ctx, addr, amount)
    }

    /// Mint gas coin to the given account.
    public(friend) fun faucet(ctx: &mut Context, addr: address, amount: u256) {
        let coin = mint(ctx, amount);
        account_coin_store::deposit_extend<GasCoin>(ctx, addr, coin);
    }

    #[test_only]
    public fun faucet_for_test(ctx: &mut Context, addr: address, amount: u256) {
        faucet(ctx, addr, amount);
    }

    /// TODO find a way to protect this function from DOS attack.
    public entry fun faucet_entry(ctx: &mut Context, account: &signer) { 
        //100 RGC
        let amount = 100_000_000_000_000_000_000u256;
        let addr = signer::address_of(account);
        faucet(ctx, addr, amount);
    }

    /// Can only be called during genesis to initialize the Rooch coin.
    public(friend) fun genesis_init(ctx: &mut Context, _genesis_account: &signer){
        coin::register_extend<GasCoin>(
            ctx,
            string::utf8(b"Rooch Gas Coin"),
            string::utf8(b"RGC"),
            18, // decimals
        );
    }


}
