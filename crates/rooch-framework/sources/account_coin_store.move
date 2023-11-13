// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

module rooch_framework::account_coin_store {

    use std::string;
    use std::error;
    use std::option;
    use std::option::Option;
    use moveos_std::object::ObjectID;
    use moveos_std::table;
    use moveos_std::table::Table;
    use moveos_std::context::{Self, Context};
    use moveos_std::event;
    use moveos_std::type_info;
    use moveos_std::signer;
    use moveos_std::object::{Self, Object};
    use rooch_framework::coin::{Coin};
    use rooch_framework::coin_store::{Self, CoinStore};
 
    friend rooch_framework::genesis;
    friend rooch_framework::account;

    //
    // Errors.
    //

    /// Account hasn't accept `CoinType`
    const ErrorAccountNotAcceptCoin: u64 = 1;
    
    /// A resource that holds the AutoAcceptCoin config for all accounts.
    /// The main scenario is that the user can actively turn off the AutoAcceptCoin setting to avoid automatically receiving Coin
    struct AutoAcceptCoins has key {
        auto_accept_coins: Table<address, bool>,
    }

    /// A resource that holds all the ids of Object<CoinStore<T>> for account.
    /// TODO after the indexer is ready, we can use the indexer to list all the CoinStore<T> objects for account
    struct CoinStores has key {
        coin_stores: Table<string::String, ObjectID>,
    }

    /// Event for auto accept coin set
    struct AcceptCoinEvent has drop, store {
        /// auto accept coin config
        enable: bool,
    }

    public(friend) fun genesis_init(ctx: &mut Context, genesis_account: &signer) {
        let auto_accepted_coins = AutoAcceptCoins {
            auto_accept_coins: context::new_table<address, bool>(ctx),
        };
        context::move_resource_to(ctx, genesis_account, auto_accepted_coins);
    }

    public(friend) fun init_account_coin_stores(ctx: &mut Context, account: &signer){
        let coin_stores = CoinStores {
            coin_stores: context::new_table<string::String, ObjectID>(ctx),
        };
        context::move_resource_to(ctx, account, coin_stores);
    }

    // Public functions
    
    /// Returns the balance of `addr` for provided `CoinType`.
    public fun balance<CoinType: key>(ctx: &Context, addr: address): u256 {
        if(exist_account_coin_store<CoinType>(ctx, addr)) {
            let coin_store = borrow_account_coin_store<CoinType>(ctx, addr);
            coin_store::balance(coin_store)
        } else {
            0u256
        }
    }

    /// Return the account CoinStore object id for addr
    public fun coin_store_id<CoinType: key>(ctx: &Context, addr: address): Option<ObjectID> {
        let account_coin_store_id = object::account_singleton_object_id<CoinStore<CoinType>>(addr);
        if (context::exists_object<CoinStore<CoinType>>(ctx, account_coin_store_id)) {
            option::some(account_coin_store_id)
        } else {
            option::none<ObjectID>()
        }
    }

    /// Return CoinStores table handle for addr
    public fun coin_stores_handle(ctx: &Context, addr: address): Option<ObjectID> {
        if (context::exists_resource<CoinStores>(ctx, addr))
        {
            let coin_stores = context::borrow_resource<CoinStores>(ctx, addr);
            option::some(*table::handle(&coin_stores.coin_stores))
        } else {
            option::none<ObjectID>()
        }
    }

    /// Return whether the account at `addr` accept `Coin` type coins
    public fun is_accept_coin<CoinType: key>(ctx: &Context, addr: address): bool {
        if (can_auto_accept_coin(ctx, addr)) {
            true
        } else {
            exist_account_coin_store<CoinType>(ctx, addr)
        }
    }

    /// Check whether the address can auto accept coin.
    /// Default is true if absent
    public fun can_auto_accept_coin(ctx: &Context, addr: address): bool {
        let auto_accept_coins = context::borrow_resource<AutoAcceptCoins>(ctx, @rooch_framework);
        if (table::contains<address, bool>(&auto_accept_coins.auto_accept_coins, addr)) {
            return *table::borrow<address, bool>(&auto_accept_coins.auto_accept_coins, addr)
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
        let auto_accept_coins = context::borrow_mut_resource<AutoAcceptCoins>(ctx, @rooch_framework);
        table::upsert<address, bool>(&mut auto_accept_coins.auto_accept_coins, addr, enable);

        event::emit<AcceptCoinEvent>(AcceptCoinEvent { enable});
    }

    /// Withdraw specifed `amount` of coin `CoinType` from the signing account.
    /// This public entry function requires the `CoinType` to have `key` and `store` abilities.
    public fun withdraw<CoinType: key + store>(
        ctx: &mut Context,
        account: &signer,
        amount: u256,
    ): Coin<CoinType> {
        let addr = signer::address_of(account);
        withdraw_internal<CoinType>(ctx, addr, amount) 
    }

    /// Deposit the coin into the recipient's account and emit an event.
    /// This public entry function requires the `CoinType` to have `key` and `store` abilities.
    public fun deposit<CoinType: key + store>(ctx: &mut Context, addr: address, coin: Coin<CoinType>) {
        deposit_internal(ctx, addr, coin);
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
        transfer_internal<CoinType>(ctx, from_addr, to, amount);
    }

    public fun exist_account_coin_store<CoinType: key>(ctx: &Context, addr: address): bool {
        let account_coin_store_id = object::account_singleton_object_id<CoinStore<CoinType>>(addr);
        context::exists_object<CoinStore<CoinType>>(ctx, account_coin_store_id)
    }

    public fun is_account_coin_store_frozen<CoinType: key>(ctx: &Context, addr: address): bool {
        if (exist_account_coin_store<CoinType>(ctx, addr)) {
            let coin_store = borrow_account_coin_store<CoinType>(ctx, addr);
            coin_store::is_frozen(coin_store)
        } else {
            false
        }
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

    //
    // Internal functions
    // 

    fun borrow_account_coin_store<CoinType: key>(ctx: &Context, addr: address): &Object<CoinStore<CoinType>>{
        let account_coin_store_id = object::account_singleton_object_id<CoinStore<CoinType>>(addr);
        context::borrow_object<CoinStore<CoinType>>(ctx, account_coin_store_id)
    }

    fun borrow_mut_account_coin_store<CoinType: key>(ctx: &mut Context, addr: address): &mut Object<CoinStore<CoinType>>{
        let account_coin_store_id = object::account_singleton_object_id<CoinStore<CoinType>>(addr);
        coin_store::borrow_mut_coin_store<CoinType>(ctx, account_coin_store_id)
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

    fun create_account_coin_store<CoinType: key>(ctx: &mut Context, addr: address) {
        let coin_store_obj = coin_store::create_account_coin_store<CoinType>(ctx, addr);
        let account_coin_store_id = object::id(coin_store_obj);
        let coin_stores = context::borrow_mut_resource<CoinStores>(ctx, addr);
        let coin_type = type_info::type_name<CoinType>();
        table::add(&mut coin_stores.coin_stores, coin_type, account_coin_store_id);
    }


    fun withdraw_internal<CoinType: key>(
        ctx: &mut Context,
        addr: address,
        amount: u256,
    ): Coin<CoinType> {
        let coin_store = borrow_mut_account_coin_store<CoinType>(ctx, addr);
        coin_store::withdraw(coin_store, amount)
    }

    fun deposit_internal<CoinType: key>(ctx: &mut Context, addr: address, coin: Coin<CoinType>) {
        assert!(
            is_accept_coin<CoinType>(ctx, addr),
            error::not_found(ErrorAccountNotAcceptCoin),
        );

        ensure_coin_store<CoinType>(ctx, addr);

        let coin_store = borrow_mut_account_coin_store<CoinType>(ctx, addr);
        coin_store::deposit<CoinType>(coin_store, coin)
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

}