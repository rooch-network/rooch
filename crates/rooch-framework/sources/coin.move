/// This module provides the foundation for typesafe Coins.
module rooch_framework::coin {
    use std::string;
    use std::error;
    use moveos_std::table;
    use moveos_std::table::Table;
    use moveos_std::type_table::TypeTable;
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

    /// CoinStore is frozen. Coins cannot be deposited or withdrawn
    const ErrorAccountWithCoinFrozen: u64 = 8;

    /// Account hasn't accept `CoinType`
    const ErrorAccountNotAcceptCoin: u64 = 9;

    /// account has no capabilities (burn/mint).
    const ErrorNoCapabilities: u64 = 12;

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

    /// Capability required to mint coins.
    struct MintCapability<phantom CoinType> has key, copy, store {}

    /// Capability required to freeze a coin store.
    struct FreezeCapability<phantom CoinType> has key, copy, store {}

    /// Capability required to burn coins.
    struct BurnCapability<phantom CoinType> has key, copy, store {}

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
    fun coin_address<CoinType>(): address {
        let type_info = type_info::type_of<CoinType>();
        type_info::account_address(&type_info)
    }


    /// Returns the balance of `addr` for provided `CoinType`.
    public fun balance<CoinType>(ctx: &StorageContext, addr: address): u256 {
        if (exist_coin_store<CoinType>(ctx, addr)) {
            borrow_coin_store<CoinType>(ctx, addr).coin.value
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

    //
    // Helper functions
    //

    fun borrow_coin_info<CoinType>(ctx: &StorageContext): &CoinInfo<CoinType> {
        let coin_infos = account_storage::global_borrow<CoinInfos>(ctx, @rooch_framework);
        type_table::borrow<CoinInfo<CoinType>>(&coin_infos.coin_infos)
    }

    fun borrow_mut_coin_info<CoinType>(ctx: &mut StorageContext): &mut CoinInfo<CoinType> {
        let coin_infos = account_storage::global_borrow_mut<CoinInfos>(ctx, @rooch_framework);
        type_table::borrow_mut<CoinInfo<CoinType>>(&mut coin_infos.coin_infos)
    }

    fun exist_auto_accept_token(ctx: &StorageContext, addr: address): bool {
        let auto_accept_coins = account_storage::global_borrow<AutoAcceptCoins>(ctx, @rooch_framework);
        table::contains<address, bool>(&auto_accept_coins.auto_accept_coins, addr)
    }

    fun borrow_coin_store<CoinType>(ctx: &StorageContext, addr: address): &CoinStore<CoinType> {
        let coin_stores = account_storage::global_borrow<CoinStores>(ctx, addr);
        type_table::borrow<CoinStore<CoinType>>(&coin_stores.coin_stores)
    }

    fun borrow_mut_coin_store<CoinType>(ctx: &mut StorageContext, addr: address): &mut CoinStore<CoinType> {
        let coin_stores = account_storage::global_borrow_mut<CoinStores>(ctx, addr);
        type_table::borrow_mut<CoinStore<CoinType>>(&mut coin_stores.coin_stores)
    }

    fun ensure_coin_store<CoinType>(ctx: &mut StorageContext, addr: address) {
        if (!exist_coin_store<CoinType>(ctx, addr) && can_auto_accept_coin(ctx, addr)) {
            inner_new_coin_store<CoinType>(ctx, addr)
        }
    }

    fun ensure_coin_store_pass_auto_accept_flag<CoinType>(ctx: &mut StorageContext, addr: address) {
        if (!exist_coin_store<CoinType>(ctx, addr)) {
            inner_new_coin_store<CoinType>(ctx, addr)
        }
    }

    fun inner_new_coin_store<CoinType>(ctx: &mut StorageContext, addr: address) {
        let coin_stores = account_storage::global_borrow_mut<CoinStores>(ctx, addr);
        type_table::add<CoinStore<CoinType>>(&mut coin_stores.coin_stores, CoinStore<CoinType> {
            coin: Coin { value: 0 },
            frozen: false,
        })
    }

    //
    // Public functions
    //

    /// Return whether the account at `addr` accept `Coin` type coins
    public fun is_account_accept_coin<CoinType>(ctx: &StorageContext, addr: address): bool {
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
    public fun do_accept_coin<CoinType>(ctx: &mut StorageContext, account: &signer) {
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
    public fun withdraw<CoinType>(
        ctx: &mut StorageContext,
        account: &signer,
        amount: u256,
    ): Coin<CoinType> {
        let addr = signer::address_of(account);
        withdraw_interal<CoinType>(ctx, addr, amount) 
    }

    #[private_generics(CoinType)]
    /// Withdraw specifed `amount` of coin `CoinType` from any addr
    /// This function is only called by the `CoinType` module, for the developer to extend custom withdraw logic
    public fun withdraw_extend<CoinType>(
        ctx: &mut StorageContext,
        addr: address,
        amount: u256,
    ): Coin<CoinType> {
        withdraw_interal<CoinType>(ctx, addr, amount) 
    }

    fun withdraw_interal<CoinType>(
        ctx: &mut StorageContext,
        addr: address,
        amount: u256,
    ): Coin<CoinType> {
        assert!(
            is_account_accept_coin<CoinType>(ctx, addr),
            error::not_found(ErrorAccountNotAcceptCoin),
        );

        assert!(
            !is_coin_store_frozen<CoinType>(ctx, addr),
            error::permission_denied(ErrorAccountWithCoinFrozen),
        );

        ensure_coin_store<CoinType>(ctx, addr);
        let coin_type_info = type_info::type_of<CoinType>();
        event::emit<WithdrawEvent>(ctx, WithdrawEvent {
            coin_type_info,
            amount,
        });

        extract_coin(ctx, addr, amount)
    }

    /// Deposit the coin balance into the recipient's account and emit an event.
    public fun deposit<CoinType>(ctx: &mut StorageContext, addr: address, coin: Coin<CoinType>) {
        assert!(
            is_account_accept_coin<CoinType>(ctx, addr),
            error::not_found(ErrorAccountNotAcceptCoin),
        );

        assert!(
            !is_coin_store_frozen<CoinType>(ctx, addr),
            error::permission_denied(ErrorAccountWithCoinFrozen),
        );

        ensure_coin_store<CoinType>(ctx, addr);
        let coin_type_info = type_info::type_of<CoinType>();
        event::emit<DepositEvent>(ctx, DepositEvent {
            coin_type_info,
            amount: value(&coin),
        });

        merge_coin(ctx, addr, coin);
    }

    /// Transfer `amount` of coins `CoinType` from `from` to `to`.
    public fun transfer<CoinType>(
        ctx: &mut StorageContext,
        from: &signer,
        to: address,
        amount: u256,
    ) {
        let coin = withdraw<CoinType>(ctx, from, amount);
        deposit(ctx, to, coin);
    }

    /// Burn `coin` with capability.
    /// The capability `_cap` should be passed as a reference to `BurnCapability<CoinType>`.
    public fun burn<CoinType>(
        ctx: &mut StorageContext,
        coin: Coin<CoinType>,
        _cap: &BurnCapability<CoinType>,
    ) {
        burn_internal(ctx, coin) 
    }

    #[private_generics(CoinType)]
    /// Burn `coin`
    /// This function is only called by the `CoinType` module, for the developer to extend custom burn logic
    public fun burn_extend<CoinType>(
        ctx: &mut StorageContext,
        coin: Coin<CoinType>,
    ) {
        burn_internal(ctx, coin) 
    }

    fun burn_internal<CoinType>(
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

    //TODO This function can be replaced by `withdraw_extend` and `burn`, we can remove it.
    /// Burn `coin` from the specified `account` with capability.
    /// The capability `burn_cap` should be passed as a reference to `BurnCapability<CoinType>`.
    /// This function shouldn't fail as it's called as part of transaction fee burning.
    public fun burn_from<CoinType>(
        ctx: &mut StorageContext,
        addr: address,
        amount: u256,
        burn_cap: &BurnCapability<CoinType>,
    ) {
        let coin_store = borrow_mut_coin_store<CoinType>(ctx, addr);
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
        ensure_coin_store<CoinType>(ctx, addr);
        let coin_store = borrow_mut_coin_store<CoinType>(ctx, addr);
        coin_store.frozen = true;
    }

    /// Unfreeze a CoinStore to allow transfers
    public fun unfreeze_coin_store<CoinType>(
        ctx: &mut StorageContext,
        addr: address,
        _freeze_cap: &FreezeCapability<CoinType>,
    ) {
        ensure_coin_store<CoinType>(ctx, addr);
        let coin_store = borrow_mut_coin_store<CoinType>(ctx, addr);
        coin_store.frozen = false;
    }

    // #[private_generics(CoinType)]
    /// Creates a new Coin with given `CoinType` and returns minting/freezing/burning capabilities.
    /// The given signer also becomes the account hosting the information about the coin
    /// (name, supply, etc.).
    public fun initialize<CoinType>(
        ctx: &mut StorageContext,
        account: &signer,
        // addr: address,
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
        mint_internal<CoinType>(ctx, amount)
    }

    #[private_generics(CoinType)]
    /// Mint new `Coin`, this function is only called by the `CoinType` module, for the developer to extend custom mint logic
    public fun mint_extend<CoinType>(ctx: &mut StorageContext,amount: u256) : Coin<CoinType> {
        mint_internal<CoinType>(ctx, amount)
    }

    fun mint_internal<CoinType>(ctx: &mut StorageContext,
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

    public fun exist_coin_store<CoinType>(ctx: &StorageContext, addr: address): bool {
        if (account_storage::global_exists<CoinStores>(ctx, addr)) {
            let coin_stores = account_storage::global_borrow<CoinStores>(ctx, addr);
            type_table::contains<CoinStore<CoinType>>(&coin_stores.coin_stores)
        } else {
            false
        }
    }

    public fun is_coin_store_frozen<CoinType>(ctx: &StorageContext, addr: address): bool {
        if (exist_coin_store<CoinType>(ctx, addr)) {
            borrow_coin_store<CoinType>(ctx, addr).frozen
        } else {
            false
        }
    }

    fun extract_coin<CoinType>(ctx: &mut StorageContext, addr: address, amount: u256): Coin<CoinType> {
        let coin_store = borrow_mut_coin_store<CoinType>(ctx, addr);
        extract<CoinType>(&mut coin_store.coin, amount)
    }

    fun merge_coin<CoinType>(ctx: &mut StorageContext, addr: address, coin: Coin<CoinType>) {
        let coin_store = borrow_mut_coin_store<CoinType>(ctx, addr);
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

    //
    // Data structures
    //

    /// Capabilities resource storing mint and burn capabilities.
    /// The resource is stored on the account that initialized coin `CoinType`.
    struct Capabilities<phantom CoinType> has key {
        burn_cap: BurnCapability<CoinType>,
        freeze_cap: FreezeCapability<CoinType>,
        mint_cap: MintCapability<CoinType>,
    }

    //
    // entry functions
    //
    // CoinEntry is built to make a simple walkthrough of the Coins module.
    // It contains scripts you will need to initialize, mint, burn, transfer coins.

    /// Initialize new coin `CoinType` in Rooch Blockchain.
    /// Mint and Burn Capabilities will be stored under `account` in `Capabilities` resource.
    /// A developer can create his own coin and care less about mint and burn capabilities
    public entry fun initialize_entry<CoinType>(
        ctx: &mut StorageContext,
        account: &signer,
        name: vector<u8>,
        symbol: vector<u8>,
        decimals: u8,
    ) {
        // let addr = signer::address_of(account);
        let (burn_cap, freeze_cap, mint_cap) = initialize<CoinType>(
            ctx,
            account,
            string::utf8(name),
            string::utf8(symbol),
            decimals,
        );

        account_storage::global_move_to(ctx, account, Capabilities<CoinType> {
            burn_cap,
            freeze_cap,
            mint_cap
        });
    }

    /// Create new coins `CoinType` and deposit them into dst_addr's account.
    public entry fun mint_entry<CoinType>(
        ctx: &mut StorageContext,
        account: &signer,
        dst_addr: address,
        amount: u256,
    ) {
        let account_addr = signer::address_of(account);

        assert!(
            account_storage::global_exists<Capabilities<CoinType>>(ctx, account_addr),
            error::not_found(ErrorNoCapabilities),
        );

        let cap = account_storage::global_move_from<Capabilities<CoinType>>(ctx, account_addr);
        // let cap = account_storage::global_borrow<Capabilities<CoinType>>(ctx, account_addr);
        let coins_minted = mint(ctx, amount, &cap.mint_cap);
        deposit(ctx, dst_addr, coins_minted);
        account_storage::global_move_to<Capabilities<CoinType>>(ctx, account, cap)
    }

    /// Withdraw an `amount` of coin `CoinType` from `account` and burn it.
    public entry fun burn_entry<CoinType>(
        ctx: &mut StorageContext,
        account: &signer,
        amount: u256,
    ) {
        let account_addr = signer::address_of(account);

        assert!(
            account_storage::global_exists<Capabilities<CoinType>>(ctx, account_addr),
            error::not_found(ErrorNoCapabilities),
        );

        // let cap = account_storage::global_borrow<Capabilities<CoinType>>(ctx, account_addr);
        let cap = account_storage::global_move_from<Capabilities<CoinType>>(ctx, account_addr);
        let to_burn = withdraw<CoinType>(ctx, account, amount);
        burn<CoinType>(ctx, to_burn, &cap.burn_cap);
        account_storage::global_move_to<Capabilities<CoinType>>(ctx, account, cap);
    }

    /// Creating a resource that stores balance of `CoinType` on user's account.
    /// Required if user wants to start accepting deposits of `CoinType` in his account.
    public entry fun accept_coin_entry<CoinType>(ctx: &mut StorageContext, account: &signer) {
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
    public entry fun transfer_entry<CoinType>(
        ctx: &mut StorageContext,
        from: &signer,
        to: address,
        amount: u256,
    ) {
        transfer<CoinType>(ctx, from, to, amount)
    }

    /// Freeze a CoinStore to prevent transfers
    public entry fun freeze_coin_store_entry<CoinType>(
        ctx: &mut StorageContext,
        account: &signer
    ) {
        let account_addr = signer::address_of(account);
        assert!(
            account_storage::global_exists<Capabilities<CoinType>>(ctx, account_addr),
            error::not_found(ErrorNoCapabilities),
        );
        // let cap = account_storage::global_borrow<Capabilities<CoinType>>(ctx, account_addr);
        let cap = account_storage::global_move_from<Capabilities<CoinType>>(ctx, account_addr);
        freeze_coin_store(ctx, account_addr, &cap.freeze_cap);
        account_storage::global_move_to<Capabilities<CoinType>>(ctx, account, cap)
    }

    /// Unfreeze a CoinStore to allow transfers
    public entry fun unfreeze_coin_store_entry<CoinType>(
        ctx: &mut StorageContext,
        account: &signer
    ) {
        let account_addr = signer::address_of(account);
        assert!(
            account_storage::global_exists<Capabilities<CoinType>>(ctx, account_addr),
            error::not_found(ErrorNoCapabilities),
        );
        let cap = account_storage::global_move_from<Capabilities<CoinType>>(ctx, account_addr);
        // let cap = account_storage::global_borrow<Capabilities<CoinType>>(ctx, account_addr);
        unfreeze_coin_store(ctx, account_addr, &cap.freeze_cap);
        account_storage::global_move_to<Capabilities<CoinType>>(ctx, account, cap)
    }

}
