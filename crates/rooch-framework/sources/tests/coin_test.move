// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

#[test_only]
/// This test module is used to test the coin logic in coin and account module.
module rooch_framework::coin_test{
    use std::string;
    
    use moveos_std::object::{Self, Object};
    use rooch_framework::coin;
    use rooch_framework::coin::{register_extend,
        supply, name, symbol, decimals, value, mint_extend, burn_extend, zero, destroy_zero, is_registered, extract
    };
 

    #[test_only]
    struct FakeCoin has key, store {}

    #[test_only]
    fun register_fake_coin(
        
        decimals: u8,
    ) : Object<coin::CoinInfo<FakeCoin>> {
        coin::register_extend<FakeCoin>(
            string::utf8(b"Fake coin"),
            string::utf8(b"FCD"),
            decimals,
        )
    }

    #[test]
    fun test_end_to_end(
    ) {
        rooch_framework::genesis::init_for_test();

        let name = string::utf8(b"Fake coin");
        let symbol = string::utf8(b"FCD");
        let decimals = 9u8;

        let coin_info_obj = register_extend<FakeCoin>(
            
            name,
            symbol,
            decimals,
        );
        {
            let coin_info = object::borrow(&coin_info_obj);
            assert!(supply<FakeCoin>(coin_info) == 0, 0);
            assert!(name<FakeCoin>(coin_info) == name, 1);
            assert!(symbol<FakeCoin>(coin_info) == symbol, 2);
            assert!(decimals<FakeCoin>(coin_info) == decimals, 3);
        };

        let coins_minted = mint_extend<FakeCoin>(&mut coin_info_obj, 100);
        
        assert!(supply<FakeCoin>(object::borrow(&coin_info_obj)) == 100, 4);

        let coins_minted2 = mint_extend<FakeCoin>(&mut coin_info_obj, 100);

        assert!(supply<FakeCoin>(object::borrow(&coin_info_obj)) == 200, 5);
        
        let coin = extract(&mut coins_minted, 50);

        burn_extend(&mut coin_info_obj, coin);
        assert!(supply<FakeCoin>(object::borrow(&coin_info_obj)) == 150, 6);

        burn_extend(&mut coin_info_obj, coins_minted); 
        burn_extend(&mut coin_info_obj, coins_minted2); 

        assert!(supply<FakeCoin>(object::borrow(&coin_info_obj)) == 0, 7);
        object::transfer(coin_info_obj, @rooch_framework);
        
    }

    #[test]
    #[expected_failure(abort_code = 2, location = rooch_framework::coin)]
    public fun fail_register() {
        rooch_framework::genesis::init_for_test();

        let coin_info_obj = register_extend<FakeCoin>(
            
            string::utf8(b"Fake coin"),
            string::utf8(b"FCD"),
            9,
        );
        object::transfer(coin_info_obj, @rooch_framework);

        let coin_info_obj = register_extend<FakeCoin>(
            
            string::utf8(b"Fake coin"),
            string::utf8(b"FCD"),
            9,
        );
        object::transfer(coin_info_obj, @rooch_framework);
        
    }

    #[test]
    #[expected_failure(abort_code = 4, location = rooch_framework::coin)]
    public fun test_destroy_non_zero(
    ) {
        rooch_framework::genesis::init_for_test();

        let coin_info_obj = register_fake_coin(9);
        let coins_minted = mint_extend<FakeCoin>(&mut coin_info_obj, 100);
        destroy_zero(coins_minted);
        object::transfer(coin_info_obj, @rooch_framework);
        
    }


    #[test]
    fun test_test_extract() {
        rooch_framework::genesis::init_for_test();
        let coin_info_obj = register_fake_coin(9);
        let coins_minted = mint_extend<FakeCoin>(&mut coin_info_obj, 100);

        let extracted = extract(&mut coins_minted, 25);
        assert!(value(&coins_minted) == 75, 0);
        assert!(value(&extracted) == 25, 1);

        burn_extend(&mut coin_info_obj, coins_minted);
        burn_extend(&mut coin_info_obj, extracted);
        object::transfer(coin_info_obj, @rooch_framework);
        
    }


    #[test]
    public fun test_is_registered() {
        rooch_framework::genesis::init_for_test();
        assert!(!is_registered<FakeCoin>(), 0);

        let coin_info_obj = register_fake_coin(9);
        object::transfer(coin_info_obj, @rooch_framework);
        assert!(is_registered<FakeCoin>(), 1);
        
    }

    #[test]
    fun test_zero() {
        let zero = zero<FakeCoin>();
        assert!(value(&zero) == 0, 1);
        destroy_zero(zero);
    }
    
}
