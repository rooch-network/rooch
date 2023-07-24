module rooch_examples::coin {
    use std::error;
    use std::signer;

    use moveos_std::account_storage;
    use moveos_std::storage_context::StorageContext;
    use moveos_std::type_info;

    const ECOIN_INFO_ADDRESS_MISMATCH: u64 = 1;

    const ECOIN_INFO_ALREADY_PUBLISHED: u64 = 2;

    const ECOIN_INFO_NOT_PUBLISHED: u64 = 3;

    const ECOIN_STORE_ALREADY_PUBLISHED: u64 = 4;

    const ECOIN_STORE_NOT_PUBLISHED: u64 = 5;

    const EINSUFFICIENT_BALANCE: u64 = 6;

    const EDESTRUCTION_OF_NONZERO_TOKEN: u64 = 7;

    const EZERO_COIN_AMOUNT: u64 = 9;

    const EFROZEN: u64 = 10;

    const ECOIN_SUPPLY_UPGRADE_NOT_SUPPORTED: u64 = 11;

    const ECOIN_NAME_TOO_LONG: u64 = 12;

    const ECOIN_SYMBOL_TOO_LONG: u64 = 13;

    const EAGGREGATABLE_COIN_VALUE_TOO_LARGE: u64 = 14;

    const MAX_COIN_NAME_LENGTH: u64 = 32;
    const MAX_COIN_SYMBOL_LENGTH: u64 = 10;

    struct Coin<phantom CoinType> has key, store {
        value: u64,
    }


    const MAX_U64: u128 = 18446744073709551615;

    const MAX_U128: u128 = 340282366920938463463374607431768211455;

    struct MintCapability<phantom CoinType> has copy, store {}

    struct FreezeCapability<phantom CoinType> has copy, store {}

    struct BurnCapability<phantom CoinType> has copy, store {}

    public fun balance<CoinType>(ctx: &StorageContext, owner: address): u64 {
        account_storage::global_borrow<Coin<CoinType>>(ctx, owner).value
    }

    public fun is_account_registered<CoinType>(ctx: &StorageContext, owner: address): bool {
        account_storage::global_exists<Coin<CoinType>>(ctx, owner)
    }


    public fun burn<CoinType>(
        coin: Coin<CoinType>,
        _cap: &BurnCapability<CoinType>
    ) {
        let Coin { value: amount } = coin;
        assert!(amount > 0, error::invalid_argument(EZERO_COIN_AMOUNT));
    }

    public fun deposit<CoinType>(account_addr: address, coin: Coin<CoinType>,
                                 ctx: &mut StorageContext, ) {
        assert!(
            is_account_registered<CoinType>(ctx, account_addr),
            error::not_found(ECOIN_STORE_NOT_PUBLISHED),
        );

        let coin_mut_ref = account_storage::global_borrow_mut<Coin<CoinType>>(ctx, account_addr);
        merge(coin_mut_ref, coin);
    }

    public fun destroy_zero<CoinType>(zero_coin: Coin<CoinType>) {
        let Coin { value } = zero_coin;
        assert!(value == 0, error::invalid_argument(EDESTRUCTION_OF_NONZERO_TOKEN))
    }

    public fun extract<CoinType>(coin: &mut Coin<CoinType>, amount: u64): Coin<CoinType> {
        assert!(coin.value >= amount, error::invalid_argument(EINSUFFICIENT_BALANCE));

        coin.value = coin.value - amount;

        Coin { value: amount }
    }

    public fun extract_all<CoinType>(coin: &mut Coin<CoinType>): Coin<CoinType> {
        let total_value = coin.value;

        coin.value = 0;

        Coin { value: total_value }
    }

    public fun initialize<CoinType>(
        account: &signer
    ): (BurnCapability<CoinType>, FreezeCapability<CoinType>, MintCapability<CoinType>) {
        initialize_internal(account)
    }

    fun coin_address<CoinType>(): address {
        let type_info = type_info::type_of<CoinType>();
        type_info::account_address(&type_info)
    }

    fun initialize_internal<CoinType>(
        account: &signer
    ): (BurnCapability<CoinType>, FreezeCapability<CoinType>, MintCapability<CoinType>) {
        let account_addr = signer::address_of(account);

        assert!(
            coin_address<CoinType>() == account_addr,
            error::invalid_argument(ECOIN_INFO_ADDRESS_MISMATCH),
        );

        (BurnCapability<CoinType> {}, FreezeCapability<CoinType> {}, MintCapability<CoinType> {})
    }

    public fun merge<CoinType>(dst_coin: &mut Coin<CoinType>, source_coin: Coin<CoinType>) {
        let Coin { value } = source_coin;

        dst_coin.value = dst_coin.value + value;
    }

    public fun global_borrow<CoinType>(ctx: &StorageContext, owner: address): &Coin<CoinType> {
        account_storage::global_borrow<Coin<CoinType>>(ctx, owner)
    }

    public fun global_borrow_mut<CoinType>(ctx: &mut StorageContext, owner: address): &mut Coin<CoinType> {
        account_storage::global_borrow_mut<Coin<CoinType>>(ctx, owner)
    }

    public fun mint<CoinType>(
        amount: u64,
        _cap: &MintCapability<CoinType>,
    ): Coin<CoinType> {
        if (amount == 0) {
            return Coin<CoinType> {
                value: 0
            }
        };

        Coin<CoinType> { value: amount }
    }

    public fun register<CoinType>(account: &signer, ctx: &mut StorageContext) {
        let account_addr = signer::address_of(account);
        if (is_account_registered<CoinType>(ctx, account_addr)) {
            return
        };

        account_storage::global_move_to(ctx, account, Coin<CoinType> {
            value: 0
        });
    }

    public entry fun transfer<CoinType>(
        from: &signer,
        to: address,
        amount: u64, ctx: &mut StorageContext,
    ) {
        let coin = withdraw<CoinType>(from, amount, ctx);
        deposit(to, coin, ctx);
    }

    public fun value<CoinType>(coin: &Coin<CoinType>): u64 {
        coin.value
    }

    public fun withdraw<CoinType>(
        account: &signer,
        amount: u64, ctx: &mut StorageContext,
    ): Coin<CoinType> {
        let account_addr = signer::address_of(account);
        assert!(
            is_account_registered<CoinType>(ctx, account_addr),
            error::not_found(ECOIN_STORE_NOT_PUBLISHED),
        );
        let coin_mut_ref = account_storage::global_borrow_mut<Coin<CoinType>>(ctx, account_addr);

        extract(coin_mut_ref, amount)
    }

    public fun zero<CoinType>(): Coin<CoinType> {
        Coin<CoinType> {
            value: 0
        }
    }

    public fun destroy_freeze_cap<CoinType>(freeze_cap: FreezeCapability<CoinType>) {
        let FreezeCapability<CoinType> {} = freeze_cap;
    }

    public fun destroy_mint_cap<CoinType>(mint_cap: MintCapability<CoinType>) {
        let MintCapability<CoinType> {} = mint_cap;
    }

    public fun destroy_burn_cap<CoinType>(burn_cap: BurnCapability<CoinType>) {
        let BurnCapability<CoinType> {} = burn_cap;
    }
}
