// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

/// This module defines Rooch Gas Coin.
module rooch_framework::gas_coin {
    use std::string;
    use moveos_std::signer;
    use moveos_std::context::{Context};
    use moveos_std::object::{Self, Object};
    use rooch_framework::coin::{Self, Coin, CoinInfo};
    use rooch_framework::account_coin_store;

    friend rooch_framework::genesis;
    friend rooch_framework::transaction_validator;

    //TODO should we allow user to transfer gas coin?
    //If not, we can remove `store` ability from GasCoin.
    struct GasCoin has key, store {}

    public fun balance(ctx: &Context, addr: address): u256 {
        account_coin_store::balance<GasCoin>(ctx, addr)
    }

    fun borrow_mut_coin_info(_ctx: &mut Context) : &mut Object<CoinInfo<GasCoin>> {
        let signer = signer::module_signer<GasCoin>();
        let coin_info_id = coin::coin_info_id<GasCoin>();
        object::borrow_mut_object<CoinInfo<GasCoin>>(&signer, coin_info_id)
    }

    fun mint(ctx: &mut Context, amount: u256): Coin<GasCoin> {
        let coin_info = borrow_mut_coin_info(ctx);
        coin::mint_extend<GasCoin>(coin_info, amount)
    }

    #[test_only]
    public fun mint_for_test(ctx: &mut Context, amount: u256) : Coin<GasCoin> {
        mint(ctx, amount)
    }

    public fun burn(ctx: &mut Context, coin: Coin<GasCoin>) {
        let coin_info = borrow_mut_coin_info(ctx); 
        coin::burn_extend<GasCoin>(coin_info, coin);
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
        let coin_info_obj = coin::register_extend<GasCoin>(
            ctx,
            string::utf8(b"Rooch Gas Coin"),
            string::utf8(b"RGC"),
            18, // decimals
        );
        object::transfer(coin_info_obj, @rooch_framework)
    }


}
