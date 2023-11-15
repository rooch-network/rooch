// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

module template::coin_module_identifier_placeholder {

    use std::string;
    use moveos_std::signer;
    use moveos_std::context::{Self, Context};
    use moveos_std::object::{Self, Object};
    use rooch_framework::coin;
    use rooch_framework::coin_store::{Self, CoinStore};
    use rooch_framework::account_coin_store;
    

    struct COIN_STRUCT_IDENTIFIER_PLACEHOLDER has key, store {}

    struct Treasury has key {
        coin_store: Object<CoinStore<COIN_STRUCT_IDENTIFIER_PLACEHOLDER>>
    }
    
    const TOTAL_SUPPLY: u256 = 123_321_123_456u256;
    const DECIMALS: u8 = 222u8; 


    fun init(ctx: &mut Context) {
        let coin_info_obj = coin::register_extend<COIN_STRUCT_IDENTIFIER_PLACEHOLDER>(
            ctx,
            string::utf8(b"COIN_NAME_PLACEHOLDER"),
            string::utf8(b"COIN_SYMBOL_PLACEHOLDER"),
            DECIMALS,
        );
        // Mint the total supply of coins, and store it to the treasury
        let coin = coin::mint_extend<COIN_STRUCT_IDENTIFIER_PLACEHOLDER>(&mut coin_info_obj, TOTAL_SUPPLY);
        // Frozen the CoinInfo object, so that no more coins can be minted
        object::to_frozen(coin_info_obj);
        let coin_store_obj = coin_store::create_coin_store<COIN_STRUCT_IDENTIFIER_PLACEHOLDER>(ctx);
        coin_store::deposit(&mut coin_store_obj, coin);
        let treasury_obj = context::new_named_object(ctx, Treasury { coin_store: coin_store_obj });
        // Make the treasury object to shared, so anyone can get mutable Treasury object
        object::to_shared(treasury_obj);
    }

     /// Provide a faucet to give out coins to users
    /// In a real world scenario, the coins should be given out in the application business logic.
    public entry fun faucet(ctx: &mut Context, account: &signer, treasury_obj: &mut Object<Treasury>) {
        let account_addr = signer::address_of(account);
        let treasury = object::borrow_mut(treasury_obj);
        let coin = coin_store::withdraw(&mut treasury.coin_store, 10000);
        account_coin_store::deposit(ctx, account_addr, coin);
    }
}