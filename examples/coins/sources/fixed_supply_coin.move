// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

module coins::fixed_supply_coin {

    use std::string;
    use moveos_std::signer;
    use moveos_std::account_storage;
    use moveos_std::context::Context;
    use moveos_std::object_ref::{Self, ObjectRef};
    use rooch_framework::coin;
    use rooch_framework::coin_store::{Self, CoinStore};
    use rooch_framework::account_coin_store;
    

    struct FSC has key, store {}

    struct Treasury has key {
        coin_store: ObjectRef<CoinStore>
    }

    const TOTAL_SUPPLY: u256 = 210_000_000_000u256;


    fun init(ctx: &mut Context) {
        coin::register_extend<FSC>(
            ctx,
            string::utf8(b"Fixed Supply Coin"),
            string::utf8(b"FSC"),
            1,
        );
        let coins_signer = signer::module_signer<FSC>();
        // Mint the total supply of coins, and store it to the treasury
        let coin = coin::mint_extend<FSC>(ctx, TOTAL_SUPPLY);
        let coin_store_ref = coin_store::create_coin_store<FSC>(ctx);
        coin_store::deposit(object_ref::borrow_mut(&mut coin_store_ref), coin);
        account_storage::global_move_to(ctx, &coins_signer, Treasury { coin_store: coin_store_ref });
    }

    /// Provide a faucet to give out coins to users
    /// In a real world scenario, the coins should be given out in the application business logic.
    public entry fun faucet(ctx: &mut Context, account: &signer) {
        let account_addr = signer::address_of(account);
        let treasury = account_storage::global_borrow_mut<Treasury>(ctx, @coins);
        let coin = coin_store::withdraw<FSC>(object_ref::borrow_mut(&mut treasury.coin_store), 10000);
        account_coin_store::deposit(ctx, account_addr, coin);
    }
}
