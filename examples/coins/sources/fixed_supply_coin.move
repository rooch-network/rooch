// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

module coins::fixed_supply_coin {

    use std::string;
    use moveos_std::signer;
    use moveos_std::context::{Self, Context};
    use moveos_std::object::{Self, Object};
    use rooch_framework::coin;
    use rooch_framework::coin_store::{Self, CoinStore};
    use rooch_framework::account_coin_store;


    struct FSC has key, store {}

    struct Treasury has key {
        coin_store: Object<CoinStore<FSC>>
    }

    const TOTAL_SUPPLY: u256 = 210_000_000_000u256;
    const DECIMALS: u8 = 1u8;


    fun init(ctx: &mut Context) {
        let coin_info_obj = coin::register_extend<FSC>(
            ctx,
            string::utf8(b"Fixed Supply Coin"),
            string::utf8(b"FSC"),
            DECIMALS,
        );
        // Mint the total supply of coins, and store it to the treasury
        let coin = coin::mint_extend<FSC>(&mut coin_info_obj, TOTAL_SUPPLY);
        // Frozen the CoinInfo object, so that no more coins can be minted
        object::to_frozen(coin_info_obj);
        let coin_store_obj = coin_store::create_coin_store<FSC>(ctx);
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
