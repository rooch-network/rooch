module rooch_framework::transfer {
    use rooch_framework::account;
    use moveos_std::storage_context::StorageContext;
    use rooch_framework::coin;

    /// Transfer `amount` of coins `CoinType` from `from` to `to`.
    /// This public entry function requires the `CoinType` to have `key` and `store` abilities.
    public entry fun transfer_coin<CoinType: key + store>(
        ctx: &mut StorageContext,
        from: &signer,
        to: address,
        amount: u256,
    ) {
        if(!account::exists_at(ctx, to)) {
            account::create_account(ctx, to);
        };

        coin::transfer<CoinType>(ctx, from, to, amount)
    }
}