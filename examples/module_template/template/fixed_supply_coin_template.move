// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

module 0xdeadeadeadeadeadeadeadeadeadeadeadeadeadeadeadeadeadeadeadeadead::coin_module_identifier_placeholder {

    use std::string;
    use moveos_std::signer;
    use moveos_std::account_storage;
    use moveos_std::context::Context;
    use moveos_std::object::{Self, Object};
    use rooch_framework::coin;
    use rooch_framework::coin_store::{Self, CoinStore};
    use rooch_framework::account_coin_store;
    

    struct COIN_STRUCT_IDENTIFIER_PLACEHOLDER has key, store {}

    struct Treasury has key {
        coin_store: Object<CoinStore>
    }
    
    const TOTAL_SUPPLY: u256 = 123_321_123_456u256;
    const DECIMALS: u8 = 222u8; 


    fun init(ctx: &mut Context) {
        coin::register_extend<COIN_STRUCT_IDENTIFIER_PLACEHOLDER>(
            ctx,
            string::utf8(b"COIN_NAME_PLACEHOLDER"),
            string::utf8(b"COIN_SYMBOL_PLACEHOLDER"),
            DECIMALS,
        );
        let coins_signer = signer::module_signer<COIN_STRUCT_IDENTIFIER_PLACEHOLDER>();
        // Mint the total supply of coins, and store it to the treasury
        let coin = coin::mint_extend<COIN_STRUCT_IDENTIFIER_PLACEHOLDER>(ctx, TOTAL_SUPPLY);
        let coin_store_ref = coin_store::create_coin_store<COIN_STRUCT_IDENTIFIER_PLACEHOLDER>(ctx);
        coin_store::deposit(object::borrow_mut(&mut coin_store_ref), coin);
        account_storage::global_move_to(ctx, &coins_signer, Treasury { coin_store: coin_store_ref });
    }

    /// Provide a faucet to give out coins to users
    /// In a real world scenario, the coins should be given out in the application business logic.
    public entry fun faucet(ctx: &mut Context, account: &signer) {
        let account_addr = signer::address_of(account);
        let treasury = account_storage::global_borrow_mut<Treasury>(ctx, @0xdeadeadeadeadeadeadeadeadeadeadeadeadeadeadeadeadeadeadeadeadead);
        let coin = coin_store::withdraw<COIN_STRUCT_IDENTIFIER_PLACEHOLDER>(object::borrow_mut(&mut treasury.coin_store), 10000);
        account_coin_store::deposit(ctx, account_addr, coin);
    }
}