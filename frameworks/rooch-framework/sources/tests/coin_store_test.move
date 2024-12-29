// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

#[test_only]
/// This test module is used to test the coin store logic
module rooch_framework::coin_store_test{
    use std::option;
    use std::signer;
    use std::string;
    
    use moveos_std::object::{Self, Object};
    use rooch_framework::coin::{Self, TreasuryCap};
    use rooch_framework::account_coin_store;
    use rooch_framework::coin_store;
    use rooch_framework::account as account_entry;

    #[test_only]
    struct FakeCoin has key, store {}

    #[test_only]
    fun register_fake_coin(
        decimals: u8,
    ) : Object<TreasuryCap<FakeCoin>> {
        coin::register_extend_v2<FakeCoin>(
            string::utf8(b"Fake coin"),
            string::utf8(b"FCD"),
            option::none(),
            decimals,
        )
    }

    #[test_only]
    fun mint_and_deposit(treasury_cap: &mut Object<TreasuryCap<FakeCoin>>, to_address: address, amount: u256) {
        let coins_minted = coin::mint_extend_by_cap<FakeCoin>(treasury_cap, amount);
        account_coin_store::deposit(to_address, coins_minted);
    }

    #[test_only]
    fun freeze_account_coin_store(
        addr: address,
        frozen: bool,
    ) {
        let coin_store_id = account_coin_store::account_coin_store_id<FakeCoin>(addr);
        let coin_store_obj = coin_store::borrow_mut_coin_store_extend<FakeCoin>(coin_store_id);
        coin_store::freeze_coin_store_extend<FakeCoin>(coin_store_obj, frozen);
    }

    #[test]
    public fun test_coin_store(){
        rooch_framework::genesis::init_for_test();
        let treasury_cap = register_fake_coin(9);
        let coin_minted = coin::mint_extend_by_cap<FakeCoin>(&mut treasury_cap, 100);

        let coin_store_obj = coin_store::create_coin_store<FakeCoin>();
    
        coin_store::deposit(&mut coin_store_obj, coin_minted);

        assert!(coin_store::balance(&coin_store_obj) == 100, 1);

        let coin_withdrawn = coin_store::withdraw(&mut coin_store_obj, 10);

        assert!(coin::value(&coin_withdrawn) == 10, 2);
        assert!(coin_store::balance(&coin_store_obj) == 90, 3);
        coin::burn_extend_by_cap(&mut treasury_cap, coin_withdrawn);
        
        let coin = coin_store::remove_coin_store<FakeCoin>(coin_store_obj);
        assert!(coin::value(&coin) == 90, 4);
        coin::burn_extend_by_cap(&mut treasury_cap, coin);
        assert!(coin::supply_by_type<FakeCoin>() == 0, 5);
        object::transfer(treasury_cap, @rooch_framework);
    }

    #[test(account = @0x42)]
    public fun test_is_account_coin_store_frozen(account: signer) {
        rooch_framework::genesis::init_for_test();
        let addr = signer::address_of(&account);
        
        let treasury_cap = register_fake_coin(9);

        // An non do_accept_coined account is has a no frozen coin store by default
        assert!(!account_coin_store::is_account_coin_store_frozen<FakeCoin>(addr), 1);
        
        account_entry::create_account_for_testing(addr);
        mint_and_deposit(&mut treasury_cap, addr, 100);

        // freeze account
        freeze_account_coin_store(addr, true);
        assert!(account_coin_store::is_account_coin_store_frozen<FakeCoin>(addr), 2);

        // unfreeze account
        freeze_account_coin_store(addr, false);
        assert!(!account_coin_store::is_account_coin_store_frozen<FakeCoin>(addr), 3);
        
        object::transfer(treasury_cap, @rooch_framework);
    }

    #[test(account = @0x42)]
    #[expected_failure(abort_code = 2, location = rooch_framework::coin_store)]
    fun test_withdraw_from_account_frozen(account: signer) {
        rooch_framework::genesis::init_for_test();
        let account_addr = signer::address_of(&account);
        account_entry::create_account_for_testing(account_addr);
        let treasury_cap = register_fake_coin(9);

        mint_and_deposit(&mut treasury_cap, account_addr, 100);
        freeze_account_coin_store(account_addr, true);
        let coin = account_coin_store::withdraw(&account, 10);
        coin::burn_extend_by_cap(&mut treasury_cap, coin);
        object::transfer(treasury_cap, @rooch_framework);
    }

    #[test(account = @0x42)]
    #[expected_failure(abort_code = 2, location = rooch_framework::coin_store)]
    fun test_deposit_to_account_frozen(account: signer) {
        rooch_framework::genesis::init_for_test();
        let account_addr = signer::address_of(&account);
        account_entry::create_account_for_testing(account_addr);

        let treasury_cap = register_fake_coin(9);

        mint_and_deposit(&mut treasury_cap, account_addr, 100);
        let coins_minted = coin::mint_extend_by_cap<FakeCoin>(&mut treasury_cap, 100);
        freeze_account_coin_store(account_addr, true);
        account_coin_store::deposit(account_addr, coins_minted);

        object::transfer(treasury_cap, @rooch_framework);
    }

    #[test(account = @0x42)]
    fun test_deposit_widthdraw_unfrozen(account: signer) {
        rooch_framework::genesis::init_for_test();
        let account_addr = signer::address_of(&account);

        account_entry::create_account_for_testing(account_addr);
        let treasury_cap = register_fake_coin(9);
        mint_and_deposit(&mut treasury_cap, account_addr, 100);

        let coins_minted = coin::mint_extend_by_cap<FakeCoin>(&mut treasury_cap, 100);
        freeze_account_coin_store(account_addr, true);
        freeze_account_coin_store(account_addr, false);
        account_coin_store::deposit(account_addr, coins_minted);

        freeze_account_coin_store(account_addr, true);
        freeze_account_coin_store(account_addr, false);
        let coin = account_coin_store::withdraw<FakeCoin>(&account, 10);
        coin::burn_extend_by_cap(&mut treasury_cap, coin);
        object::transfer(treasury_cap, @rooch_framework);
    }
}
