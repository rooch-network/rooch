// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

#[test_only]
/// This test module is used to test the coin logic in coin and account module.
module rooch_framework::coin_test{
    use std::string;
    use std::option;
    use moveos_std::object::{Self, Object};
    use rooch_framework::coin::{Self, TreasuryCap};

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

    #[test]
    fun test_end_to_end() {
        rooch_framework::genesis::init_for_test();

        let name = string::utf8(b"Fake coin");
        let symbol = string::utf8(b"FCD");
        let decimals = 9u8;

        let treasury_cap = coin::register_extend_v2<FakeCoin>(
            name,
            symbol,
            option::none(),
            decimals,
        );
        {
            let coin_info = coin::coin_info<FakeCoin>();
            assert!(coin::supply<FakeCoin>(coin_info) == 0, 0);
            assert!(coin::name<FakeCoin>(coin_info) == name, 1);
            assert!(coin::symbol<FakeCoin>(coin_info) == symbol, 2);
            assert!(coin::decimals<FakeCoin>(coin_info) == decimals, 3);
        };

        let coins_minted = coin::mint_extend_by_cap<FakeCoin>(&mut treasury_cap, 100);
        
        assert!(coin::supply_by_type<FakeCoin>() == 100, 4);

        let coins_minted2 = coin::mint_extend_by_cap<FakeCoin>(&mut treasury_cap, 100);

        assert!(coin::supply_by_type<FakeCoin>() == 200, 5);
        
        let coin = coin::extract(&mut coins_minted, 50);

        coin::burn_extend_by_cap(&mut treasury_cap, coin);
        assert!(coin::supply_by_type<FakeCoin>() == 150, 6);

        coin::burn_extend_by_cap(&mut treasury_cap, coins_minted); 
        coin::burn_extend_by_cap(&mut treasury_cap, coins_minted2); 

        assert!(coin::supply_by_type<FakeCoin>() == 0, 7);
        object::transfer(treasury_cap, @rooch_framework);
    }

    #[test]
    #[expected_failure(abort_code = 2, location = rooch_framework::coin)]
    public fun fail_register() {
        rooch_framework::genesis::init_for_test();

        let treasury_cap = coin::register_extend_v2<FakeCoin>(
            string::utf8(b"Fake coin"),
            string::utf8(b"FCD"),
            option::none(),
            9,
        );
        object::transfer(treasury_cap, @rooch_framework);

        let treasury_cap = coin::register_extend_v2<FakeCoin>(
            string::utf8(b"Fake coin"),
            string::utf8(b"FCD"),
            option::none(),
            9,
        );
        object::transfer(treasury_cap, @rooch_framework);
    }

    #[test]
    #[expected_failure(abort_code = 4, location = rooch_framework::coin)]
    public fun test_destroy_non_zero() {
        rooch_framework::genesis::init_for_test();

        let treasury_cap = register_fake_coin(9);
        let coins_minted = coin::mint_extend_by_cap<FakeCoin>(&mut treasury_cap, 100);
        coin::destroy_zero(coins_minted);
        object::transfer(treasury_cap, @rooch_framework);
    }

    #[test]
    fun test_extract() {
        rooch_framework::genesis::init_for_test();
        let treasury_cap = register_fake_coin(9);
        let coins_minted = coin::mint_extend_by_cap<FakeCoin>(&mut treasury_cap, 100);

        let extracted = coin::extract(&mut coins_minted, 25);
        assert!(coin::value(&coins_minted) == 75, 0);
        assert!(coin::value(&extracted) == 25, 1);

        coin::burn_extend_by_cap(&mut treasury_cap, coins_minted);
        coin::burn_extend_by_cap(&mut treasury_cap, extracted);
        object::transfer(treasury_cap, @rooch_framework);
    }

    #[test]
    public fun test_is_registered() {
        rooch_framework::genesis::init_for_test();
        assert!(!coin::is_registered<FakeCoin>(), 0);

        let treasury_cap = register_fake_coin(9);
        object::transfer(treasury_cap, @rooch_framework);
        assert!(coin::is_registered<FakeCoin>(), 1);
    }

    #[test]
    fun test_zero() {
        let zero = coin::zero<FakeCoin>();
        assert!(coin::value(&zero) == 0, 1);
        coin::destroy_zero(zero);
    }
}
