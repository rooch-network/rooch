// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

/// This module provides migration functionality to transition from the generic
/// coin store system (using CoinType) to the non-generic multi coin store system.
/// It helps migrate coin stores, balances, frozen states, and accept data.
module rooch_framework::coin_migration {
    use std::string;
    use moveos_std::type_info;
    use moveos_std::event;
    use moveos_std::object::{Self, ObjectID};
    use moveos_std::table;
    
    use rooch_framework::coin::{convert_coin_to_generic_coin};
    use rooch_framework::coin_store::{Self};
    use rooch_framework::multi_coin_store::{Self};
    use rooch_framework::account_coin_store;

    friend rooch_framework::genesis;

    //
    // Errors
    //

    /// Migration is already done for an account
    const ErrorMigrationAlreadyDone: u64 = 1;

    /// Nothing to migrate for the account
    const ErrorNothingToMigrate: u64 = 2;

    //
    // Events
    //

    /// Event emitted when an account's coin stores are migrated
    struct AccountMigrationEvent has drop, store, copy {
        /// The account address that was migrated
        account: address,
    }

    /// Event emitted when a specific coin store is migrated for an account
    struct CoinStoreMigrationEvent has drop, store, copy {
        /// The account address
        account: address,
        /// The coin type that was migrated
        coin_type: string::String,
        /// The balance that was migrated
        balance: u256,
        /// Whether the coin store was frozen
        was_frozen: bool,
    }

    /// State tracking for migration progress
    struct MigrationState has key, store {
        /// Accounts that have been migrated
        migrated_accounts: table::Table<address, bool>,
    }

    /// Initialize the migration module, called during genesis or framework upgrade
    // public(friend) fun init_migration(_framework: &signer) {
    fun init() {
        let migration_state_id = object::named_object_id<MigrationState>();
        if(!object::exists_object(migration_state_id)){
            let migration_state = object::new_named_object(MigrationState{
                migrated_accounts: table::new<address, bool>(),
            });
            object::transfer_extend(migration_state, @rooch_framework);
        };
    }

    /// Entry function to migrate a specific account's coin stores
    /// The coin type must be only key to compatiable with both the public(key+store) and private(key) coins
    /// Need to limit to only called by the admin to migrate their own coin stores ?
    public entry fun migrate_account_entry<CoinType: key>(_admin: &signer, addr: address) {
        // let addr = signer::address_of(account);
        migrate_account<CoinType>(addr);
    }

    /// Entry function to update migration state for a specific account
    /// Need to limit to only called by the admin to migrate their own coin stores ?
    public entry fun update_migration_state_entry(_admin: &signer, addr: address) {
        // let addr = signer::address_of(account);
        update_migration_state(addr);
    }

    public fun migration_state_id(): ObjectID {
        object::named_object_id<MigrationState>()
    }
    /// Check if an account has already been migrated
    public fun is_account_migrated(addr: address): bool {
        let state_id = migration_state_id();
        let state_obj = object::borrow_object<MigrationState>(state_id);
        let state = object::borrow(state_obj);
        table::contains(&state.migrated_accounts, addr)
    }

    /// Get migration statistics
    public fun get_migration_stats(): u64 {
        let state_id = migration_state_id();
        let state_obj = object::borrow_object<MigrationState>(state_id);
        let state = object::borrow(state_obj);
        table::length(&state.migrated_accounts)
    }

    /// This function handles the migration state
    fun update_migration_state(addr: address) {
        let state_id = migration_state_id();
        let state_obj = object::borrow_mut_object_extend<MigrationState>(state_id);
        let state = object::borrow_mut<MigrationState>(state_obj);

        // Check if already migrated
        if (table::contains(&state.migrated_accounts, addr)) {
            return
        };

        // Update migration state
        table::add(&mut state.migrated_accounts, addr, true);

        event::emit(AccountMigrationEvent {
            account: addr,
        });
    }

    /// Migrate a specific coin type for an
    /// The coin type must be only key to compatiable with both the public(key+store) and private(key) coins
    /// Returns whether migration was performed and the balance migrated
    fun migrate_account<CoinType: key>(
        addr: address, 
    ): (bool, u256) {
        // Check if there's a coin store for this coin type
        let coin_store_id = account_coin_store::account_coin_store_id<CoinType>(addr);
        if (!object::exists_object(coin_store_id)) {
            return (false, 0)
        };
        let coin_store = coin_store::borrow_mut_coin_store_internal<CoinType>(coin_store_id);

        let coin_type = type_info::type_name<CoinType>();
        let multi_coin_store_id = account_coin_store::multi_coin_store_id(addr);
        // Create multi coin store if not exist
        if (!account_coin_store::exist_multi_coin_store(addr)) {
            multi_coin_store::create_multi_coin_store(addr);
        };

        if (!account_coin_store::exist_multi_coin_store_field(addr, coin_type)) {
            // TO avoid he object or field multi coin store is already borrowed
            let tmp_multi_coin_store = multi_coin_store::borrow_mut_coin_store_internal(multi_coin_store_id);
            multi_coin_store::create_coin_store_field_if_not_exist(tmp_multi_coin_store, coin_type);
        };
        let multi_coin_store = multi_coin_store::borrow_mut_coin_store_internal(multi_coin_store_id);
        // Check if frozen and get balance
        let was_frozen = coin_store::is_frozen(coin_store);
        let balance = coin_store::balance(coin_store);

        // Withdraw all coins from the source
        let coin = coin_store::withdraw_uncheck_internal<CoinType>(coin_store, balance);

        // Convert to GenericCoin
        let generic_coin = convert_coin_to_generic_coin(coin);

        // Deposit to destination
        multi_coin_store::deposit_internal(multi_coin_store, generic_coin);

        // Set frozen state if needed
        if (was_frozen) {
            multi_coin_store::freeze_coin_store_internal(multi_coin_store, coin_type, true);
        };
        
        // Emit event
        event::emit(CoinStoreMigrationEvent {
            account: addr, 
            coin_type,
            balance,
            was_frozen
        });
        
        (true, balance)
    }

    //
    // Test-only functions
    //
    #[test_only]
    public fun init_for_test() {
        init();
    }

    #[test_only]
    public fun migrate_account_for_test<CoinType: key + store>(addr: address) {
        let (_,_) = migrate_account<CoinType>(addr);
    }
} 