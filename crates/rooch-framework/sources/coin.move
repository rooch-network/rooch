/// This module provides the foundation for typesafe Coins.
module rooch_framework::coin {
    use std::string;
    use std::error;
    use std::option;
    use std::option::Option;
    use moveos_std::object_id::ObjectID;
    use moveos_std::table;
    use moveos_std::table::Table;
    use moveos_std::type_table::{TypeTable};
    use moveos_std::storage_context;
    use moveos_std::type_table;
    use moveos_std::account_storage;
    use moveos_std::storage_context::{StorageContext};
    use moveos_std::event;
    use moveos_std::type_info::{Self, TypeInfo, type_of};
    use moveos_std::signer;

    friend rooch_framework::account;
    friend rooch_framework::genesis;

    //
    // Errors.
    //

    /// `CoinType` is not registered as a coin
    const ErrorCoinInfoNotRegistered: u64 = 0;

    /// `CoinType` is already registered as a coin
    const ErrorCoinInfoAlreadyRegistered: u64 = 1;

    /// Not enough coins to complete transaction
    const ErrorInSufficientBalance: u64 = 2;

    /// Cannot destroy non-zero coins
    const ErrorDestroyOfNonZeroCoin: u64 = 3;

    /// Coin amount cannot be zero
    const ErrorZeroCoinAmount: u64 = 4;

    /// Name of the coin is too long
    const ErrorCoinNameTooLong: u64 = 5;

    /// Symbol of the coin is too long
    const ErrorCoinSymbolTooLong: u64 = 6;

    /// CoinStore is frozen. Coins cannot be deposited or withdrawn
    const ErrorAccountWithCoinFrozen: u64 = 7;

    /// Account hasn't accept `CoinType`
    const ErrorAccountNotAcceptCoin: u64 = 8;


    //
    // Constants
    //

    const MAX_COIN_NAME_LENGTH: u64 = 32;
    const MAX_COIN_SYMBOL_LENGTH: u64 = 10;

    /// Core data structures

    /// Main structure representing a coin/coin in an account's custody.
    /// Note the `CoinType` must have `key` ability.
    /// if the `CoinType` has `store` ability, the `Coin` is a public coin, the user can operate it directly by coin module's function.
    /// Otherwise, the `Coin` is a private coin, the user can only operate it by `CoinType` module's function.
    struct Coin<phantom CoinType : key> has store {
        /// Amount of coin this address has.
        /// Following the ERC20 standard, both asset balance and supply are expressed in u256
        value: u256,
    }

    /// Maximum possible aggregatable coin value.
    const MAX_U64: u128 = 18446744073709551615;

    // /// A holder of a specific coin types.
    // /// These are kept in a single resource to ensure locality of data.
    struct CoinStore<phantom CoinType : key> has key {
        coin: Coin<CoinType>,
        frozen: bool,
    }

    /// Maximum possible coin supply.
    const MAX_U128: u128 = 340282366920938463463374607431768211455;

    const MAX_U256: u256 = 115792089237316195423570985008687907853269984665640564039457584007913129639935;

    /// Information about a specific coin type. Stored on the creator of the coin's account.
    struct CoinInfo<phantom CoinType : key> has key {
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

    /// A resource that holds the AutoAcceptCoin config for all accounts.
    /// The main scenario is that the user can actively turn off the AutoAcceptCoin setting to avoid automatically receiving Coin
    struct AutoAcceptCoins has key {
        auto_accept_coins: Table<address, bool>,
    }

    /// A resource that holds all the CoinStore for account.
    /// Default Deposit Coin no longer depends on accept coin
    struct CoinStores has key {
        coin_stores: TypeTable,
    }

    /// Event emitted when some amount of a coin is deposited into an account.
    struct DepositEvent has drop, store {
        /// The type info for the coin that was sent
        coin_type_info: TypeInfo,
        amount: u256,
    }

    /// Event emitted when some amount of a coin is withdrawn from an account.
    struct WithdrawEvent has drop, store {
        /// The type info for the coin that was sent
        coin_type_info: TypeInfo,
        amount: u256,
    }

    /// Event for auto accept coin set
    struct AcceptCoinEvent has drop, store {
        /// auto accept coin config
        enable: bool,
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

    public(friend) fun genesis_init(ctx: &mut StorageContext, genesis_account: &signer) {
        let tx_ctx = storage_context::tx_context_mut(ctx);
        account_storage::global_move_to(ctx, genesis_account, CoinInfos{
            coin_infos: type_table::new(tx_ctx),
        });
        let tx_ctx = storage_context::tx_context_mut(ctx);
        account_storage::global_move_to(ctx, genesis_account, AutoAcceptCoins{
            auto_accept_coins: table::new<address, bool>(tx_ctx),
        });
    }

    public(friend) fun init_account_coin_store(ctx: &mut StorageContext, account: &signer){
        let tx_ctx = storage_context::tx_context_mut(ctx);
        account_storage::global_move_to(ctx, account, CoinStores{
            coin_stores: type_table::new(tx_ctx),
        });
    }

    //
    // Getter functions
    //

    /// A helper function that returns the address of CoinType.
    fun coin_address<CoinType: key>(): address {
        let type_info = type_info::type_of<CoinType>();
        type_info::account_address(&type_info)
    }


    /// Returns the balance of `addr` for provided `CoinType`.
    public fun balance<CoinType: key>(ctx: &StorageContext, addr: address): u256 {
        if (exist_coin_store<CoinType>(ctx, addr)) {
            borrow_coin_store<CoinType>(ctx, addr).coin.value
        } else {
            0u256
        }
    }

    /// Returns `true` if the type `CoinType` is an registered coin.
    public fun is_registered<CoinType: key>(ctx: &StorageContext): bool {
        if (account_storage::global_exists<CoinInfos>(ctx, @rooch_framework)) {
            let coin_infos = account_storage::global_borrow<CoinInfos>(ctx, @rooch_framework);
            type_table::contains<CoinInfo<CoinType>>(&coin_infos.coin_infos)
        } else {
            false
        }
    }

    /// Returns the name of the coin.
    public fun name<CoinType: key>(ctx: &StorageContext): string::String {
        borrow_coin_info<CoinType>(ctx).name
    }

    /// Returns the symbol of the coin, usually a shorter version of the name.
    public fun symbol<CoinType: key>(ctx: &StorageContext): string::String {
        borrow_coin_info<CoinType>(ctx).symbol
    }

    /// Returns the number of decimals used to get its user representation.
    /// For example, if `decimals` equals `2`, a balance of `505` coins should
    /// be displayed to a user as `5.05` (`505 / 10 ** 2`).
    public fun decimals<CoinType: key>(ctx: &StorageContext): u8 {
        borrow_coin_info<CoinType>(ctx).decimals
    }


    /// Returns the amount of coin in existence.
    public fun supply<CoinType: key>(ctx: &StorageContext): u256 {
        borrow_coin_info<CoinType>(ctx).supply
    }

    /// Return true if the type `CoinType1` is same with `CoinType2`
    public fun is_same_coin<CoinType1, CoinType2>(): bool {
        return type_of<CoinType1>() == type_of<CoinType2>()
    }

    /// Return coin store handle for addr with coin type `CoinType`
    public fun coin_store_handle<CoinType: key>(ctx: &StorageContext, addr: address): Option<ObjectID> {
        if (exist_coin_store<CoinType>(ctx, addr))
        {
            let coin_stores = account_storage::global_borrow<CoinStores>(ctx, addr);
            option::some(*type_table::handle<CoinType>(&coin_stores.coin_stores))
        } else {
            option::none<ObjectID>()
        }
    }

    //
    // Helper functions
    //

    fun borrow_coin_info<CoinType: key>(ctx: &StorageContext): &CoinInfo<CoinType> {
        let coin_infos = account_storage::global_borrow<CoinInfos>(ctx, @rooch_framework);
        check_coin_info_registered<CoinType>(coin_infos);
        type_table::borrow<CoinInfo<CoinType>>(&coin_infos.coin_infos)
    }

    fun borrow_mut_coin_info<CoinType: key>(ctx: &mut StorageContext): &mut CoinInfo<CoinType> {
        let coin_infos = account_storage::global_borrow_mut<CoinInfos>(ctx, @rooch_framework);
        check_coin_info_registered<CoinType>(coin_infos);
        type_table::borrow_mut<CoinInfo<CoinType>>(&mut coin_infos.coin_infos)
    }

    fun exist_auto_accept_token(ctx: &StorageContext, addr: address): bool {
        let auto_accept_coins = account_storage::global_borrow<AutoAcceptCoins>(ctx, @rooch_framework);
        table::contains<address, bool>(&auto_accept_coins.auto_accept_coins, addr)
    }

    fun borrow_coin_store<CoinType: key>(ctx: &StorageContext, addr: address): &CoinStore<CoinType> {
        let coin_stores = account_storage::global_borrow<CoinStores>(ctx, addr);
        type_table::borrow<CoinStore<CoinType>>(&coin_stores.coin_stores)
    }

    fun borrow_mut_coin_store<CoinType: key>(ctx: &mut StorageContext, addr: address): &mut CoinStore<CoinType> {
        let coin_stores = account_storage::global_borrow_mut<CoinStores>(ctx, addr);
        type_table::borrow_mut<CoinStore<CoinType>>(&mut coin_stores.coin_stores)
    }

    fun ensure_coin_store<CoinType: key>(ctx: &mut StorageContext, addr: address) {
        if (!exist_coin_store<CoinType>(ctx, addr) && can_auto_accept_coin(ctx, addr)) {
            inner_new_coin_store<CoinType>(ctx, addr)
        }
    }

    fun ensure_coin_store_pass_auto_accept_flag<CoinType: key>(ctx: &mut StorageContext, addr: address) {
        if (!exist_coin_store<CoinType>(ctx, addr)) {
            inner_new_coin_store<CoinType>(ctx, addr)
        }
    }

    fun inner_new_coin_store<CoinType: key>(ctx: &mut StorageContext, addr: address) {
        let coin_stores = account_storage::global_borrow_mut<CoinStores>(ctx, addr);
        type_table::add<CoinStore<CoinType>>(&mut coin_stores.coin_stores, CoinStore<CoinType> {
            coin: Coin { value: 0 },
            frozen: false,
        })
    }

    fun extract_coin<CoinType: key>(ctx: &mut StorageContext, addr: address, amount: u256): Coin<CoinType> {
        let coin_store = borrow_mut_coin_store<CoinType>(ctx, addr);
        extract<CoinType>(&mut coin_store.coin, amount)
    }

    fun merge_coin<CoinType: key>(ctx: &mut StorageContext, addr: address, coin: Coin<CoinType>) {
        let coin_store = borrow_mut_coin_store<CoinType>(ctx, addr);
        merge<CoinType>(&mut coin_store.coin, coin)
    }

    fun check_coin_store_frozen<CoinType: key>(ctx: &StorageContext, addr: address) {
        assert!(
            !is_coin_store_frozen<CoinType>(ctx, addr),
            error::permission_denied(ErrorAccountWithCoinFrozen),
        );
    }

    fun check_coin_info_registered<CoinType: key>(coin_infos: &CoinInfos) {
        assert!(
            type_table::contains<CoinInfo<CoinType>>(&coin_infos.coin_infos),
            error::not_found(ErrorCoinInfoNotRegistered),
        );
    }

    //
    // Internal functions
    //

    fun mint_internal<CoinType: key>(ctx: &mut StorageContext,
        amount: u256): Coin<CoinType>{
        let coin_info = borrow_mut_coin_info<CoinType>(ctx);
        coin_info.supply = coin_info.supply + amount;
        let coin_type_info = type_info::type_of<CoinType>();
        event::emit<MintEvent>(ctx, MintEvent {
            coin_type_info,
            amount,
        });
        Coin<CoinType> { value: amount }
    }

    fun withdraw_internal<CoinType: key>(
        ctx: &mut StorageContext,
        addr: address,
        amount: u256,
    ): Coin<CoinType> {
        assert!(
            is_account_accept_coin<CoinType>(ctx, addr),
            error::not_found(ErrorAccountNotAcceptCoin),
        );

        
        ensure_coin_store<CoinType>(ctx, addr);
        let coin_type_info = type_info::type_of<CoinType>();
        event::emit<WithdrawEvent>(ctx, WithdrawEvent {
            coin_type_info,
            amount,
        });

        extract_coin(ctx, addr, amount)
    }

    fun deposit_internal<CoinType: key>(ctx: &mut StorageContext, addr: address, coin: Coin<CoinType>) {
        assert!(
            is_account_accept_coin<CoinType>(ctx, addr),
            error::not_found(ErrorAccountNotAcceptCoin),
        );

        ensure_coin_store<CoinType>(ctx, addr);
        let coin_type_info = type_info::type_of<CoinType>();
        event::emit<DepositEvent>(ctx, DepositEvent {
            coin_type_info,
            amount: value(&coin),
        });

        merge_coin(ctx, addr, coin);
    }

    fun transfer_internal<CoinType: key>(
        ctx: &mut StorageContext,
        from: address,
        to: address,
        amount: u256,
    ) {
        let coin = withdraw_internal<CoinType>(ctx, from, amount);
        deposit_internal(ctx, to, coin);
    }

    fun burn_internal<CoinType: key>(
        ctx: &mut StorageContext,
        coin: Coin<CoinType>,
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


    //
    // Public functions
    //

    /// Return whether the account at `addr` accept `Coin` type coins
    public fun is_account_accept_coin<CoinType: key>(ctx: &StorageContext, addr: address): bool {
        if (can_auto_accept_coin(ctx, addr)) {
            true
        } else {
            exist_coin_store<CoinType>(ctx, addr)
        }
    }

    /// Check whether the address can auto accept coin.
    /// Default is true if absent
    public fun can_auto_accept_coin(ctx: &StorageContext, addr: address): bool {
        if (account_storage::global_exists<AutoAcceptCoins>(ctx, @rooch_framework)) {
            let auto_accept_coins = account_storage::global_borrow<AutoAcceptCoins>(ctx, @rooch_framework);
            if (table::contains<address, bool>(&auto_accept_coins.auto_accept_coins, addr)) {
                return *table::borrow<address, bool>(&auto_accept_coins.auto_accept_coins, addr)
            }
        };
        true
    }

    /// Add a balance of `Coin` type to the sending account.
    /// If user turns off AutoAcceptCoin, call this method to receive the corresponding Coin
    public fun do_accept_coin<CoinType: key>(ctx: &mut StorageContext, account: &signer) {
        let addr = signer::address_of(account);
        ensure_coin_store_pass_auto_accept_flag<CoinType>(ctx, addr);
    }

    /// Configure whether auto-accept coins.
    public fun set_auto_accept_coin(ctx: &mut StorageContext, account: &signer, enable: bool)  {
        let addr = signer::address_of(account);
        let auto_accept_coins = account_storage::global_borrow_mut<AutoAcceptCoins>(ctx, @rooch_framework);
        table::upsert<address, bool>(&mut auto_accept_coins.auto_accept_coins, addr, enable);

        event::emit<AcceptCoinEvent>(ctx,
            AcceptCoinEvent {
                enable,
            },
        );
    }

    /// Withdraw specifed `amount` of coin `CoinType` from the signing account.
    /// This public entry function requires the `CoinType` to have `key` and `store` abilities.
    public fun withdraw<CoinType: key + store>(
        ctx: &mut StorageContext,
        account: &signer,
        amount: u256,
    ): Coin<CoinType> {
        let addr = signer::address_of(account);
        // the coin `frozen` only affect user withdraw, does not affect `withdraw_extend`. 
        check_coin_store_frozen<CoinType>(ctx, addr);
        withdraw_internal<CoinType>(ctx, addr, amount) 
    }

    /// Deposit the coin into the recipient's account and emit an event.
    /// This public entry function requires the `CoinType` to have `key` and `store` abilities.
    public fun deposit<CoinType: key + store>(ctx: &mut StorageContext, addr: address, coin: Coin<CoinType>) {
        check_coin_store_frozen<CoinType>(ctx, addr);
        deposit_internal(ctx, addr, coin);
    }

    /// Transfer `amount` of coins `CoinType` from `from` to `to`.
    /// Any account and module can call this function to transfer coins, the `CoinType` must have `key` and `store` abilities.
    public fun transfer<CoinType: key + store>(
        ctx: &mut StorageContext,
        from: &signer,
        to: address,
        amount: u256,
    ) {
        let from_addr = signer::address_of(from);
        check_coin_store_frozen<CoinType>(ctx, from_addr);
        check_coin_store_frozen<CoinType>(ctx, to);
        transfer_internal<CoinType>(ctx, from_addr, to, amount);
    }

    /// Destroys a zero-value coin. Calls will fail if the `value` in the passed-in `coin` is non-zero
    /// so it is impossible to "burn" any non-zero amount of `Coin`. 
    public fun destroy_zero<CoinType: key>(zero_coin: Coin<CoinType>) {
        let Coin { value } = zero_coin;
        assert!(value == 0, error::invalid_argument(ErrorDestroyOfNonZeroCoin))
    }

    /// Extracts `amount` from the passed-in `coin`, where the original coin is modified in place.
    public fun extract<CoinType: key>(coin: &mut Coin<CoinType>, amount: u256): Coin<CoinType> {
        assert!(coin.value >= amount, error::invalid_argument(ErrorInSufficientBalance));
        coin.value = coin.value - amount;
        Coin { value: amount }
    }

    /// Extracts the entire amount from the passed-in `coin`, where the original coin is modified in place.
    public fun extract_all<CoinType: key>(coin: &mut Coin<CoinType>): Coin<CoinType> {
        let total_value = coin.value;
        coin.value = 0;
        Coin { value: total_value }
    }

    /// "Merges" the two given coins.  The coin passed in as `dst_coin` will have a value equal
    /// to the sum of the two coins (`dst_coin` and `source_coin`).
    public fun merge<CoinType: key>(dst_coin: &mut Coin<CoinType>, source_coin: Coin<CoinType>) {
        let Coin { value } = source_coin;
        dst_coin.value = dst_coin.value + value;
    }

    /// Returns the `value` passed in `coin`.
    public fun value<CoinType: key>(coin: &Coin<CoinType>): u256 {
        coin.value
    }

    /// Create a new `Coin<CoinType>` with a value of `0`.
    public fun zero<CoinType: key>(): Coin<CoinType> {
        Coin<CoinType> {
            value: 0
        }
    }

    public fun exist_coin_store<CoinType: key>(ctx: &StorageContext, addr: address): bool {
        if (account_storage::global_exists<CoinStores>(ctx, addr)) {
            let coin_stores = account_storage::global_borrow<CoinStores>(ctx, addr);
            type_table::contains<CoinStore<CoinType>>(&coin_stores.coin_stores)
        } else {
            false
        }
    }

    public fun is_coin_store_frozen<CoinType: key>(ctx: &StorageContext, addr: address): bool {
        if (exist_coin_store<CoinType>(ctx, addr)) {
            borrow_coin_store<CoinType>(ctx, addr).frozen
        } else {
            false
        }
    }

    //
    // Extend functions
    //

    #[private_generics(CoinType)]
    /// Creates a new Coin with given `CoinType`
    /// The given signer also becomes the account hosting the information about the coin
    /// (name, supply, etc.).
    /// This function is protected by `private_generics`, so it can only be called by the `CoinType` module.
    public fun register_extend<CoinType: key>(
        ctx: &mut StorageContext,
        name: string::String,
        symbol: string::String,
        decimals: u8,
    ){
        
        let coin_infos = account_storage::global_borrow_mut<CoinInfos>(ctx, @rooch_framework);
        
        assert!(
            !type_table::contains<CoinInfo<CoinType>>(&coin_infos.coin_infos),
            error::already_exists(ErrorCoinInfoAlreadyRegistered),
        );

        assert!(string::length(&name) <= MAX_COIN_NAME_LENGTH, error::invalid_argument(ErrorCoinNameTooLong));
        assert!(string::length(&symbol) <= MAX_COIN_SYMBOL_LENGTH, error::invalid_argument(ErrorCoinSymbolTooLong));

        let coin_info = CoinInfo<CoinType> {
            name,
            symbol,
            decimals,
            supply: 0u256,
        };
        type_table::add(&mut coin_infos.coin_infos, coin_info);
    }

    #[private_generics(CoinType)]
    /// Mint new `Coin`, this function is only called by the `CoinType` module, for the developer to extend custom mint logic
    public fun mint_extend<CoinType: key>(ctx: &mut StorageContext,amount: u256) : Coin<CoinType> {
        mint_internal<CoinType>(ctx, amount)
    }

    #[private_generics(CoinType)]
    /// Withdraw specifed `amount` of coin `CoinType` from any addr, this function does not check the Coin `frozen` attribute
    /// This function is only called by the `CoinType` module, for the developer to extend custom withdraw logic
    public fun withdraw_extend<CoinType: key>(
        ctx: &mut StorageContext,
        addr: address,
        amount: u256,
    ): Coin<CoinType> {
        withdraw_internal<CoinType>(ctx, addr, amount) 
    }

    #[private_generics(CoinType)]
    /// Deposit the coin into the recipient's account and emit an event.
    /// This function is only called by the `CoinType` module, for the developer to extend custom deposit logic
    public fun deposit_extend<CoinType: key>(ctx: &mut StorageContext, addr: address, coin: Coin<CoinType>) {
        deposit_internal(ctx, addr, coin);
    }

    #[private_generics(CoinType)]
    /// Transfer `amount` of coins `CoinType` from `from` to `to`.
    /// This function is only called by the `CoinType` module, for the developer to extend custom transfer logic
    public fun transfer_extend<CoinType: key>(
        ctx: &mut StorageContext,
        from: address,
        to: address,
        amount: u256,
    ) {
        transfer_internal<CoinType>(ctx, from, to, amount);
    }

    #[private_generics(CoinType)]
    /// Burn `coin`
    /// This function is only called by the `CoinType` module, for the developer to extend custom burn logic
    public fun burn_extend<CoinType: key>(
        ctx: &mut StorageContext,
        coin: Coin<CoinType>,
    ) {
        burn_internal(ctx, coin) 
    }

    #[private_generics(CoinType)]
    /// Freeze a CoinStore to prevent transfers
    public fun freeze_coin_store_extend<CoinType: key>(
        ctx: &mut StorageContext,
        addr: address,
    ) {
        ensure_coin_store<CoinType>(ctx, addr);
        let coin_store = borrow_mut_coin_store<CoinType>(ctx, addr);
        coin_store.frozen = true;
    }

    #[private_generics(CoinType)]
    /// Unfreeze a CoinStore to allow transfers
    public fun unfreeze_coin_store_extend<CoinType: key>(
        ctx: &mut StorageContext,
        addr: address,
    ) {
        ensure_coin_store<CoinType>(ctx, addr);
        let coin_store = borrow_mut_coin_store<CoinType>(ctx, addr);
        coin_store.frozen = false;
    }

    //
    // Entry functions
    //

    /// Creating a resource that stores balance of `CoinType` on user's account.
    /// Required if user wants to start accepting deposits of `CoinType` in his account.
    public entry fun accept_coin_entry<CoinType: key>(ctx: &mut StorageContext, account: &signer) {
        do_accept_coin<CoinType>(ctx, account)
    }

    /// Enable account's auto-accept-coin feature.
    /// The script function is reenterable.
    public entry fun enable_auto_accept_coin_entry(ctx: &mut StorageContext, account: &signer) {
        set_auto_accept_coin(ctx, account, true)
    }

    /// Disable account's auto-accept-coin feature.
    /// The script function is reenterable.
    public entry fun disable_auto_accept_coin_entry(ctx: &mut StorageContext, account: &signer) {
        set_auto_accept_coin(ctx, account, false);
    }

    /// Transfer `amount` of coins `CoinType` from `from` to `to`.
    /// This public entry function requires the `CoinType` to have `key` and `store` abilities.
    public entry fun transfer_entry<CoinType: key + store>(
        ctx: &mut StorageContext,
        from: &signer,
        to: address,
        amount: u256,
    ) {
        transfer<CoinType>(ctx, from, to, amount)
    }

}
