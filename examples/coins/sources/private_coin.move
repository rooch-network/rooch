// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

module coins::private_coin {

    use std::error;
    use std::string;
    use moveos_std::signer;
    use moveos_std::context::Context;
    use rooch_framework::coin;

    const ErrorTransferAmountTooLarge: u64 = 1;

    /// This Coin has no `store` ability, 
    /// so it can not be operate via `coin::transfer`, `coin::deposit` and `coin::withdraw`
    struct PRC has key {}


    fun init(ctx: &mut Context) {
        coin::register_extend<PRC>(
            ctx,
            string::utf8(b"Private Coin"),
            string::utf8(b"PRC"),
            1,
        );
    }

    /// Provide a faucet to give out coins to users
    /// In a real world scenario, the coins should be given out in the application business logic.
    public entry fun faucet(ctx: &mut Context, account: &signer) {
        let account_addr = signer::address_of(account);
        let coin = coin::mint_extend<PRC>(ctx, 10000);
        coin::deposit_extend(ctx, account_addr, coin);
    }

    /// This function shows how to use `coin::transfer_extend` to define a custom transfer logic
    /// This transfer function limits the amount of transfer to 100
    public entry fun transfer(ctx: &mut Context, from: &signer, to_addr: address, amount: u256) {
        assert!(amount <= 100u256, error::invalid_argument(ErrorTransferAmountTooLarge));
        let from_addr = signer::address_of(from);
        coin::transfer_extend<PRC>(ctx, from_addr, to_addr, amount);
    }
}
