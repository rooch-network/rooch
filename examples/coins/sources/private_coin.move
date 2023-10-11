// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

module coins::private_coin {

    use std::error;
    use std::string;
    use moveos_std::signer;
    use moveos_std::context::Context;
    use moveos_std::object_ref::{Self, ObjectRef};
    use moveos_std::account_storage;
    use rooch_framework::coin::{Self, Coin};
    use rooch_framework::coin_store::{Self, CoinStore};
    use rooch_framework::account_coin_store;

    const ErrorTransferAmountTooLarge: u64 = 1;

    /// This Coin has no `store` ability, 
    /// so it can not be operate via `account_coin_store::transfer`, `account_coin_store::deposit` and `account_coin_store::withdraw`
    struct PRC has key {}

    struct Treasury has key {
        coin_store: ObjectRef<CoinStore>
    }

    fun init(ctx: &mut Context) {
        coin::register_extend<PRC>(
            ctx,
            string::utf8(b"Private Coin"),
            string::utf8(b"PRC"),
            1,
        );
        let coins_signer = signer::module_signer<PRC>();
        let coin_store_ref = coin_store::create_coin_store_extend<PRC>(ctx);
        account_storage::global_move_to(ctx, &coins_signer, Treasury { coin_store: coin_store_ref });
    }

    /// Provide a faucet to give out coins to users
    /// In a real world scenario, the coins should be given out in the application business logic.
    public entry fun faucet(ctx: &mut Context, account: &signer) {
        let account_addr = signer::address_of(account);
        let coin = coin::mint_extend<PRC>(ctx, 10000);
        account_coin_store::deposit_extend(ctx, account_addr, coin);
    }

    /// This function shows how to use `coin::transfer_extend` to define a custom transfer logic
    /// This transfer function limits the amount of transfer to 10000, and take 1% of the amount as fee
    public entry fun transfer(ctx: &mut Context, from: &signer, to_addr: address, amount: u256) {
        assert!(amount <= 10000u256, error::invalid_argument(ErrorTransferAmountTooLarge));
        let from_addr = signer::address_of(from);
        let fee_amount = amount / 100u256;
        if (fee_amount > 0u256) {
            let fee = account_coin_store::withdraw_extend<PRC>(ctx, from_addr, fee_amount);
            deposit_to_treaury(ctx, fee);
        };
        account_coin_store::transfer_extend<PRC>(ctx, from_addr, to_addr, amount);
    }

    fun deposit_to_treaury(ctx: &mut Context, coin: Coin<PRC>) {
        let treasury = account_storage::global_borrow_mut<Treasury>(ctx, @coins);
        coin_store::deposit(object_ref::borrow_mut(&mut treasury.coin_store), coin);
    }
}
