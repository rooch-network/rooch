// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

#[test_only]
module rooch_framework::coin_compatibility_tests {
    use std::string;
    use std::option;
    use moveos_std::signer;
    use rooch_framework::coin;
    use rooch_framework::account_coin_store;
    use rooch_framework::multi_coin_store;
    use moveos_std::account;
    use moveos_std::object;
    use moveos_std::type_info;

    use rooch_framework::genesis;

    // Test coin types
    struct TestCoin1 has key, store {}
    struct TestCoin2 has key, store {}
    struct TestCoin3 has key, store {}
    struct TestCoin4 has key {} // Coin with only key ability, no store

    // Addresses for testing
    const ALICE: address = @0x42;
    const BOB: address = @0x43;

    #[test(account = @0x40)]
    fun test_init_coin_stores(account: &signer) {
        genesis::init_for_test();
        let alice = account::create_account_for_testing(ALICE);
        
        // Register coin types
        let coin1_info = register_coin<TestCoin1>(account, b"TestCoin1", b"TC1", 8);
        
        // Test generic coin store creation
        account_coin_store::do_accept_coin<TestCoin1>(&alice);
        
        // Verify both stores are created
        assert!(account_coin_store::exist_account_coin_store<TestCoin1>(ALICE), 1);
        assert!(account_coin_store::exist_multi_coin_store(ALICE), 2);
        
        // Verify coin type string representation
        let coin_type = type_info::type_name<TestCoin1>();
        assert!(account_coin_store::exist_multi_coin_store_field(ALICE, coin_type), 3);

        object::transfer(coin1_info, @rooch_framework);
    }

    #[test(account = @0x40)]
    fun test_deposit_withdraw_compatibility(account: &signer) {
        genesis::init_for_test();
        let alice = account::create_account_for_testing(ALICE);
        let bob = account::create_account_for_testing(BOB);
        
        // Register coin type
        let coin1_info = register_coin<TestCoin1>(account, b"TestCoin1", b"TC1", 8);
        
        // Create coin stores for both users
        account_coin_store::do_accept_coin<TestCoin1>(&alice);
        account_coin_store::do_accept_coin<TestCoin1>(&bob);
        
        // Mint coins
        let coins_to_mint = 1000;
        let coin1 = mint_coin<TestCoin1>(&mut coin1_info, coins_to_mint);
        
        // Deposit via generic interface
        account_coin_store::deposit<TestCoin1>(ALICE, coin1);
        
        // Verify total balance is correct
        let alice_balance = account_coin_store::balance<TestCoin1>(ALICE);
        assert!(alice_balance == coins_to_mint, 1);
        
        // Check balance in each store component
        let coin_type = type_info::type_name<TestCoin1>();
        let multi_store_balance = account_coin_store::balance_by_type_name(ALICE, coin_type);
        
        // In current implementation, the coins should be in the multi_coin_store
        assert!(multi_store_balance == coins_to_mint, 2);
        
        // Test withdrawing via generic interface
        let withdraw_amount1 = 400;
        let coin1 = account_coin_store::withdraw<TestCoin1>(&alice, withdraw_amount1);
        
        // Verify balance updates
        alice_balance = account_coin_store::balance<TestCoin1>(ALICE);
        assert!(alice_balance == coins_to_mint - withdraw_amount1, 3);
        
        // Deposit to Bob using non-generic interface
        let coin_type = type_info::type_name<TestCoin1>();
        let generic_coin = coin::convert_coin_to_generic_coin(coin1);
        account_coin_store::deposit_by_type_name(BOB, generic_coin);
        
        // Verify Bob's balance
        let bob_balance = account_coin_store::balance<TestCoin1>(BOB);
        assert!(bob_balance == withdraw_amount1, 4);
        
        // Test withdrawing via non-generic interface
        let withdraw_amount2 = 200;
        let generic_coin = account_coin_store::withdraw_by_type_name(&bob, coin_type, withdraw_amount2);
        
        // Verify balance updates
        bob_balance = account_coin_store::balance<TestCoin1>(BOB);
        assert!(bob_balance == withdraw_amount1 - withdraw_amount2, 5);
        
        // Deposit back to Alice using generic interface
        let (_, value) = coin::unpack_generic_coin_for_test(generic_coin);
        let typed_coin = coin::pack_for_test<TestCoin1>(value);
        account_coin_store::deposit<TestCoin1>(ALICE, typed_coin);
        
        // Verify final balances
        alice_balance = account_coin_store::balance<TestCoin1>(ALICE);
        assert!(alice_balance == coins_to_mint - withdraw_amount1 + withdraw_amount2, 6);

        object::transfer(coin1_info, @rooch_framework);
    }

    #[test(account = @0x40)]
    fun test_transfer_compatibility(account: &signer) {
        genesis::init_for_test();
        let alice = account::create_account_for_testing(ALICE);
        let bob = account::create_account_for_testing(BOB);
        
        // Register coin type
        let coin1_info = register_coin<TestCoin1>(account, b"TestCoin1", b"TC1", 8);
        
        // Create coin stores for both users
        account_coin_store::do_accept_coin<TestCoin1>(&alice);
        account_coin_store::do_accept_coin<TestCoin1>(&bob);
        
        // Mint coins
        let coins_to_mint = 1000;
        let coin1 = mint_coin<TestCoin1>(&mut coin1_info, coins_to_mint);
        
        // Deposit to Alice
        account_coin_store::deposit<TestCoin1>(ALICE, coin1);
        
        // Transfer using generic interface
        let transfer_amount1 = 300;
        account_coin_store::transfer<TestCoin1>(&alice, BOB, transfer_amount1);
        
        // Verify balances after generic transfer
        let alice_balance = account_coin_store::balance<TestCoin1>(ALICE);
        let bob_balance = account_coin_store::balance<TestCoin1>(BOB);
        assert!(alice_balance == coins_to_mint - transfer_amount1, 1);
        assert!(bob_balance == transfer_amount1, 2);
        
        // Transfer using non-generic interface
        let coin_type = type_info::type_name<TestCoin1>();
        let transfer_amount2 = 200;
        account_coin_store::transfer_by_type_name(&bob, ALICE, coin_type, transfer_amount2);
        
        // Verify balances after non-generic transfer
        alice_balance = account_coin_store::balance<TestCoin1>(ALICE);
        bob_balance = account_coin_store::balance<TestCoin1>(BOB);
        assert!(alice_balance == coins_to_mint - transfer_amount1 + transfer_amount2, 3);
        assert!(bob_balance == transfer_amount1 - transfer_amount2, 4);

        // cleanup
        object::transfer(coin1_info, @rooch_framework);
    }

    #[test(account = @0x40)]
    fun test_multi_account_compatibility(account: &signer) {
        genesis::init_for_test();
        let alice = account::create_account_for_testing(ALICE);
        let bob = account::create_account_for_testing(BOB);
        
        // Register multiple coin types
        let coin1_info = register_coin<TestCoin1>(account, b"TestCoin1", b"TC1", 8);
        let coin2_info = register_coin<TestCoin2>(account, b"TestCoin2", b"TC2", 6);
        let coin3_info = register_coin<TestCoin3>(account, b"TestCoin3", b"TC3", 10);
        
        // Initialize coin stores
        account_coin_store::do_accept_coin<TestCoin1>(&alice);
        account_coin_store::do_accept_coin<TestCoin2>(&alice);
        account_coin_store::do_accept_coin<TestCoin3>(&alice);
        
        account_coin_store::do_accept_coin<TestCoin1>(&bob);
        account_coin_store::do_accept_coin<TestCoin2>(&bob);
        account_coin_store::do_accept_coin<TestCoin3>(&bob);
        
        // Mint different coins
        let coin1 = mint_coin<TestCoin1>(&mut coin1_info, 1000);
        let coin2 = mint_coin<TestCoin2>(&mut coin2_info, 2000);
        let coin3 = mint_coin<TestCoin3>(&mut coin3_info, 3000);
        
        // Deposit coins using different methods
        account_coin_store::deposit<TestCoin1>(ALICE, coin1); // Generic deposit
        
        let _coin_type2 = type_info::type_name<TestCoin2>();
        let generic_coin2 = coin::convert_coin_to_generic_coin(coin2);
        account_coin_store::deposit_by_type_name(ALICE, generic_coin2); // Non-generic deposit
        
        account_coin_store::deposit<TestCoin3>(ALICE, coin3); // Generic deposit
        
        // Verify balances
        assert!(account_coin_store::balance<TestCoin1>(ALICE) == 1000, 1);
        assert!(account_coin_store::balance<TestCoin2>(ALICE) == 2000, 2);
        assert!(account_coin_store::balance<TestCoin3>(ALICE) == 3000, 3);
        
        // Transfer using mixed methods
        account_coin_store::transfer<TestCoin1>(&alice, BOB, 300); // Generic transfer
        
        let coin_type2 = type_info::type_name<TestCoin2>();
        account_coin_store::transfer_by_type_name(&alice, BOB, coin_type2, 500); // Non-generic transfer
        
        account_coin_store::transfer<TestCoin3>(&alice, BOB, 700); // Generic transfer
        
        // Verify all balances after transfers
        assert!(account_coin_store::balance<TestCoin1>(ALICE) == 700, 4);
        assert!(account_coin_store::balance<TestCoin2>(ALICE) == 1500, 5);
        assert!(account_coin_store::balance<TestCoin3>(ALICE) == 2300, 6);
        
        assert!(account_coin_store::balance<TestCoin1>(BOB) == 300, 7);
        assert!(account_coin_store::balance<TestCoin2>(BOB) == 500, 8);
        assert!(account_coin_store::balance<TestCoin3>(BOB) == 700, 9);

        // cleanup
        object::transfer(coin1_info, @rooch_framework);
        object::transfer(coin2_info, @rooch_framework);
        object::transfer(coin3_info, @rooch_framework);

    }

    #[test(account = @0x40)]
    fun test_mixed_deposits_withdrawals(account: &signer) {
        genesis::init_for_test();
        let alice = account::create_account_for_testing(ALICE);
        
        // Register coin type
        let coin1_info = register_coin<TestCoin1>(account, b"TestCoin1", b"TC1", 8);
        
        // Initialize stores
        account_coin_store::do_accept_coin<TestCoin1>(&alice);
        
        // Test scenario where we deposit using multiple methods
        let coin_type = type_info::type_name<TestCoin1>();
        
        // First deposit (generic)
        let coin1 = mint_coin<TestCoin1>(&mut coin1_info, 500);
        account_coin_store::deposit<TestCoin1>(ALICE, coin1);
        
        // Second deposit (non-generic)
        let coin2 = mint_coin<TestCoin1>(&mut coin1_info, 700);
        let generic_coin = coin::convert_coin_to_generic_coin(coin2);
        account_coin_store::deposit_by_type_name(ALICE, generic_coin);
        
        // Verify total balance
        let total_balance = account_coin_store::balance<TestCoin1>(ALICE);
        assert!(total_balance == 1200, 1);
        
        // Test withdrawals from mixed stores
        // The withdraw_internal function should handle balances from both stores
        
        // Withdraw less than what's in the multi_coin_store
        let coin3 = account_coin_store::withdraw<TestCoin1>(&alice, 300);
        assert!(coin::value(&coin3) == 300, 2);
        
        // Verify balance decreased correctly
        total_balance = account_coin_store::balance<TestCoin1>(ALICE);
        assert!(total_balance == 900, 3);
        
        // Return coin to store
        account_coin_store::deposit<TestCoin1>(ALICE, coin3);
        
        // Now withdraw more than what's in multi_coin_store but less than total
        let coin4 = account_coin_store::withdraw<TestCoin1>(&alice, 1000);
        
        // Verify we got the right amount
        assert!(coin::value(&coin4) == 1000, 4);
        
        // Verify remaining balance
        total_balance = account_coin_store::balance<TestCoin1>(ALICE);
        assert!(total_balance == 200, 5);
        
        // Try a non-generic withdrawal
        let generic_coin = account_coin_store::withdraw_by_type_name(&alice, coin_type, 200);
        
        // Verify amount
        assert!(coin::generic_coin_value(&generic_coin) == 200, 6);
        
        // Verify balance is zero
        total_balance = account_coin_store::balance<TestCoin1>(ALICE);
        assert!(total_balance == 0, 7);

        // cleanup
        coin::unpack_for_test(coin4);
        coin::unpack_generic_coin_for_test(generic_coin);
        object::transfer(coin1_info, @rooch_framework);
    }

    #[test(account = @0x40)]
    #[expected_failure(abort_code = account_coin_store::ErrorInsufficientBalance)]
    fun test_insufficient_balance_fail(account: &signer) {
        genesis::init_for_test();
        let alice = account::create_account_for_testing(ALICE);
        
        // Register coin type
        let coin1_info = register_coin<TestCoin1>(account, b"TestCoin1", b"TC1", 8);
        
        // Initialize store
        account_coin_store::do_accept_coin<TestCoin1>(&alice);
        
        // Deposit some coins
        let coin1 = mint_coin<TestCoin1>(&mut coin1_info, 500);
        account_coin_store::deposit<TestCoin1>(ALICE, coin1);
        
        // Attempt to withdraw more than balance
        let coin2 = account_coin_store::withdraw<TestCoin1>(&alice, 1000);
        // Should abort here

        // cleanup
        coin::unpack_for_test(coin2);
        object::transfer(coin1_info, @rooch_framework);
    }

    #[test(account = @0x40)]
    fun test_freeze_compatibility(account: &signer) {
        genesis::init_for_test();
        let alice = account::create_account_for_testing(ALICE);
        let bob = account::create_account_for_testing(BOB);
        
        // Register coin type
        let coin1_info = register_coin<TestCoin1>(account, b"TestCoin1", b"TC1", 8);
        
        // Setup accounts
        account_coin_store::do_accept_coin<TestCoin1>(&alice);
        account_coin_store::do_accept_coin<TestCoin1>(&bob);
        
        // Deposit funds
        let coin1 = mint_coin<TestCoin1>(&mut coin1_info, 1000);
        account_coin_store::deposit<TestCoin1>(ALICE, coin1);
        
        // Freeze Alice's account
        account_coin_store::freeze_extend<TestCoin1>(ALICE, true);
        
        // Verify frozen state in both stores
        assert!(account_coin_store::is_account_coin_store_frozen<TestCoin1>(ALICE), 1);
        
        let coin_type = type_info::type_name<TestCoin1>();
        assert!(account_coin_store::is_multi_coin_store_frozen_by_type_name(ALICE, coin_type), 2);
        
        // Unfreeze
        account_coin_store::freeze_extend<TestCoin1>(ALICE, false);
        
        // Verify unfrozen
        assert!(!account_coin_store::is_account_coin_store_frozen<TestCoin1>(ALICE), 3);
        assert!(!account_coin_store::is_multi_coin_store_frozen_by_type_name(ALICE, coin_type), 4);

        // cleanup
        object::transfer(coin1_info, @rooch_framework);
    }

    #[test(account = @0x40)]
    #[expected_failure(abort_code = multi_coin_store::ErrorCoinTypeShouldHaveKeyAndStoreAbility)]
    fun test_coin_has_only_key_compatibility_should_fail(account: &signer) {
        genesis::init_for_test();
        let alice = account::create_account_for_testing(ALICE);
        
        // Register TestCoin4 which only has key ability
        // We need to use a different register function since the helper requires key+store
        let coin4_info = register_coin<TestCoin4>(account, b"TestCoin4", b"TC4", 8);
        
        // Accept the coin - this should work with private_generics
        account_coin_store::do_accept_coin<TestCoin4>(&alice);
        
        // Verify stores are created correctly
        assert!(account_coin_store::exist_account_coin_store<TestCoin4>(ALICE), 1);
        assert!(account_coin_store::exist_multi_coin_store(ALICE), 2);
        
        // Get coin type string representation
        let coin_type = type_info::type_name<TestCoin4>();
        assert!(account_coin_store::exist_multi_coin_store_field(ALICE, coin_type), 3);
        
        // Mint coins using the extend variant which allows key-only types
        let amount = 1000;
        let coin4 = coin::mint_extend<TestCoin4>(&mut coin4_info, amount);
        
        // Deposit using extend variant
        account_coin_store::deposit_extend<TestCoin4>(ALICE, coin4);
        
        // Verify balance
        let alice_balance = account_coin_store::balance<TestCoin4>(ALICE);
        assert!(alice_balance == amount, 4);
        
        // Withdraw using extend variant
        let withdraw_amount = 500;
        let coin4 = account_coin_store::withdraw_extend<TestCoin4>(signer::address_of(&alice), withdraw_amount);
        
        // Verify balance after withdrawal
        alice_balance = account_coin_store::balance<TestCoin4>(ALICE);
        assert!(alice_balance == amount - withdraw_amount, 5);
        
        // Test freezing with key-only coin
        account_coin_store::freeze_extend<TestCoin4>(ALICE, true);
        assert!(account_coin_store::is_account_coin_store_frozen<TestCoin4>(ALICE), 6);
        assert!(account_coin_store::is_multi_coin_store_frozen_by_type_name(ALICE, coin_type), 7);

        // Unfreeze for cleanup
        account_coin_store::freeze_extend<TestCoin4>(ALICE, false);
        
        // Test deposit after unfreezing
        account_coin_store::deposit_extend<TestCoin4>(ALICE, coin4);
        
        // Verify final balance
        alice_balance = account_coin_store::balance<TestCoin4>(ALICE);
        assert!(alice_balance == amount, 8);

        // cleanup
        object::transfer(coin4_info, @rooch_framework);
    }

    // Helper functions for testing

    fun register_coin<CoinType: key>(
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