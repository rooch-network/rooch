/// This module provides the foundation for typesafe Coins.
module rooch_framework::coin {
    use std::string;
    use std::error;
    use std::signer;
    use moveos_std::type_table::TypeTable;
    use moveos_std::storage_context;
    use moveos_std::type_table;
    use moveos_std::account_storage;
    use moveos_std::storage_context::{StorageContext};
    use moveos_std::event;
    use moveos_std::type_info::{Self, TypeInfo, type_of};

    friend rooch_framework::account;

    //
    // Errors.
    //

    /// Address of account which is used to initialize a coin `CoinType` doesn't match the deployer of module
    const ErrorCoinInfoAddressMismatch: u64 = 1;

    /// `CoinType` is already initialized as a coin
    const ErrorCoinInfoAlreadyPublished: u64 = 2;

    /// Not enough coins to complete transaction
    const ErrorInSufficientBalance: u64 = 3;

    /// Cannot destroy non-zero coins
    const ErrorDestroyOfNonZeroCoin: u64 = 4;

    /// Coin amount cannot be zero
    const ErrorZeroCoinAmount: u64 = 5;

    /// Name of the coin is too long
    const ErrorCoinNameTooLong: u64 = 6;

    /// Symbol of the coin is too long
    const ErrorCoinSymbolTooLong: u64 = 7;

    //
    // Constants
    //

    const MAX_COIN_NAME_LENGTH: u64 = 32;
    const MAX_COIN_SYMBOL_LENGTH: u64 = 10;

    /// Core data structures

    /// Main structure representing a coin/coin in an account's custody.
    struct Coin<phantom CoinType> has store {
        /// Amount of coin this address has.
        /// Following the ERC20 standard, both asset balance and supply are expressed in u256
        value: u256,
    }

    /// Maximum possible aggregatable coin value.
    const MAX_U64: u128 = 18446744073709551615;

    // /// A holder of a specific coin types.
    // /// These are kept in a single resource to ensure locality of data.
    struct CoinStore<phantom CoinType> has key {
        coin: Coin<CoinType>,
        frozen: bool,
    }

    /// Maximum possible coin supply.
    const MAX_U128: u128 = 340282366920938463463374607431768211455;

    const MAX_U256: u256 = 115792089237316195423570985008687907853269984665640564039457584007913129639935;

    /// Information about a specific coin type. Stored on the creator of the coin's account.
    struct CoinInfo<phantom CoinType> has key {
        name: string::String,
        /// Symbol of the coin, usually a shorter version of the name.
        /// For example, Singapore Dollar is SGD.
        symbol: string::String,
        /// Number of decimals used to get its user representation.
        /// For example, if `decimals` equals `2`, a balance of `505` coins should
        /// be displayed to a user as `5.05` (`505 / 10 ** 2`).
        decimals: u8,
        /// The total value for the coin represented by `CoinType`. Mutable.
        supply: u256,
    }

    /// A resource that holds the CoinInfo for all accounts.
    struct CoinInfos has key {
        coin_infos: TypeTable,
    }

    /// Event emitted when coin minted.
    struct MintEvent has drop, store {
        /// full info of coin
        coin_type_info: TypeInfo,
        /// coins added to the system
        amount: u256,
    }

    /// Event emitted when coin burned.
    struct BurnEvent has drop, store {
        /// full info of coin
        coin_type_info: TypeInfo,
        /// coins removed from the system
        amount: u256,
    }

    /// Capability required to mint coins.
    struct MintCapability<phantom CoinType> has key, copy, store {}

    /// Capability required to freeze a coin store.
    struct FreezeCapability<phantom CoinType> has key, copy, store {}

    /// Capability required to burn coins.
    struct BurnCapability<phantom CoinType> has key, copy, store {}

    fun init(ctx: &mut StorageContext, account: &signer) {
        rooch_framework::core_addresses::assert_rooch_framework(account);
        let tx_ctx = storage_context::tx_context_mut(ctx);
        let coin_infos = type_table::new(tx_ctx);
        account_storage::global_move_to(ctx, account, CoinInfos{
            coin_infos,
        });
    }

    //
    // Getter functions
    //

    /// A helper function that returns the address of CoinType.
    fun coin_address<CoinType>(): address {
        let type_info = type_info::type_of<CoinType>();
        type_info::account_address(&type_info)
    }


    /// Returns the balance of `addr` for provided `CoinType`.
    public fun balance<CoinType>(ctx: &StorageContext, addr: address): u256 {
        if (account_storage::global_exists<CoinStore<CoinType>>(ctx, addr)) {
            account_storage::global_borrow<CoinStore<CoinType>>(ctx, addr).coin.value
        } else {
            0u256
        }
    }

    /// Returns `true` if the type `CoinType` is an initialized coin.
    public fun is_coin_initialized<CoinType>(ctx: &StorageContext): bool {
        if (account_storage::global_exists<CoinInfos>(ctx, @rooch_framework)) {
            let coin_infos = account_storage::global_borrow<CoinInfos>(ctx, @rooch_framework);
            type_table::contains<CoinInfo<CoinType>>(&coin_infos.coin_infos)
        } else {
            false
        }
    }

    /// Returns the name of the coin.
    public fun name<CoinType>(ctx: &StorageContext): string::String {
        borrow_coin_info<CoinType>(ctx).name
    }

    /// Returns the symbol of the coin, usually a shorter version of the name.
    public fun symbol<CoinType>(ctx: &StorageContext): string::String {
        borrow_coin_info<CoinType>(ctx).symbol
    }

    /// Returns the number of decimals used to get its user representation.
    /// For example, if `decimals` equals `2`, a balance of `505` coins should
    /// be displayed to a user as `5.05` (`505 / 10 ** 2`).
    public fun decimals<CoinType>(ctx: &StorageContext): u8 {
        borrow_coin_info<CoinType>(ctx).decimals
    }


    /// Returns the amount of coin in existence.
    public fun supply<CoinType>(ctx: &StorageContext): u256 {
        borrow_coin_info<CoinType>(ctx).supply
    }

    /// Return true if the type `CoinType1` is same with `CoinType2`
    public fun is_same_coin<CoinType1, CoinType2>(): bool {
        return type_of<CoinType1>() == type_of<CoinType2>()
    }

    fun borrow_coin_info<CoinType>(ctx: &StorageContext): &CoinInfo<CoinType> {
        let coin_infos = account_storage::global_borrow<CoinInfos>(ctx, @rooch_framework);
        type_table::borrow<CoinInfo<CoinType>>(&coin_infos.coin_infos)
    }

    fun borrow_mut_coin_info<CoinType>(ctx: &mut StorageContext): &mut CoinInfo<CoinType> {
        let coin_infos = account_storage::global_borrow_mut<CoinInfos>(ctx, @rooch_framework);
        type_table::borrow_mut<CoinInfo<CoinType>>(&mut coin_infos.coin_infos)
    }
    
    //
    // Public functions
    //

    /// Burn `coin` with capability.
    /// The capability `_cap` should be passed as a reference to `BurnCapability<CoinType>`.
    public fun burn<CoinType>(
        ctx: &mut StorageContext,
        coin: Coin<CoinType>,
        _cap: &BurnCapability<CoinType>,
    ) {
        let Coin { value: amount } = coin;

        let coin_type_info = type_info::type_of<CoinType>();
        let coin_info = borrow_mut_coin_info<CoinType>(ctx);
        coin_info.supply = coin_info.supply - amount;
        event::emit<BurnEvent>(ctx, BurnEvent {
            coin_type_info,
            amount,
        });
    }

    /// Burn `coin` from the specified `account` with capability.
    /// The capability `burn_cap` should be passed as a reference to `BurnCapability<CoinType>`.
    /// This function shouldn't fail as it's called as part of transaction fee burning.
    public fun burn_from<CoinType>(
        ctx: &mut StorageContext,
        addr: address,
        amount: u256,
        burn_cap: &BurnCapability<CoinType>,
    ) {
        let coin_store = account_storage::global_borrow_mut<CoinStore<CoinType>>(ctx, addr);
        let coin_to_burn = extract(&mut coin_store.coin, amount);
        burn(ctx, coin_to_burn, burn_cap);
    }

    /// Destroys a zero-value coin. Calls will fail if the `value` in the passed-in `coin` is non-zero
    /// so it is impossible to "burn" any non-zero amount of `Coin` without having
    /// a `BurnCapability` for the specific `CoinType`.
    public fun destroy_zero<CoinType>(zero_coin: Coin<CoinType>) {
        let Coin { value } = zero_coin;
        assert!(value == 0, error::invalid_argument(ErrorDestroyOfNonZeroCoin))
    }

    /// Extracts `amount` from the passed-in `coin`, where the original coin is modified in place.
    public fun extract<CoinType>(coin: &mut Coin<CoinType>, amount: u256): Coin<CoinType> {
        assert!(coin.value >= amount, error::invalid_argument(ErrorInSufficientBalance));
        coin.value = coin.value - amount;
        Coin { value: amount }
    }

    /// Extracts the entire amount from the passed-in `coin`, where the original coin is modified in place.
    public fun extract_all<CoinType>(coin: &mut Coin<CoinType>): Coin<CoinType> {
        let total_value = coin.value;
        coin.value = 0;
        Coin { value: total_value }
    }

    /// Freeze a CoinStore to prevent transfers
    public fun freeze_coin_store<CoinType>(
        ctx: &mut StorageContext,
        addr: address,
        _freeze_cap: &FreezeCapability<CoinType>,
    ) {
        let coin_store = account_storage::global_borrow_mut<CoinStore<CoinType>>(ctx, addr);
        coin_store.frozen = true;
    }

    /// Unfreeze a CoinStore to allow transfers
    public fun unfreeze_coin_store<CoinType>(
        ctx: &mut StorageContext,
        addr: address,
        _freeze_cap: &FreezeCapability<CoinType>,
    ) {
        let coin_store = account_storage::global_borrow_mut<CoinStore<CoinType>>(ctx, addr);
        coin_store.frozen = false;
    }

    // #[private_generics(CoinType)]
    /// Creates a new Coin with given `CoinType` and returns minting/freezing/burning capabilities.
    /// The given signer also becomes the account hosting the information about the coin
    /// (name, supply, etc.).
    public fun initialize<CoinType>(
        ctx: &mut StorageContext,
        account: &signer,
        name: string::String,
        symbol: string::String,
        decimals: u8,
    ): (BurnCapability<CoinType>, FreezeCapability<CoinType>, MintCapability<CoinType>) {
        let addr = signer::address_of(account);
        assert!(
            coin_address<CoinType>() == addr,
            error::invalid_argument(ErrorCoinInfoAddressMismatch),
        );

        assert!(
            !account_storage::global_exists<CoinInfo<CoinType>>(ctx, addr),
            error::already_exists(ErrorCoinInfoAlreadyPublished),
        );

        assert!(string::length(&name) <= MAX_COIN_NAME_LENGTH, error::invalid_argument(ErrorCoinNameTooLong));
        assert!(string::length(&symbol) <= MAX_COIN_SYMBOL_LENGTH, error::invalid_argument(ErrorCoinSymbolTooLong));

        let coin_info = CoinInfo<CoinType> {
            name,
            symbol,
            decimals,
            supply: 0u256,
        };
        // account_storage::global_move_to<CoinInfo<CoinType>>(ctx, account, coin_info);
        let coin_infos = account_storage::global_borrow_mut<CoinInfos>(ctx, @rooch_framework);
        type_table::add(&mut coin_infos.coin_infos, coin_info);

        (BurnCapability<CoinType> {}, FreezeCapability<CoinType> {}, MintCapability<CoinType> {})
    }

    /// "Merges" the two given coins.  The coin passed in as `dst_coin` will have a value equal
    /// to the sum of the two coins (`dst_coin` and `source_coin`).
    public fun merge<CoinType>(dst_coin: &mut Coin<CoinType>, source_coin: Coin<CoinType>) {
        let Coin { value } = source_coin;
        dst_coin.value = dst_coin.value + value;
    }

    /// Mint new `Coin` with capability.
    /// The capability `_cap` should be passed as reference to `MintCapability<CoinType>`.
    /// Returns minted `Coin`.
    public fun mint<CoinType>(
        ctx: &mut StorageContext,
        amount: u256,
        _cap: &MintCapability<CoinType>,
    ): Coin<CoinType> {
       let coin_info = borrow_mut_coin_info<CoinType>(ctx);
        coin_info.supply = coin_info.supply + amount;
        let coin_type_info = type_info::type_of<CoinType>();
        event::emit<MintEvent>(ctx, MintEvent {
            coin_type_info,
            amount,
        });
        Coin<CoinType> { value: amount }
    }

    /// Returns the `value` passed in `coin`.
    public fun value<CoinType>(coin: &Coin<CoinType>): u256 {
        coin.value
    }

    /// Create a new `Coin<CoinType>` with a value of `0`.
    public fun zero<CoinType>(): Coin<CoinType> {
        Coin<CoinType> {
            value: 0
        }
    }

    public fun initialize_coin_store<CoinType>(ctx: &mut StorageContext, account: &signer) {
        account_storage::global_move_to<CoinStore<CoinType>>(ctx, account, CoinStore<CoinType> {
            coin: Coin { value: 0 },
            frozen: false,
        });
    }

    public fun exist_coin_store<CoinType>(ctx: &StorageContext, addr: address): bool {
        account_storage::global_exists<CoinStore<CoinType>>(ctx, addr)
    }

    public fun is_coin_store_frozen<CoinType>(ctx: &StorageContext, addr: address): bool {
        if (account_storage::global_exists<CoinStore<CoinType>>(ctx, addr)) {
            account_storage::global_borrow<CoinStore<CoinType>>(ctx, addr).frozen
        } else {
            false
        }
    }

    // public(friend) fun borrow_mut_coin<CoinType>(ctx: &mut StorageContext, addr: address): &mut Coin<CoinType> {
    //     let coin_store = account_storage::global_borrow_mut<CoinStore<CoinType>>(ctx, addr);
    //     &mut coin_store.coin
    // }

    public(friend) fun extract_coin<CoinType>(ctx: &mut StorageContext, addr: address, amount: u256): Coin<CoinType> {
        let coin_store = account_storage::global_borrow_mut<CoinStore<CoinType>>(ctx, addr);
        extract<CoinType>(&mut coin_store.coin, amount)
    }

    public(friend) fun merge_coin<CoinType>(ctx: &mut StorageContext, addr: address, coin: Coin<CoinType>) {
        let coin_store = account_storage::global_borrow_mut<CoinStore<CoinType>>(ctx, addr);
        merge<CoinType>(&mut coin_store.coin, coin)
    }

    /// Destroy a freeze capability. Freeze capability is dangerous and therefore should be destroyed if not used.
    public fun destroy_freeze_cap<CoinType>(freeze_cap: FreezeCapability<CoinType>) {
        let FreezeCapability<CoinType> {} = freeze_cap;
    }

    /// Destroy a mint capability.
    public fun destroy_mint_cap<CoinType>(mint_cap: MintCapability<CoinType>) {
        let MintCapability<CoinType> {} = mint_cap;
    }

    /// Destroy a burn capability.
    public fun destroy_burn_cap<CoinType>(burn_cap: BurnCapability<CoinType>) {
        let BurnCapability<CoinType> {} = burn_cap;
    }

    #[test_only]
    public fun init_for_test(ctx: &mut StorageContext, account: &signer){
        init(ctx, account);
    }
}
