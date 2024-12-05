// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

/// This module defines Rooch Gas Coin.
module rooch_framework::gas_coin {
    use std::option;
    use std::string;
    use moveos_std::signer;
    use moveos_std::object::{Self, Object};
    use rooch_framework::coin::{Self, Coin, CoinInfo};
    use rooch_framework::account_coin_store;
    use rooch_framework::onchain_config;
    use rooch_framework::chain_id;

    friend rooch_framework::genesis;
    friend rooch_framework::transaction_validator;
    friend rooch_framework::transaction_fee;

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

    /// Entry point for the faucet, anyone can get Gas via this function on local/dev net, otherwise only admin account can call this function.
    public entry fun faucet_entry(account: &signer, amount: u256) {
        if(!chain_id::is_local_or_dev()){
            onchain_config::ensure_admin(account);
        };
        let addr = signer::address_of(account); 
        faucet(addr, amount);
    }

    /// Can only be called during genesis to initialize the Rooch coin.
    public(friend) fun genesis_init(_genesis_account: &signer){
        let rgas_image = b"<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n<svg id=\"uuid-f3c10da3-9417-410e-a3ab-8ce1ab7d75a2\" data-name=\"layer 1\" xmlns=\"http://www.w3.org/2000/svg\" viewBox=\"0 0 500 500\">\n  <defs>\n    <style>\n      .uuid-f8ed1a91-d770-44e4-b9a1-8898aed79e23 {\n        fill: #006840;\n      }\n\n      .uuid-06e2debf-0aef-494d-b577-6d49611e1cf8 {\n        fill: #b2ff04;\n      }\n    </style>\n  </defs>\n  <circle class=\"uuid-06e2debf-0aef-494d-b577-6d49611e1cf8\" cx=\"250\" cy=\"250\" r=\"250\"/>\n  <path class=\"uuid-f8ed1a91-d770-44e4-b9a1-8898aed79e23\" d=\"M346.39,403.15c-26.17-23.18-51.58-45.69-77.73-68.86-.13,2.58-.28,4.18-.29,5.77-.19,30.24-.36,60.48-.54,90.73q-.04,6.48-6.74,6.45c-8.78-.05-17.57-.27-26.35-.07-3.46,.08-4.36-.83-4.32-4.3,.33-31.62,.46-63.23,.64-94.85,0-1.05,.01-2.09,.02-4.12-26.46,22.99-52.29,45.44-78.06,67.84-1.6-.72-.96-1.95-.96-2.88,.04-14.97,.2-29.94,.17-44.9,0-2.36,.75-3.86,2.52-5.42,28.99-25.57,57.91-51.22,86.85-76.84,.67-.59,1.45-1.07,2.4-1.76-1.89-1.66-3.64-1.09-5.18-1.1-27.12-.2-54.24-.43-81.36-.41-3.91,0-4.88-1.05-4.74-4.83,.34-9.46,.42-18.94,.16-28.4-.11-3.83,1.29-4.39,4.68-4.35,26.96,.28,53.93,.38,80.9,.54,1.68,0,3.37,.02,5.83,.03-2.08-3.59-5.06-5.5-7.53-7.74-27-24.56-54.06-49.05-81.17-73.49-1.49-1.35-2.15-2.64-2.13-4.66,.17-16.03,.23-32.07,.33-49.24,26.24,23.34,51.86,46.12,78.27,69.61,.01-2.46,.02-4.06,.03-5.65,.18-31.01,.43-62.01,.45-93.02,0-3.71,1.1-4.55,4.65-4.44,9.39,.29,18.8,.42,28.19,.14,4.06-.12,4.69,1.28,4.65,4.93-.33,30.7-.44,61.4-.62,92.1,0,1.63-.02,3.27-.04,6.19,26.52-23.13,52.16-45.5,77.98-68.01,1.04,1.6,.64,2.99,.64,4.27-.05,13.9-.29,27.8-.13,41.7,.05,3.86-1.14,6.41-4.05,8.97-28.31,24.95-56.48,50.05-84.69,75.1-.78,.69-1.52,1.41-2.45,2.27,1.66,1.71,3.6,1.03,5.27,1.04,26.96,.21,53.93,.41,80.9,.44,3.42,0,4.77,.6,4.62,4.41-.37,9.61-.37,19.24-.19,28.86,.06,3.38-.77,4.35-4.31,4.31-27.12-.32-54.24-.4-81.36-.56-1.64,0-3.28-.02-6.14-.04,8.77,7.94,16.64,15.09,24.54,22.22,21.63,19.52,43.26,39.04,64.91,58.53,1.08,.97,1.81,1.89,1.8,3.46-.14,16.32-.22,32.64-.32,50.05Z\"/>\n</svg>";
        let coin_info_obj = coin::register_extend<RGas>(
            string::utf8(b"Rooch Gas Coin"),
            string::utf8(b"RGAS"),
            option::some(string::utf8(rgas_image)),
            DECIMALS, // decimals
        );
        object::transfer(coin_info_obj, @rooch_framework);
    }
}
