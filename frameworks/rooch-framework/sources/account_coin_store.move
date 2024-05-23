// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

module rooch_framework::account_coin_store {

    use moveos_std::account;
    use moveos_std::object::{Self, ObjectID, Object};
    use moveos_std::table;
    use moveos_std::table::Table;
    use moveos_std::event;
    use moveos_std::signer;

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
        if (exist_account_coin_store<CoinType>(addr)) {
            let coin_store = borrow_account_coin_store<CoinType>(addr);
            coin_store::balance(coin_store)
        } else {
            0u256
        }
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


    fun withdraw_internal<CoinType: key>(
        
        addr: address,
        amount: u256,
    ): Coin<CoinType> {
        let coin_store = borrow_mut_account_coin_store<CoinType>(addr);
        coin_store::withdraw_internal(coin_store, amount)
    }

    fun deposit_internal<CoinType: key>(addr: address, coin: Coin<CoinType>) {
        assert!(
            is_accept_coin<CoinType>(addr),
            ErrorAccountNotAcceptCoin,
        );
        let coin_store = create_or_borrow_mut_account_coin_store<CoinType>(addr);
        coin_store::deposit_internal<CoinType>(coin_store, coin)
    }

    fun transfer_internal<CoinType: key>(
        
        from: address,
        to: address,
        amount: u256,
    ) {
        let coin = withdraw_internal<CoinType>(from, amount);
        deposit_internal(to, coin);
    }
}