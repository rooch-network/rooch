/// This module provides the foundation for typesafe Coins.
module rooch_framework::coin {
    use std::string;
    use std::error;
    use std::option;
    use std::option::Option;
    use moveos_std::object_id::ObjectID;
    use moveos_std::table;
    use moveos_std::table::Table;
    use moveos_std::account_storage;
    use moveos_std::context::{Self, Context};
    use moveos_std::event;
    use moveos_std::type_info::{Self, type_of};
    use moveos_std::signer;
    use moveos_std::object_ref::{Self, ObjectRef};

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

    /// Global CoinInfos should exist
    const ErrorCoinInfosNotFound: u64 = 9;

    /// The CoinType parameter and CoinType in CoinStore do not match
    const ErrorCoinTypeAndStoreMismatch: u64 = 10;


    //
    // Constants
    //

    const MAX_COIN_NAME_LENGTH: u64 = 32;
    const MAX_COIN_SYMBOL_LENGTH: u64 = 10;

    /// Core data structures

    /// Main structure representing a coin.
    /// Note the `CoinType` must have `key` ability.
    /// if the `CoinType` has `store` ability, the `Coin` is a public coin, the user can operate it directly by coin module's function.
    /// Otherwise, the `Coin` is a private coin, the user can only operate it by `CoinType` module's function.
    /// The Coin has no ability, it is a hot potato type, only can handle by Coin module.
    struct Coin<phantom CoinType : key> {
        /// Amount of coin this address has.
        /// Following the ERC20 standard, both asset balance and supply are expressed in u256
        value: u256,
    }

    /// The Balance resource that stores the balance of a specific coin type.
    struct Balance has store {
        value: u256,
    }

    /// Maximum possible aggregatable coin value.
    const MAX_U64: u128 = 18446744073709551615;

    // /// A holder of a specific coin types.
    // /// These are kept in a single resource to ensure locality of data.
    struct CoinStore has key {
        coin_type: string::String,
        balance: Balance,
        frozen: bool,
    }

    /// Maximum possible coin supply.
    const MAX_U128: u128 = 340282366920938463463374607431768211455;

    const MAX_U256: u256 = 115792089237316195423570985008687907853269984665640564039457584007913129639935;

    /// Information about a specific coin type. Stored in the global CoinInfos table.
    struct CoinInfo has store {
        /// Type of the coin: `address::my_module::XCoin`, same as `moveos_std::type_info::type_name<CoinType>()`.
        /// The name and symbol can repeat across different coin types, but the coin type must be unique.
        coin_type: string::String,
        /// Name of the coin.
        name: string::String,
        /// Symbol of the coin, usually a shorter version of the name.
        /// For example, Singapore Dollar is SGD.
        symbol: string::String,
        /// Number of decimals used to get its user representation.
        /// For example, if `decimals` equals `2`, a balance of `505` coins should
        /// be displayed to a user as `5.05` (`505 / 10 ** 2`).
        decimals: u8,
        /// The total value for the coin represented by coin type. Mutable.
        supply: u256,
    }

    /// A resource that holds the CoinInfo for all accounts.
    struct CoinInfos has key {
        coin_infos: Table<string::String, CoinInfo>,
    }

    /// A resource that holds the AutoAcceptCoin config for all accounts.
    /// The main scenario is that the user can actively turn off the AutoAcceptCoin setting to avoid automatically receiving Coin
    struct AutoAcceptCoins has key {
        auto_accept_coins: Table<address, bool>,
    }

    /// A resource that holds all the ObjectRef<CoinStore> for account.
    /// Default Deposit Coin no longer depends on accept coin
    struct CoinStores has key {
        coin_stores: Table<string::String, ObjectRef<CoinStore>>,
    }

    /// Event emitted when some amount of a coin is deposited into an account.
    struct DepositEvent has drop, store {
        /// The type of the coin that was sent
        coin_type: string::String,
        amount: u256,
    }

    /// Event emitted when some amount of a coin is withdrawn from an account.
    struct WithdrawEvent has drop, store {
        /// The type of the coin that was sent
        coin_type: string::String,
        amount: u256,
    }

    /// Event for auto accept coin set
    struct AcceptCoinEvent has drop, store {
        /// auto accept coin config
        enable: bool,
    }

    /// Event emitted when coin minted.
    struct MintEvent has drop, store {
        /// The type of coin that was minted
        coin_type: string::String,
        /// coins added to the system
        amount: u256,
    }

    /// Event emitted when coin burned.
    struct BurnEvent has drop, store {
         /// The type of coin that was burned
        coin_type: string::String,
        /// coins removed from the system
        amount: u256,
    }

    public(friend) fun genesis_init(ctx: &mut Context, genesis_account: &signer) {
        let coin_infos = CoinInfos {
            coin_infos: table::new(ctx),
        };
        account_storage::global_move_to(ctx, genesis_account, coin_infos);

        let auto_accepted_coins = AutoAcceptCoins {
            auto_accept_coins: table::new<address, bool>(ctx),
        };
        account_storage::global_move_to(ctx, genesis_account, auto_accepted_coins);
    }

    public(friend) fun init_account_coin_store(ctx: &mut Context, account: &signer){
        let coin_stores = CoinStores {
            coin_stores: table::new<string::String, ObjectRef<CoinStore>>(ctx),
        };
        account_storage::global_move_to(ctx, account, coin_stores);
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
    public fun balance<CoinType: key>(ctx: &Context, addr: address): u256 {
        if (exist_account_coin_store<CoinType>(ctx, addr)) {
            borrow_account_coin_store<CoinType>(ctx, addr).balance.value
        } else {
            0u256
        }
    }

    /// Returns `true` if the type `CoinType` is an registered coin.
    public fun is_registered<CoinType: key>(ctx: &Context): bool {
        if (account_storage::global_exists<CoinInfos>(ctx, @rooch_framework)) {
            let coin_infos = account_storage::global_borrow<CoinInfos>(ctx, @rooch_framework);
            let coin_type = type_info::type_name<CoinType>();
            table::contains(&coin_infos.coin_infos, coin_type)
        } else {
            false
        }
    }

    /// Returns the name of the coin.
    public fun name<CoinType: key>(ctx: &Context): string::String {
        borrow_coin_info<CoinType>(ctx).name
    }

    /// Returns the symbol of the coin, usually a shorter version of the name.
    public fun symbol<CoinType: key>(ctx: &Context): string::String {
        borrow_coin_info<CoinType>(ctx).symbol
    }

    /// Returns the number of decimals used to get its user representation.
    /// For example, if `decimals` equals `2`, a balance of `505` coins should
    /// be displayed to a user as `5.05` (`505 / 10 ** 2`).
    public fun decimals<CoinType: key>(ctx: &Context): u8 {
        borrow_coin_info<CoinType>(ctx).decimals
    }


    /// Returns the amount of coin in existence.
    public fun supply<CoinType: key>(ctx: &Context): u256 {
        borrow_coin_info<CoinType>(ctx).supply
    }

    /// Return true if the type `CoinType1` is same with `CoinType2`
    public fun is_same_coin<CoinType1, CoinType2>(): bool {
        return type_of<CoinType1>() == type_of<CoinType2>()
    }

    /// Return the account CoinStore object id for addr
    public fun coin_store_id<CoinType: key>(ctx: &Context, addr: address): Option<ObjectID> {
        if (exist_account_coin_store<CoinType>(ctx, addr)) {
            let coin_stores = account_storage::global_borrow<CoinStores>(ctx, addr);
            let coin_type = type_info::type_name<CoinType>();
            let coin_store_ref = table::borrow(&coin_stores.coin_stores, coin_type);
            option::some(object_ref::id(coin_store_ref))
        } else {
            option::none<ObjectID>()
        }
    }

    /// Return CoinStores table handle for addr
    public fun coin_stores_handle(ctx: &Context, addr: address): Option<ObjectID> {
        if (account_storage::global_exists<CoinStores>(ctx, addr))
        {
            let coin_stores = account_storage::global_borrow<CoinStores>(ctx, addr);
            option::some(*table::handle(&coin_stores.coin_stores))
        } else {
            option::none<ObjectID>()
        }
    }

    /// Return CoinInfos table handle
    public fun coin_infos_handle(ctx: &Context): ObjectID {
        // coin info ensured via the Genesis transaction, so it should always exist
        assert!(account_storage::global_exists<CoinInfos>(ctx, @rooch_framework), error::invalid_argument(ErrorCoinInfosNotFound));
        let coin_infos = account_storage::global_borrow<CoinInfos>(ctx, @rooch_framework);
        *table::handle(&coin_infos.coin_infos)
    }

    //
    // Helper functions
    //

    fun borrow_coin_info<CoinType: key>(ctx: &Context): &CoinInfo {
        let coin_infos = account_storage::global_borrow<CoinInfos>(ctx, @rooch_framework);
        let coin_type = type_info::type_name<CoinType>();
        check_coin_info_registered(coin_infos, coin_type);
        table::borrow(&coin_infos.coin_infos, coin_type)
    }

    fun borrow_mut_coin_info<CoinType: key>(ctx: &mut Context): &mut CoinInfo {
        let coin_infos = account_storage::global_borrow_mut<CoinInfos>(ctx, @rooch_framework);
        let coin_type = type_info::type_name<CoinType>();
        check_coin_info_registered(coin_infos, coin_type);
        table::borrow_mut(&mut coin_infos.coin_infos, coin_type)
    }

    fun borrow_account_coin_store<CoinType: key>(ctx: &Context, addr: address): &CoinStore{
        let coin_stores = account_storage::global_borrow<CoinStores>(ctx, addr);
        let coin_type = type_info::type_name<CoinType>();
        let ref = table::borrow(&coin_stores.coin_stores, coin_type);
        object_ref::borrow(ref)
    }

    fun borrow_mut_account_coin_store<CoinType: key>(ctx: &mut Context, addr: address): &mut CoinStore{
        let coin_stores = account_storage::global_borrow_mut<CoinStores>(ctx, addr);
        let coin_type = type_info::type_name<CoinType>();
        let ref = table::borrow_mut(&mut coin_stores.coin_stores, coin_type);
        object_ref::borrow_mut(ref)
    }

    fun ensure_coin_store<CoinType: key>(ctx: &mut Context, addr: address) {
        if (!exist_account_coin_store<CoinType>(ctx, addr) && can_auto_accept_coin(ctx, addr)) {
            create_account_coin_store<CoinType>(ctx, addr)
        }
    }

    fun ensure_coin_store_bypass_auto_accept_flag<CoinType: key>(ctx: &mut Context, addr: address) {
        if (!exist_account_coin_store<CoinType>(ctx, addr)) {
            create_account_coin_store<CoinType>(ctx, addr)
        }
    }

    /// Create a new CoinStore Object for `CoinType` and return the ObjectRef
    public fun create_coin_store<CoinType: key>(ctx: &mut Context): ObjectRef<CoinStore>{
        //TODO check the CoinType is registered
        let coin_store_object = context::new_object(ctx, CoinStore{
            coin_type: type_info::type_name<CoinType>(),
            balance: Balance { value: 0 },
            frozen: false,
        });
        let ref = object_ref::new(&coin_store_object);
        context::add_object(ctx, coin_store_object);
        ref
    }

    fun create_account_coin_store<CoinType: key>(ctx: &mut Context, addr: address) {
        let coin_store_ref = create_coin_store<CoinType>(ctx);
        let coin_stores = account_storage::global_borrow_mut<CoinStores>(ctx, addr);
        let coin_type = type_info::type_name<CoinType>();
        table::add(&mut coin_stores.coin_stores, coin_type, coin_store_ref);
    }

    fun extract_coin<CoinType: key>(ctx: &mut Context, addr: address, amount: u256): Coin<CoinType> {
        let coin_store = borrow_mut_account_coin_store<CoinType>(ctx, addr);
        extract_from_store<CoinType>(coin_store, amount)
    }

    fun merge_coin<CoinType: key>(ctx: &mut Context, addr: address, coin: Coin<CoinType>) {
        let coin_store = borrow_mut_account_coin_store<CoinType>(ctx, addr);
        merge_to_store<CoinType>(coin_store, coin)
    }

    fun check_account_coin_store_not_frozen<CoinType: key>(ctx: &Context, addr: address) {
        assert!(
            !is_account_coin_store_frozen<CoinType>(ctx, addr),
            error::permission_denied(ErrorAccountWithCoinFrozen),
        );
    }

    fun check_coin_info_registered(coin_infos: &CoinInfos, coin_type: string::String) {
        assert!(
            table::contains(&coin_infos.coin_infos, coin_type),
            error::not_found(ErrorCoinInfoNotRegistered),
        );
    }

    //
    // Internal functions
    //

    fun mint_internal<CoinType: key>(
        ctx: &mut Context,
        amount: u256
    ): Coin<CoinType>{
        let coin_info = borrow_mut_coin_info<CoinType>(ctx);
        coin_info.supply = coin_info.supply + amount;
        let coin_type = type_info::type_name<CoinType>();
        event::emit<MintEvent>(ctx, MintEvent {
            coin_type,
            amount,
        });
        Coin<CoinType> { value: amount }
    }

    fun withdraw_internal<CoinType: key>(
        ctx: &mut Context,
        addr: address,
        amount: u256,
    ): Coin<CoinType> {       
        ensure_coin_store<CoinType>(ctx, addr);
        let coin_type = type_info::type_name<CoinType>();
        event::emit<WithdrawEvent>(ctx, WithdrawEvent {
            coin_type,
            amount,
        });

        extract_coin(ctx, addr, amount)
    }

    fun deposit_internal<CoinType: key>(ctx: &mut Context, addr: address, coin: Coin<CoinType>) {
        assert!(
            is_account_accept_coin<CoinType>(ctx, addr),
            error::not_found(ErrorAccountNotAcceptCoin),
        );

        ensure_coin_store<CoinType>(ctx, addr);
        let coin_type = type_info::type_name<CoinType>();
        event::emit<DepositEvent>(ctx, DepositEvent {
            coin_type,
            amount: value(&coin),
        });

        merge_coin(ctx, addr, coin);
    }

    fun transfer_internal<CoinType: key>(
        ctx: &mut Context,
        from: address,
        to: address,
        amount: u256,
    ) {
        let coin = withdraw_internal<CoinType>(ctx, from, amount);
        deposit_internal(ctx, to, coin);
    }

    fun burn_internal<CoinType: key>(
        ctx: &mut Context,
        coin: Coin<CoinType>,
    ) {
        let Coin { value: amount } = coin;

        let coin_type = type_info::type_name<CoinType>();
        let coin_info = borrow_mut_coin_info<CoinType>(ctx);
        coin_info.supply = coin_info.supply - amount;
        event::emit<BurnEvent>(ctx, BurnEvent {
            coin_type,
            amount,
        });
    }


    //
    // Public functions
    //

    /// Return whether the account at `addr` accept `Coin` type coins
    public fun is_account_accept_coin<CoinType: key>(ctx: &Context, addr: address): bool {
        if (can_auto_accept_coin(ctx, addr)) {
            true
        } else {
            exist_account_coin_store<CoinType>(ctx, addr)
        }
    }

    /// Check whether the address can auto accept coin.
    /// Default is true if absent
    public fun can_auto_accept_coin(ctx: &Context, addr: address): bool {
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
    public fun do_accept_coin<CoinType: key>(ctx: &mut Context, account: &signer) {
        let addr = signer::address_of(account);
        ensure_coin_store_bypass_auto_accept_flag<CoinType>(ctx, addr);
    }

    /// Configure whether auto-accept coins.
    public fun set_auto_accept_coin(ctx: &mut Context, account: &signer, enable: bool)  {
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
        ctx: &mut Context,
        account: &signer,
        amount: u256,
    ): Coin<CoinType> {
        let addr = signer::address_of(account);
        // the coin `frozen` only affect user withdraw, does not affect `withdraw_extend`. 
        check_account_coin_store_not_frozen<CoinType>(ctx, addr);
        withdraw_internal<CoinType>(ctx, addr, amount) 
    }

    /// Deposit the coin into the recipient's account and emit an event.
    /// This public entry function requires the `CoinType` to have `key` and `store` abilities.
    public fun deposit<CoinType: key + store>(ctx: &mut Context, addr: address, coin: Coin<CoinType>) {
        check_account_coin_store_not_frozen<CoinType>(ctx, addr);
        deposit_internal(ctx, addr, coin);
    }

    public fun deposit_to_store<CoinType: key + store>(coin_store: &mut CoinStore, coin: Coin<CoinType>) {
        merge_to_store<CoinType>(coin_store, coin);
    }

    /// Transfer `amount` of coins `CoinType` from `from` to `to`.
    /// Any account and module can call this function to transfer coins, the `CoinType` must have `key` and `store` abilities.
    public fun transfer<CoinType: key + store>(
        ctx: &mut Context,
        from: &signer,
        to: address,
        amount: u256,
    ) {
        let from_addr = signer::address_of(from);
        check_account_coin_store_not_frozen<CoinType>(ctx, from_addr);
        check_account_coin_store_not_frozen<CoinType>(ctx, to);
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

    /// Extracts `amount` Coin from the balance of the passed-in `coin_store`
    public fun extract_from_store<CoinType: key>(coin_store: &mut CoinStore, amount: u256): Coin<CoinType> {
        assert!(coin_store.balance.value >= amount, error::invalid_argument(ErrorInSufficientBalance));
        let coin_type = type_info::type_name<CoinType>();
        assert!(coin_store.coin_type == coin_type, error::invalid_argument(ErrorCoinTypeAndStoreMismatch));
        coin_store.balance.value = coin_store.balance.value - amount;
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

    /// "Merges" the given coins to the balance of the passed-in `coin_store`.
    public fun merge_to_store<CoinType: key>(coin_store: &mut CoinStore, source_coin: Coin<CoinType>) {
        let coin_type = type_info::type_name<CoinType>();
        assert!(coin_store.coin_type == coin_type, error::invalid_argument(ErrorCoinTypeAndStoreMismatch));
        let Coin { value } = source_coin;
        coin_store.balance.value = coin_store.balance.value + value;
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

    public fun exist_account_coin_store<CoinType: key>(ctx: &Context, addr: address): bool {
        if (account_storage::global_exists<CoinStores>(ctx, addr)) {
            let coin_stores = account_storage::global_borrow<CoinStores>(ctx, addr);
            let coin_type = type_info::type_name<CoinType>();
            table::contains(&coin_stores.coin_stores, coin_type)
        } else {
            false
        }
    }

    public fun is_account_coin_store_frozen<CoinType: key>(ctx: &Context, addr: address): bool {
        if (exist_account_coin_store<CoinType>(ctx, addr)) {
            borrow_account_coin_store<CoinType>(ctx, addr).frozen
        } else {
            false
        }
    }

    public fun is_coin_store_frozen<CoinType: key>(coin_store_ref: &ObjectRef<CoinStore>): bool {
        if (object_ref::contains(coin_store_ref)) {
            object_ref::borrow(coin_store_ref).frozen
        } else {
            //TODO if the coin store is not exist, should we return true or false?
            false
        }
    }

    //
    // Extend functions
    //

    #[private_generics(CoinType)]
    /// Creates a new Coin with given `CoinType`
    /// This function is protected by `private_generics`, so it can only be called by the `CoinType` module.
    public fun register_extend<CoinType: key>(
        ctx: &mut Context,
        name: string::String,
        symbol: string::String,
        decimals: u8,
    ){
        
        let coin_infos = account_storage::global_borrow_mut<CoinInfos>(ctx, @rooch_framework);
        let coin_type = type_info::type_name<CoinType>();
        
        assert!(
            !table::contains(&coin_infos.coin_infos, coin_type),
            error::already_exists(ErrorCoinInfoAlreadyRegistered),
        ); 

        assert!(string::length(&name) <= MAX_COIN_NAME_LENGTH, error::invalid_argument(ErrorCoinNameTooLong));
        assert!(string::length(&symbol) <= MAX_COIN_SYMBOL_LENGTH, error::invalid_argument(ErrorCoinSymbolTooLong));

        let coin_info = CoinInfo {
            coin_type,
            name,
            symbol,
            decimals,
            supply: 0u256,
        };
        table::add(&mut coin_infos.coin_infos, coin_type, coin_info);
    }

    #[private_generics(CoinType)]
    /// Mint new `Coin`, this function is only called by the `CoinType` module, for the developer to extend custom mint logic
    public fun mint_extend<CoinType: key>(ctx: &mut Context,amount: u256) : Coin<CoinType> {
        mint_internal<CoinType>(ctx, amount)
    }

    #[private_generics(CoinType)]
    /// Withdraw specifed `amount` of coin `CoinType` from any addr, this function does not check the Coin `frozen` attribute
    /// This function is only called by the `CoinType` module, for the developer to extend custom withdraw logic
    public fun withdraw_extend<CoinType: key>(
        ctx: &mut Context,
        addr: address,
        amount: u256,
    ): Coin<CoinType> {
        withdraw_internal<CoinType>(ctx, addr, amount) 
    }

    #[private_generics(CoinType)]
    /// Deposit the coin into the recipient's account and emit an event.
    /// This function is only called by the `CoinType` module, for the developer to extend custom deposit logic
    public fun deposit_extend<CoinType: key>(ctx: &mut Context, addr: address, coin: Coin<CoinType>) {
        deposit_internal(ctx, addr, coin);
    }

    #[private_generics(CoinType)]
    /// Transfer `amount` of coins `CoinType` from `from` to `to`.
    /// This function is only called by the `CoinType` module, for the developer to extend custom transfer logic
    public fun transfer_extend<CoinType: key>(
        ctx: &mut Context,
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
        ctx: &mut Context,
        coin: Coin<CoinType>,
    ) {
        burn_internal(ctx, coin) 
    }

    #[private_generics(CoinType)]
    /// Freeze a CoinStore to prevent transfers
    public fun freeze_coin_store_extend<CoinType: key>(
        ctx: &mut Context,
        addr: address,
    ) {
        ensure_coin_store<CoinType>(ctx, addr);
        let coin_store = borrow_mut_account_coin_store<CoinType>(ctx, addr);
        coin_store.frozen = true;
    }

    #[private_generics(CoinType)]
    /// Unfreeze a CoinStore to allow transfers
    public fun unfreeze_coin_store_extend<CoinType: key>(
        ctx: &mut Context,
        addr: address,
    ) {
        ensure_coin_store<CoinType>(ctx, addr);
        let coin_store = borrow_mut_account_coin_store<CoinType>(ctx, addr);
        coin_store.frozen = false;
    }

    //
    // Entry functions
    //

    /// Creating a resource that stores balance of `CoinType` on user's account.
    /// Required if user wants to start accepting deposits of `CoinType` in his account.
    public entry fun accept_coin_entry<CoinType: key>(ctx: &mut Context, account: &signer) {
        do_accept_coin<CoinType>(ctx, account)
    }

    /// Enable account's auto-accept-coin feature.
    /// The script function is reenterable.
    public entry fun enable_auto_accept_coin_entry(ctx: &mut Context, account: &signer) {
        set_auto_accept_coin(ctx, account, true)
    }

    /// Disable account's auto-accept-coin feature.
    /// The script function is reenterable.
    public entry fun disable_auto_accept_coin_entry(ctx: &mut Context, account: &signer) {
        set_auto_accept_coin(ctx, account, false);
    }
}