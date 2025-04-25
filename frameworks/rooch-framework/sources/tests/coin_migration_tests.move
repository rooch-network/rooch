// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

#[test_only]
module rooch_framework::coin_migration_tests {
    use std::string;
    use std::option;
    use rooch_framework::onchain_config;
    use rooch_framework::coin_store;
    use moveos_std::account;
    use moveos_std::object;
    use moveos_std::type_info;
    
    use rooch_framework::coin;
    use rooch_framework::account_coin_store;
    use rooch_framework::coin_migration;
    use rooch_framework::genesis;

    // Test coin types
    struct TestCoin1 has key, store {}
    struct TestCoin2 has key, store {}

    // Addresses for testing
    const ALICE: address = @0x42;
    const BOB: address = @0x43;
    const CHARLIE: address = @0x44;

    #[test]
    fun test_migration_init() {
        genesis::init_for_test();
        
        // Initialize migration module
        coin_migration::init_for_test();
        
        // Verify migration state is initialized
        let migration_state_id = coin_migration::migration_state_id();
        assert!(object::exists_object(migration_state_id), 1);
        
        // Verify no accounts are migrated yet
        assert!(coin_migration::get_migration_stats() == 0, 2);
    }

    #[test(rooch_framework = @rooch_framework)]
    fun test_account_migration_status(rooch_framework: &signer) {
        genesis::init_for_test();
        coin_migration::init_for_test();
        let onchain_config_admin = account::create_account_for_testing(onchain_config::admin());
        coin_migration::dispatch_cap_for_test(&onchain_config_admin, @rooch_framework);

        let _alice = account::create_account_for_testing(ALICE);

        // Initially account should not be migrated
        assert!(!coin_migration::is_account_migrated(ALICE), 1);
        
        // Update migration state for the account
        coin_migration::update_migration_state_entry(rooch_framework, ALICE);
        
        // Now account should be marked as migrated
        assert!(coin_migration::is_account_migrated(ALICE), 2);
        
        // Stats should show 1 account migrated
        assert!(coin_migration::get_migration_stats() == 1, 3);
    }

    #[test(rooch_framework = @rooch_framework)]
    fun test_migration_with_existing_coin_stores(rooch_framework: &signer) {
        genesis::init_for_test();
        coin_migration::init_for_test();
        let onchain_config_admin = account::create_account_for_testing(onchain_config::admin());
        coin_migration::dispatch_cap_for_test(&onchain_config_admin, @rooch_framework);
        
        // Setup test accounts
        let alice = account::create_account_for_testing(ALICE);
        
        // Register coin types
        let coin1_info = register_coin<TestCoin1>(rooch_framework, b"TestCoin1", b"TC1", 8);

        // Create coin store and deposit coins
        account_coin_store::do_accept_coin_only_for_coin_store_for_test<TestCoin1>(&alice);

        // Mint and deposit some coins
        let coin1 = mint_coin<TestCoin1>(&mut coin1_info, 500);
        let coin_store_id1 = account_coin_store::account_coin_store_id<TestCoin1>(ALICE);
        coin_store::deposit_for_test<TestCoin1>(coin_store_id1, coin1);

        // Verify coin store exists before migration
        assert!(object::exists_object(coin_store_id1), 1);
        
        // Verify balance in coin store
        assert!(coin_store::balance_for_test<TestCoin1>(coin_store_id1) == 500, 2);

        // Migrate the specific coin type
        coin_migration::migrate_account_entry<TestCoin1>(rooch_framework, ALICE);

        // Verify migration moved balance to multi coin store
        let coin_type = type_info::type_name<TestCoin1>();
        assert!(account_coin_store::balance_by_type_name(ALICE, coin_type) == 500, 3);

        // Update migration state
        coin_migration::update_migration_state_entry(rooch_framework, ALICE);

        // Account should be marked as migrated
        assert!(coin_migration::is_account_migrated(ALICE), 4);

        // cleanup
        object::transfer(coin1_info, @rooch_framework);
    }

    #[test(rooch_framework = @rooch_framework)]
    fun test_end_to_end_migration(rooch_framework: &signer) {
        genesis::init_for_test();
        coin_migration::init_for_test();
        let onchain_config_admin = account::create_account_for_testing(onchain_config::admin());
        coin_migration::dispatch_cap_for_test(&onchain_config_admin, @rooch_framework);
        
        // Setup test accounts with various coin stores
        let alice = account::create_account_for_testing(ALICE);
        
        // Register coin types
        let coin1_info = register_coin<TestCoin1>(rooch_framework, b"TestCoin1", b"TC1", 8);
        let coin2_info = register_coin<TestCoin2>(rooch_framework, b"TestCoin2", b"TC2", 6);
        
        // Create coin stores
        account_coin_store::do_accept_coin_only_for_coin_store_for_test<TestCoin1>(&alice);
        account_coin_store::do_accept_coin_only_for_coin_store_for_test<TestCoin2>(&alice);
        
        // Mint and deposit some coins
        let coin1 = mint_coin<TestCoin1>(&mut coin1_info, 1000);
        let coin_store_id1 = account_coin_store::account_coin_store_id<TestCoin1>(ALICE);
        coin_store::deposit_for_test<TestCoin1>(coin_store_id1, coin1);
        
        let coin2 = mint_coin<TestCoin2>(&mut coin2_info, 2000);
        let coin_store_id2 = account_coin_store::account_coin_store_id<TestCoin2>(ALICE);
        coin_store::deposit_for_test<TestCoin2>(coin_store_id2, coin2);
        
        // Freeze one of the coin stores
        coin_store::freeze_coin_store_for_test<TestCoin1>(coin_store_id1, true);

        // Initial balances - verify data structures before migration
        let balance1_before = account_coin_store::balance<TestCoin1>(ALICE);
        let balance2_before = account_coin_store::balance<TestCoin2>(ALICE);

        assert!(balance1_before == 1000, 1);
        assert!(balance2_before == 2000, 2);
        
        // Check frozen state before migration
        assert!(account_coin_store::is_account_coin_store_frozen<TestCoin1>(ALICE), 3);
        assert!(!account_coin_store::is_account_coin_store_frozen<TestCoin2>(ALICE), 4);
        
        // Verify original CoinStore structures
        let coin_store_id1 = account_coin_store::account_coin_store_id<TestCoin1>(ALICE);
        let coin_store_id2 = account_coin_store::account_coin_store_id<TestCoin2>(ALICE);
        assert!(object::exists_object(coin_store_id1), 5);
        assert!(object::exists_object(coin_store_id2), 6);
        
        // Verify multi coin store doesn't have these coins yet
        let coin_type1 = type_info::type_name<TestCoin1>();
        let coin_type2 = type_info::type_name<TestCoin2>();
        assert!(!account_coin_store::exist_multi_coin_store_field(ALICE, coin_type1) || 
                account_coin_store::balance_by_type_name(ALICE, coin_type1) == 0, 7);
        assert!(!account_coin_store::exist_multi_coin_store_field(ALICE, coin_type2) || 
                account_coin_store::balance_by_type_name(ALICE, coin_type2) == 0, 8);
        
        // Run migration for each coin type
        coin_migration::migrate_account_entry<TestCoin1>(rooch_framework, ALICE);
        coin_migration::migrate_account_entry<TestCoin2>(rooch_framework, ALICE);
        
        // Update migration state
        coin_migration::update_migration_state_entry(rooch_framework, ALICE);
        
        // Verify balances after migration - combined from both stores
        let balance1_after = account_coin_store::balance<TestCoin1>(ALICE);
        let balance2_after = account_coin_store::balance<TestCoin2>(ALICE);
        
        assert!(balance1_after == balance1_before, 9);
        assert!(balance2_after == balance2_before, 10);
        
        // Verify specifically that balances are in multi_coin_store
        let multi_balance1 = account_coin_store::balance_by_type_name(ALICE, coin_type1);
        let multi_balance2 = account_coin_store::balance_by_type_name(ALICE, coin_type2);
        assert!(multi_balance1 == 1000, 11);
        assert!(multi_balance2 == 2000, 12);
        
        // Verify frozen state persisted in multi coin store
        assert!(account_coin_store::is_multi_coin_store_frozen_by_type_name(ALICE, coin_type1), 13);
        assert!(!account_coin_store::is_multi_coin_store_frozen_by_type_name(ALICE, coin_type2), 14);
        
        // Account should be marked as migrated
        assert!(coin_migration::is_account_migrated(ALICE), 15);
        
        // Stats should reflect the migration
        assert!(coin_migration::get_migration_stats() == 1, 16);

        // cleanup
        object::transfer(coin1_info, @rooch_framework);
        object::transfer(coin2_info, @rooch_framework);
    }

    // Helper functions for testing

    fun register_coin<CoinType: key + store>(
        _admin: &signer,
        name: vector<u8>,
        symbol: vector<u8>,
        decimals: u8
    ): object::Object<coin::CoinInfo<CoinType>> {
        coin::register_for_test<CoinType>(
            string::utf8(name),
            string::utf8(symbol),
            option::none(),
            decimals,
        )
    }

    fun mint_coin<CoinType: key + store>(
        coin_info: &mut object::Object<coin::CoinInfo<CoinType>>,
        amount: u256
    ): coin::Coin<CoinType> {
        coin::mint<CoinType>(coin_info, amount)
    }
} 