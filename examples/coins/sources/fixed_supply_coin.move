module coins::fixed_supply_coin {

    use std::string;
    use moveos_std::signer;
    use moveos_std::account_storage;
    use moveos_std::storage_context::StorageContext;
    use rooch_framework::coin::{Self, Coin};

    struct FSC has key, store {}

    struct Treasury has key {
        coin: Coin<FSC>
    }

    const TOTAL_SUPPLY: u256 = 210_000_000_000u256;


    fun init(ctx: &mut StorageContext) {
        //TODO remove this check after https://github.com/rooch-network/rooch/issues/742 is fixed
        if (coin::is_registered<FSC>(ctx)) {
            return
        };
        coin::register_extend<FSC>(
            ctx,
            string::utf8(b"Fixed Supply Coin"),
            string::utf8(b"FSC"),
            1,
        );
        let coins_signer = signer::module_signer<FSC>();
        // Mint the total supply of coins, and store it to the treasury
        let coin = coin::mint_extend<FSC>(ctx, TOTAL_SUPPLY);
        account_storage::global_move_to(ctx, &coins_signer, Treasury { coin });
    }

    /// Provide a faucet to give out coins to users
    /// In a real world scenario, the coins should be given out in the application business logic.
    public entry fun faucet(ctx: &mut StorageContext, account: &signer) {
        let account_addr = signer::address_of(account);
        let treasury = account_storage::global_borrow_mut<Treasury>(ctx, @coins);
        let coin = coin::extract(&mut treasury.coin, 10000);
        coin::deposit(ctx, account_addr, coin);
    }
}