// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

#[test_only]
module rooch_framework::multi_coin_store_tests {
    use std::string;
    use std::option;
    use rooch_framework::coin::{convert_coin_to_generic_coin, register_extend};
    use rooch_framework::coin;
    use rooch_framework::multi_coin_store::MultiCoinStore;
    use rooch_framework::multi_coin_store;
    use rooch_framework::genesis;
    use moveos_std::signer;
    use moveos_std::account;
    use moveos_std::object;

    // Test coin types
    struct FakeCoin has key, store {}

    struct BTC has key, store {}

    // Coin type with no store ability
    struct PrivateCoin has key {}

    #[test(account = @0x42)]
    fun test_create_multi_coin_store(account: &signer) {
        genesis::init_for_test();
        let addr = signer::address_of(account);
        let _test_account = account::create_account_for_testing(addr);

        // Create multi coin store
        let store_id = multi_coin_store::create_multi_coin_store_for_test(addr);

        // Verify the store exists
        assert!(object::exists_object_with_type<MultiCoinStore>(store_id), 1);
    }

    #[test(account = @0x42)]
    fun test_multi_coin_store_field_operations(account: &signer) {
        genesis::init_for_test();
        let addr = signer::address_of(account);
        let _test_account = account::create_account_for_testing(addr);

        // Register coin types
        let btc_info = register_extend<BTC>(string::utf8(b"Bitcoin"), string::utf8(b"BTC"), option::none(), 8);
        // let _fakecoin_info = register_extend<BTC>(string::utf8(b"FakeCoin"), string::utf8(b"FC"), option::none(), 9);
        object::transfer(btc_info, @rooch_framework);

        // Create multi coin store
        let store_id = multi_coin_store::create_multi_coin_store_for_test(addr);
        let store_obj = object::borrow_object<MultiCoinStore>(store_id);

        // Test initially empty
        let btc_type = string::utf8(b"0x0000000000000000000000000000000000000000000000000000000000000003::multi_coin_store_tests::BTC");
        // let fakecoin_type = string::utf8(b"0x0000000000000000000000000000000000000000000000000000000000000003::multi_coin_store_tests::FakeCoin");

        assert!(multi_coin_store::balance(store_obj, btc_type) == 0, 1);
        assert!(!multi_coin_store::exist_coin_store_field(store_obj, btc_type), 2);

        // Create coin store field
        let store_obj_mut = object::borrow_mut_object<MultiCoinStore>(account, store_id);
        multi_coin_store::create_coin_store_field_if_not_exist_for_test(store_obj_mut, btc_type);

        // Verify field exists now
        assert!(multi_coin_store::exist_coin_store_field(store_obj_mut, btc_type), 3);
        assert!(multi_coin_store::balance(store_obj_mut, btc_type) == 0, 4);
        assert!(!multi_coin_store::is_frozen(store_obj_mut, btc_type), 5);
    }

    #[test(account = @0x42)]
    fun test_deposit_withdraw(account: &signer) {
        genesis::init_for_test();
        let addr = signer::address_of(account);
        let _test_account = account::create_account_for_testing(addr);

        // Register coin types
        let btc_info = register_extend<BTC>(string::utf8(b"Bitcoin"), string::utf8(b"BTC"), option::none(), 8);

        // Create multi coin store
        let store_id = multi_coin_store::create_multi_coin_store_for_test(addr);
        let store_obj_mut = object::borrow_mut_object<MultiCoinStore>(account, store_id);

        // Get coin types
        let btc_type = string::utf8(b"0x0000000000000000000000000000000000000000000000000000000000000003::multi_coin_store_tests::BTC");

        // Mint some coins
        let btc_coin = mint_coin<BTC>(&mut btc_info, 100);
        let _btc_value = coin::value(&btc_coin);
        let btc_generic = coin::convert_coin_to_generic_coin(btc_coin);

        // Deposit BTC
        multi_coin_store::deposit(store_obj_mut, btc_generic);

        // Verify balance
        assert!(multi_coin_store::balance(store_obj_mut, btc_type) == 100, 1);
        // assert!(multi_coin_store::balance(store_obj_mut, btc_type) == 100, 1);

        // Withdraw half
        let withdrawn_coin = multi_coin_store::withdraw(store_obj_mut, btc_type, 50);
        // let withdrawn_coin = multi_coin_store::withdraw(store_obj_mut, btc_type, 50);

        // Verify balance reduced
        assert!(multi_coin_store::balance(store_obj_mut, btc_type) == 50, 2);

        // Verify withdrawn coin
        assert!(coin::coin_type(&withdrawn_coin) == btc_type, 3);
        assert!(coin::generic_coin_value(&withdrawn_coin) == 50, 4);

        coin::unpack_generic_coin_for_test(withdrawn_coin);
        object::transfer(btc_info, @rooch_framework);
    }

    #[test(account = @0x42)]
    #[expected_failure(abort_code = multi_coin_store::ErrorInsufficientBalance)]
    fun test_withdraw_insufficient_balance(account: &signer) {
        genesis::init_for_test();
        let addr = signer::address_of(account);
        let _test_account = account::create_account_for_testing(addr);
        
        // Register coin type
        let btc_info = register_extend<BTC>(string::utf8(b"Bitcoin"), string::utf8(b"BTC"), option::none(), 8);
        // object::transfer(btc_info, @rooch_framework);
        
        // Create multi coin store
        let store_id = multi_coin_store::create_multi_coin_store_for_test(addr);
        let store_obj_mut = object::borrow_mut_object<MultiCoinStore>(account, store_id);
        
        // Get coin type
        let btc_type = string::utf8(b"0x0000000000000000000000000000000000000000000000000000000000000003::multi_coin_store_tests::BTC");
        
        // Mint and deposit some coins
        let btc_coin = mint_coin<BTC>(&mut btc_info, 100);
        let btc_generic = convert_coin_to_generic_coin(btc_coin);
        multi_coin_store::deposit(store_obj_mut, btc_generic);
        
        // Try to withdraw more than balance - should fail
        let generic_coin = multi_coin_store::withdraw(store_obj_mut, btc_type, 200);
        coin::unpack_generic_coin_for_test(generic_coin);
        object::transfer(btc_info, @rooch_framework);
    }

    #[test(account = @0x42)]
    #[expected_failure(abort_code = multi_coin_store::ErrorCoinStoreIsFrozen)]
    fun test_frozen_coin_store(account: &signer) {
        genesis::init_for_test();
        let addr = signer::address_of(account);
        let _test_account = account::create_account_for_testing(addr);
        
        // Register coin type
        let btc_info = register_extend<BTC>(string::utf8(b"Bitcoin"), string::utf8(b"BTC"), option::none(), 8);

        // Create multi coin store
        let store_id = multi_coin_store::create_multi_coin_store_for_test(addr);
        let store_obj_mut = object::borrow_mut_object<MultiCoinStore>(account, store_id);
        
        // Get coin type
        let btc_type = string::utf8(b"0x0000000000000000000000000000000000000000000000000000000000000003::multi_coin_store_tests::BTC");
        
        // Create the coin store field
        multi_coin_store::create_coin_store_field_if_not_exist_for_test(store_obj_mut, btc_type);
        
        // freeze the coin store field
        multi_coin_store::freeze_coin_store_extend<BTC>(store_obj_mut, true);
        
        // Try to deposit - should fail because store is frozen
        let btc_coin = mint_coin<BTC>(&mut btc_info, 100);
        let btc_generic = convert_coin_to_generic_coin(btc_coin);
        multi_coin_store::deposit(store_obj_mut, btc_generic);

        object::transfer(btc_info, @rooch_framework);
    }

    #[test(account = @0x42)]
    #[expected_failure(abort_code = multi_coin_store::ErrorCoinTypeShouldHaveKeyAndStoreAbility)]
    fun test_private_coin_type(account: &signer) {
        genesis::init_for_test();
        let addr = signer::address_of(account);
        let _test_account = account::create_account_for_testing(addr);

        // Register coin type
        let private_coin_info = register_extend<PrivateCoin>(string::utf8(b"PrivateCoin"), string::utf8(b"PC"), option::none(), 8);
        
        // Create multi coin store
        let store_id = multi_coin_store::create_multi_coin_store_for_test(addr);
        let store_obj_mut = object::borrow_mut_object<MultiCoinStore>(account, store_id);
        
        // Try to operate with a coin type that doesn't have store ability
        let private_coin_type = string::utf8(b"0x0000000000000000000000000000000000000000000000000000000000000003::multi_coin_store_tests::PrivateCoin");
        multi_coin_store::create_coin_store_field_if_not_exist_for_test(store_obj_mut, private_coin_type);

        let private_coin = coin::mint_extend<PrivateCoin>(&mut private_coin_info, 100);
        let private_coin_generic = coin::convert_coin_to_generic_coin(private_coin);
        // This should fail because PrivateCoin doesn't have the store ability
        multi_coin_store::deposit(store_obj_mut, private_coin_generic);

        object::transfer(private_coin_info, @rooch_framework);
    }

    #[test(account = @0x42)]
    fun test_multiple_coin_types(account: &signer) {
        genesis::init_for_test();
        let addr = signer::address_of(account);
        let _test_account = account::create_account_for_testing(addr);
        
        // Register coin types
        let btc_info = register_extend<BTC>(string::utf8(b"Bitcoin"), string::utf8(b"BTC"), option::none(), 8);
        let fakecoin_info = register_extend<FakeCoin>(string::utf8(b"FakeCoin"), string::utf8(b"FC"), option::none(), 9);

        // Create multi coin store
        let store_id = multi_coin_store::create_multi_coin_store_for_test(addr);
        let store_obj_mut = object::borrow_mut_object<MultiCoinStore>(account, store_id);
        
        // Get coin types
        let btc_type = string::utf8(b"0x0000000000000000000000000000000000000000000000000000000000000003::multi_coin_store_tests::BTC");
        let fakecoin_type = string::utf8(b"0x0000000000000000000000000000000000000000000000000000000000000003::multi_coin_store_tests::FakeCoin");
        
        // Mint and deposit different coins
        let btc_coin = mint_coin<BTC>(&mut btc_info, 100);
        let btc_generic = convert_coin_to_generic_coin(btc_coin);
        multi_coin_store::deposit(store_obj_mut, btc_generic);
        
        let fake_coin = mint_coin<FakeCoin>(&mut fakecoin_info, 200);
        let fakecoin_generic = convert_coin_to_generic_coin(fake_coin);
        multi_coin_store::deposit(store_obj_mut, fakecoin_generic);
        
        // Verify all balances
        assert!(multi_coin_store::balance(store_obj_mut, btc_type) == 100, 1);
        assert!(multi_coin_store::balance(store_obj_mut, fakecoin_type) == 200, 2);

        // Withdraw some of each
        let withdrawn_btc = multi_coin_store::withdraw(store_obj_mut, btc_type, 50);
        let withdrawn_fakecoin = multi_coin_store::withdraw(store_obj_mut, fakecoin_type, 100);
        
        // Verify updated balances
        assert!(multi_coin_store::balance(store_obj_mut, btc_type) == 50, 4);
        assert!(multi_coin_store::balance(store_obj_mut, fakecoin_type) == 100, 5);

        coin::unpack_generic_coin_for_test(withdrawn_btc);
        coin::unpack_generic_coin_for_test(withdrawn_fakecoin);
        object::transfer(btc_info, @rooch_framework);
        object::transfer(fakecoin_info, @rooch_framework);
    }

    #[test(account = @0x42)]
    fun test_remove_coin_store_field(account: &signer) {
        genesis::init_for_test();
        let addr = signer::address_of(account);
        let _test_account = account::create_account_for_testing(addr);
        
        // Register coin type
        let btc_info = register_extend<BTC>(string::utf8(b"Bitcoin"), string::utf8(b"BTC"), option::none(), 8);

        // Create multi coin store
        let store_id = multi_coin_store::create_multi_coin_store_for_test(addr);
        let store_obj_mut = object::borrow_mut_object<MultiCoinStore>(account, store_id);
        
        // Get coin type
        let btc_type = string::utf8(b"0x0000000000000000000000000000000000000000000000000000000000000003::multi_coin_store_tests::BTC");
        
        // Mint and deposit some coins
        let btc_coin = mint_coin<BTC>(&mut btc_info, 100);
        let btc_generic = convert_coin_to_generic_coin(btc_coin);
        multi_coin_store::deposit(store_obj_mut, btc_generic);
        
        // Check balance
        assert!(multi_coin_store::balance(store_obj_mut, btc_type) == 100, 1);
        
        // Remove the coin store field
        let removed_coin = multi_coin_store::remove_coin_store_field(store_obj_mut, btc_type);
        
        // Verify coin returned from removal
        assert!(coin::coin_type(&removed_coin) == btc_type, 2);
        assert!(coin::generic_coin_value(&removed_coin) == 100, 3);
        
        // Verify field is removed
        assert!(!multi_coin_store::exist_coin_store_field(store_obj_mut, btc_type), 4);
        assert!(multi_coin_store::balance(store_obj_mut, btc_type) == 0, 5);

        coin::unpack_generic_coin_for_test(removed_coin);
        object::transfer(btc_info, @rooch_framework);
    }

    // Helper functions for testing
    fun mint_coin<CoinType: key + store>(
        coin_info: &mut object::Object<coin::CoinInfo<CoinType>>,
        amount: u256
    ): coin::Coin<CoinType> {
        coin::mint<CoinType>(coin_info, amount)
    }
} 