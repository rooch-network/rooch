module rooch_framework::transfer {    
    use moveos_std::context::Context;
    use rooch_framework::account;
    use rooch_framework::coin;
    use rooch_framework::multichain_address;
    use rooch_framework::address_mapping;

    /// Transfer `amount` of coins `CoinType` from `from` to `to`.
    /// This public entry function requires the `CoinType` to have `key` and `store` abilities.
    public entry fun transfer_coin<CoinType: key + store>(
        ctx: &mut Context,
        from: &signer,
        to: address,
        amount: u256,
    ) {
        if(!account::exists_at(ctx, to)) {
            account::create_account(ctx, to);
        };

        coin::transfer<CoinType>(ctx, from, to, amount)
    }

    /// Transfer `amount` of coins `CoinType` from `from` to a MultiChainAddress.
    /// The MultiChainAddress is represented by `multichain_id` and `raw_address`.
    /// This public entry function requires the `CoinType` to have `key` and `store` abilities.
    public entry fun transfer_coin_to_multichain_address<CoinType: key + store>(
        ctx: &mut Context,
        from: &signer,
        multichain_id: u64,
        raw_address: vector<u8>,
        amount: u256,
    ) {
        let maddress = multichain_address::new(multichain_id, raw_address);
        let to = address_mapping::resolve_or_generate(ctx, maddress);
        if(!account::exists_at(ctx, to)) {
            account::create_account(ctx, to);
            address_mapping::bind_no_check(ctx, to, maddress);
        };
        coin::transfer<CoinType>(ctx, from, to, amount)
    }
}