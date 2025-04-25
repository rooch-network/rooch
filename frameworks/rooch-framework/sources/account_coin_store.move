// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

module rooch_framework::account_coin_store {
    use std::string;
    use rooch_framework::coin::{Coin, GenericCoin};
    use rooch_framework::coin;
    use moveos_std::type_info;
    use rooch_framework::multi_coin_store;
    use rooch_framework::multi_coin_store::MultiCoinStore;
    use moveos_std::account;
    use moveos_std::object::{Self, ObjectID, Object};
    use moveos_std::table;
    use moveos_std::table::Table;
    use moveos_std::event;
    use moveos_std::signer;

    use rooch_framework::coin_store::{Self, CoinStore};

    friend rooch_framework::genesis;
    friend rooch_framework::account;

    //
    // Errors.
    //

    /// Account hasn't accept `CoinType`
    const ErrorAccountNotAcceptCoin: u64 = 1;

    /// The coin type is not match
    const ErrorCoinTypeNotMatch: u64 = 2;

    /// Not enough coins to extract
    const ErrorInsufficientBalance: u64 = 3;

    /// A resource that holds the AutoAcceptCoin config for all accounts.
    /// The main scenario is that the user can actively turn off the AutoAcceptCoin setting to avoid automatically receiving Coin
    struct AutoAcceptCoins has key,store {
        auto_accept_coins: Table<address, bool>,
    }

    /// Event for auto accept coin set
    struct AcceptCoinEvent has drop, store, copy {
        /// auto accept coin config
        enable: bool,
    }

    public(friend) fun genesis_init(genesis_account: &signer) {
        let auto_accepted_coins = AutoAcceptCoins {
            auto_accept_coins: table::new<address, bool>(),
        };
        account::move_resource_to(genesis_account, auto_accepted_coins);
    }

    // Public functions

    /// Returns the balance of `addr` for provided `CoinType`.
    public fun balance<CoinType: key>(addr: address): u256 {
        let coin_type = type_info::type_name<CoinType>();
        let coin_store_balance = balance_of<CoinType>(addr);
        let multi_coin_store_balance = balance_by_type_name(addr, coin_type);
        let balance = coin_store_balance + multi_coin_store_balance;
        balance
    }

    /// Return the account CoinStore object id for addr
    /// the account CoinStore is a account named object, the id is determinate for each addr and CoinType
    public fun account_coin_store_id<CoinType: key>(addr: address): ObjectID {
        object::account_named_object_id<CoinStore<CoinType>>(addr)
    }

    /// Return whether the account at `addr` accept `Coin` type coins
    public fun is_accept_coin<CoinType: key>(addr: address): bool {
        if (can_auto_accept_coin(addr)) {
            true
        } else {
            exist_account_coin_store<CoinType>(addr)
        }
    }

    /// Check whether the address can auto accept coin.
    /// Default is true if absent
    public fun can_auto_accept_coin(addr: address): bool {
        let auto_accept_coins = account::borrow_resource<AutoAcceptCoins>(@rooch_framework);
        if (table::contains<address, bool>(&auto_accept_coins.auto_accept_coins, addr)) {
            return *table::borrow<address, bool>(&auto_accept_coins.auto_accept_coins, addr)
        };
        true
    }

    /// Add a balance of `Coin` type to the sending account.
    /// If user turns off AutoAcceptCoin, call this method to receive the corresponding Coin
    public fun do_accept_coin<CoinType: key>(account: &signer) {
        let addr = signer::address_of(account);
        create_or_borrow_mut_account_coin_store<CoinType>(addr);

        // Create a multi coin store and multi coin store field for the account
        let coin_type = type_info::type_name<CoinType>();
        do_accept_coin_by_type_name(account, coin_type);
    }

    /// Configure whether auto-accept coins.
    public fun set_auto_accept_coin(account: &signer, enable: bool) {
        let addr = signer::address_of(account);
        let auto_accept_coins = account::borrow_mut_resource<AutoAcceptCoins>(@rooch_framework);
        table::upsert<address, bool>(&mut auto_accept_coins.auto_accept_coins, addr, enable);

        event::emit<AcceptCoinEvent>(AcceptCoinEvent { enable });
    }

    /// Withdraw specified `amount` of coin `CoinType` from the signing account.
    /// This public entry function requires the `CoinType` to have `key` and `store` abilities.
    public fun withdraw<CoinType: key + store>(
        account: &signer,
        amount: u256,
    ): Coin<CoinType> {
        let addr = signer::address_of(account);
        withdraw_internal<CoinType>(addr, amount)
    }

    /// Deposit the coin into the recipient's account and emit an event.
    /// This public entry function requires the `CoinType` to have `key` and `store` abilities.
    public fun deposit<CoinType: key + store>(addr: address, coin: Coin<CoinType>) {
        deposit_internal(addr, coin);
    }


    /// Transfer `amount` of coins `CoinType` from `from` to `to`.
    /// Any account and module can call this function to transfer coins, the `CoinType` must have `key` and `store` abilities.
    public fun transfer<CoinType: key + store>(
        from: &signer,
        to: address,
        amount: u256,
    ) {
        let from_addr = signer::address_of(from);
        transfer_internal<CoinType>(from_addr, to, amount);
    }

    public fun exist_account_coin_store<CoinType: key>(addr: address): bool {
        let account_coin_store_id = account_coin_store_id<CoinType>(addr);
        object::exists_object_with_type<CoinStore<CoinType>>(account_coin_store_id)
    }

    public fun is_account_coin_store_frozen<CoinType: key>(addr: address): bool {
        if (exist_account_coin_store<CoinType>(addr)) {
            let coin_store = borrow_account_coin_store<CoinType>(addr);
            coin_store::is_frozen(coin_store)
        } else {
            false
        }
    }

    #[private_generics(CoinType)]
    /// Withdraw specified `amount` of coin `CoinType` from any addr, this function does not check the Coin `frozen` attribute
    /// This function is only called by the `CoinType` module, for the developer to extend custom withdraw logic
    public fun withdraw_extend<CoinType: key>(
        addr: address,
        amount: u256,
    ): Coin<CoinType> {
        withdraw_internal<CoinType>(addr, amount)
    }

    #[private_generics(CoinType)]
    /// Deposit the coin into the recipient's account and emit an event.
    /// This function is only called by the `CoinType` module, for the developer to extend custom deposit logic
    public fun deposit_extend<CoinType: key>(addr: address, coin: Coin<CoinType>) {
        deposit_internal(addr, coin);
    }

    #[private_generics(CoinType)]
    /// Transfer `amount` of coins `CoinType` from `from` to `to`.
    /// This function is only called by the `CoinType` module, for the developer to extend custom transfer logic
    public fun transfer_extend<CoinType: key>(
        from: address,
        to: address,
        amount: u256,
    ) {
        transfer_internal<CoinType>(from, to, amount);
    }

    #[private_generics(CoinType)]
    /// Freeze or Unfreeze a CoinStore to prevent withdraw and desposit
    /// This function is for he `CoinType` module to extend,
    /// Only the `CoinType` module can freeze or unfreeze a CoinStore by the coin store id
    public fun freeze_extend<CoinType: key>(
        addr: address,
        frozen: bool,
    ) {
        freeze_internal<CoinType>(addr, frozen);
    }

    //
    // Entry functions
    //

    /// Creating a resource that stores balance of `CoinType` on user's account.
    /// Required if user wants to start accepting deposits of `CoinType` in his account.
    public entry fun accept_coin_entry<CoinType: key>(account: &signer) {
        do_accept_coin<CoinType>(account)
    }

    /// Enable account's auto-accept-coin feature.
    /// The script function is reenterable.
    public entry fun enable_auto_accept_coin_entry(account: &signer) {
        set_auto_accept_coin(account, true)
    }

    /// Disable account's auto-accept-coin feature.
    /// The script function is reenterable.
    public entry fun disable_auto_accept_coin_entry(account: &signer) {
        set_auto_accept_coin(account, false);
    }

    //
    // Internal functions
    //

    fun borrow_account_coin_store<CoinType: key>(addr: address): &Object<CoinStore<CoinType>> {
        let account_coin_store_id = account_coin_store_id<CoinType>(addr);
        object::borrow_object<CoinStore<CoinType>>(account_coin_store_id)
    }

    fun borrow_mut_account_coin_store<CoinType: key>(
        addr: address
    ): &mut Object<CoinStore<CoinType>> {
        let account_coin_store_id = account_coin_store_id<CoinType>(addr);
        coin_store::borrow_mut_coin_store_internal<CoinType>(account_coin_store_id)
    }

    fun create_or_borrow_mut_account_coin_store<CoinType: key>(
        addr: address
    ): &mut Object<CoinStore<CoinType>> {
        let account_coin_store_id = account_coin_store_id<CoinType>(addr);
        if (!object::exists_object_with_type<CoinStore<CoinType>>(account_coin_store_id)) {
            coin_store::create_account_coin_store<CoinType>(addr);
        };
        coin_store::borrow_mut_coin_store_internal<CoinType>(account_coin_store_id)
    }

    fun balance_of<CoinType: key>(addr: address): u256 {
        let coin_store_balance = if (exist_account_coin_store<CoinType>(addr)) {
            let coin_store = borrow_account_coin_store<CoinType>(addr);
            coin_store::balance(coin_store)
        } else {
            0u256
        };
        coin_store_balance
    }

    fun withdraw_internal<CoinType: key>(
        addr: address,
        amount: u256,
    ): Coin<CoinType> {
        // Check CoinType-specific coin store first
        let coin_type = type_info::type_name<CoinType>();
        
        // Get the total balance from both stores
        let coin_store_balance = balance_of<CoinType>(addr);
        let generic_balance = balance_of_by_type_name(addr, coin_type);
        
        // Ensure we have enough total balance
        let total_balance = coin_store_balance + generic_balance;
        assert!(total_balance >= amount, ErrorInsufficientBalance);
        
        // If we have enough in the coin store, use it first
        let coin_store_coin = if (coin_store_balance > 0) {
            let withdraw_amount = if (coin_store_balance >= amount) {
                amount
            } else {
                coin_store_balance
            };
            
            let coin_store = borrow_mut_account_coin_store<CoinType>(addr);
            coin_store::withdraw_internal(coin_store, withdraw_amount)
        } else {
            coin::zero<CoinType>()
        };
        
        // If we need more from the multi coin store
        if (coin_store_balance < amount) {
            let generic_amount = amount - coin_store_balance;
            let generic_store = borrow_mut_multi_coin_store(addr);
            let generic_coin = multi_coin_store::withdraw(generic_store, coin_type, generic_amount);
            let generic_coin_store_coin = coin::convert_generic_coin_to_coin<CoinType>(generic_coin);
            coin::merge(&mut coin_store_coin, generic_coin_store_coin);
        };

        coin_store_coin
    }

    fun deposit_internal<CoinType: key>(addr: address, coin: Coin<CoinType>) {
        // try to auto migrate and write multi coin store first
        let coin_type = type_info::type_name<CoinType>();
        assert!(
            is_accept_coin_by_type_name(addr, coin_type),
            ErrorAccountNotAcceptCoin,
        );

        let multi_coin_store = create_or_borrow_mut_multi_coin_store(addr);
        let generic_coin = coin::convert_coin_to_generic_coin(coin);
        multi_coin_store::deposit_internal(multi_coin_store, generic_coin);

        // just for compatiable coin store
        if(is_accept_coin<CoinType>(addr)){
            let _coin_store = create_or_borrow_mut_account_coin_store<CoinType>(addr);
        };
    }

    fun transfer_internal<CoinType: key>(
        from: address,
        to: address,
        amount: u256,
    ) {
        let coin = withdraw_internal<CoinType>(from, amount);
        deposit_internal(to, coin);
    }

    fun freeze_internal<CoinType: key>(
        addr: address,
        frozen: bool,
    ) {
        let coin_store = borrow_mut_account_coin_store<CoinType>(addr);
        coin_store::freeze_coin_store_internal(coin_store, frozen);

        // Compatiable multi coin store
        let coin_type = type_info::type_name<CoinType>();
        let multi_coin_store = borrow_mut_multi_coin_store(addr);
        multi_coin_store::freeze_coin_store_internal(multi_coin_store, coin_type, frozen);
    }

    // === Non-generic functions ===

    public fun balance_by_type_name(addr: address, coin_type: string::String): u256 {
        if (exist_multi_coin_store(addr)) {
            let coin_store = borrow_multi_coin_store(addr);
            multi_coin_store::balance(coin_store, coin_type)
        } else {
            0
        }
    }

    public fun multi_coin_store_id(addr: address): ObjectID {
        object::account_named_object_id<MultiCoinStore>(addr)
    }

    public fun is_accept_coin_by_type_name(addr: address, coin_type: string::String): bool {
        if (can_auto_accept_coin(addr)) {
            true
        } else {
            if(exist_multi_coin_store_field(addr, coin_type)) {
                true
            }else {
                false
            }
        }
    }

    public fun do_accept_coin_by_type_name(account: &signer, coin_type: string::String) {
        let addr = signer::address_of(account);
        let coin_store_obj = create_or_borrow_mut_multi_coin_store(addr);
        multi_coin_store::create_coin_store_field_if_not_exist(coin_store_obj, coin_type);
    }

    public fun withdraw_by_type_name(
        account: &signer,
        coin_type: string::String,
        amount: u256,
    ): GenericCoin {
        let addr = signer::address_of(account);
        withdraw_internal_by_type_name(addr, coin_type, amount)
    }

    public fun deposit_by_type_name(addr: address, coin: GenericCoin) {
        deposit_internal_by_type_name(addr, coin);
    }

    public fun transfer_by_type_name(
        from: &signer,
        to: address,
        coin_type: string::String,
        amount: u256,
    ) {
        let from_addr = signer::address_of(from);
        transfer_internal_by_type_name(from_addr, to, coin_type, amount);
    }

    public fun exist_multi_coin_store(addr: address): bool {
        let coin_store_id = multi_coin_store_id(addr);
        object::exists_object_with_type<MultiCoinStore>(coin_store_id)
    }

    public fun exist_multi_coin_store_field(addr: address, coin_type: string::String): bool {
        if(exist_multi_coin_store(addr)){
            let coin_store_obj = borrow_multi_coin_store(addr);
            multi_coin_store::exist_coin_store_field(coin_store_obj, coin_type)
        }else{
            false
        }
    }

    public fun is_multi_coin_store_frozen_by_type_name(addr: address, coin_type: string::String): bool {
        if (exist_multi_coin_store(addr)) {
            let coin_store_obj = borrow_multi_coin_store(addr);
            multi_coin_store::is_frozen(coin_store_obj, coin_type)
        } else {
            false
        }
    }

    public entry fun accept_coin_entry_by_type_name(account: &signer, coin_type: string::String) {
        do_accept_coin_by_type_name(account, coin_type)
    }

    fun borrow_multi_coin_store(addr: address): &Object<MultiCoinStore> {
        let multi_coin_store_id = multi_coin_store_id(addr);
        object::borrow_object<MultiCoinStore>(multi_coin_store_id)
    }

    fun borrow_mut_multi_coin_store(
        addr: address,
    ): &mut Object<MultiCoinStore> {
        let multi_coin_store_id = multi_coin_store_id(addr);
        multi_coin_store::borrow_mut_coin_store_internal(multi_coin_store_id)
    }

    fun create_or_borrow_mut_multi_coin_store(
        addr: address,
    ): &mut Object<MultiCoinStore> {
        let multi_coin_store_id = multi_coin_store_id(addr);
        if (!object::exists_object_with_type<MultiCoinStore>(multi_coin_store_id)) {
            multi_coin_store::create_multi_coin_store(addr);
        };
        multi_coin_store::borrow_mut_coin_store_internal(multi_coin_store_id)
    }

    fun balance_of_by_type_name(addr: address, coin_type: string::String): u256 {
        let multi_coin_store_balance = if (exist_multi_coin_store(addr)) {
            let coin_store = borrow_multi_coin_store(addr);
            multi_coin_store::balance(coin_store, coin_type)
        } else {
            0u256
        };
        multi_coin_store_balance
    }

    fun withdraw_internal_by_type_name(
        addr: address,
        coin_type: string::String,
        amount: u256,
    ): GenericCoin {
        let coin_store = borrow_mut_multi_coin_store(addr);
        multi_coin_store::withdraw(coin_store, coin_type, amount)
    }

    fun deposit_internal_by_type_name(addr: address, coin: GenericCoin) {
        let coin_type = coin::coin_type(&coin);
        assert!(
            is_accept_coin_by_type_name(addr, coin_type),
            ErrorAccountNotAcceptCoin,
        );
        let coin_store = create_or_borrow_mut_multi_coin_store(addr);
        multi_coin_store::deposit(coin_store, coin)
    }

    fun transfer_internal_by_type_name(
        from: address,
        to: address,
        coin_type: string::String,
        amount: u256,
    ) {
        let coin = withdraw_internal_by_type_name(from, coin_type, amount);
        deposit_internal_by_type_name(to, coin);
    }


    #[test_only]
    public fun do_accept_coin_only_for_coin_store_for_test<CoinType: key>(account: &signer) {
        let addr = signer::address_of(account);
        create_or_borrow_mut_account_coin_store<CoinType>(addr);
    }
}