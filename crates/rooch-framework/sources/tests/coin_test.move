// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

#[test_only]
/// This test module is used to test the coin logic in coin and account module.
module rooch_framework::coin_test{
    use std::string;
    use moveos_std::context::{Context};
    use rooch_framework::coin;
    use rooch_framework::coin::{register_extend,
        supply, name, symbol, decimals, value, mint_extend, burn_extend, zero, destroy_zero, is_registered, extract
    };
 

    #[test_only]
    struct FakeCoin has key, store {}

    #[test_only]
    fun register_fake_coin(
        ctx: &mut Context,
        decimals: u8,
    ) : &mut coin::CoinInfo<FakeCoin> {
        coin::register_extend<FakeCoin>(
            ctx,
            string::utf8(b"Fake coin"),
            string::utf8(b"FCD"),
            decimals,
        )
    }

    #[test]
    fun test_end_to_end(
    ) {
        let source_ctx = rooch_framework::genesis::init_for_test();

        let name = string::utf8(b"Fake coin");
        let symbol = string::utf8(b"FCD");
        let decimals = 9u8;

        let coin_info = register_extend<FakeCoin>(
            &mut source_ctx,
            name,
            symbol,
            decimals,
        );
        
        assert!(supply<FakeCoin>(coin_info) == 0, 0);

        assert!(name<FakeCoin>(coin_info) == name, 1);
        assert!(symbol<FakeCoin>(coin_info) == symbol, 2);
        assert!(decimals<FakeCoin>(coin_info) == decimals, 3);

        let coins_minted = mint_extend<FakeCoin>(coin_info, 100);
        
        assert!(supply<FakeCoin>(coin_info) == 100, 4);

        let coins_minted2 = mint_extend<FakeCoin>(coin_info, 100);

        assert!(supply<FakeCoin>(coin_info) == 200, 5);
        
        let coin = extract(&mut coins_minted, 50);

        burn_extend(coin_info, coin);
        assert!(supply<FakeCoin>(coin_info) == 150, 6);

        burn_extend(coin_info, coins_minted); 
        burn_extend(coin_info, coins_minted2); 

        assert!(supply<FakeCoin>(coin_info) == 0, 7);

        moveos_std::context::drop_test_context(source_ctx);
    }

    #[test]
    #[expected_failure(abort_code = 524290, location = rooch_framework::coin)]
    public fun fail_register() {
        let source_ctx = rooch_framework::genesis::init_for_test();

        register_extend<FakeCoin>(
            &mut source_ctx,
            string::utf8(b"Fake coin"),
            string::utf8(b"FCD"),
            9,
        );

        register_extend<FakeCoin>(
            &mut source_ctx,
            string::utf8(b"Fake coin"),
            string::utf8(b"FCD"),
            9,
        );

        moveos_std::context::drop_test_context(source_ctx);
    }

    #[test]
    #[expected_failure(abort_code = 65540, location = rooch_framework::coin)]
    public fun test_destroy_non_zero(
    ) {
        let source_ctx = rooch_framework::genesis::init_for_test();

        let coin_info = register_fake_coin(&mut source_ctx, 9);
        let coins_minted = mint_extend<FakeCoin>(coin_info, 100);
        destroy_zero(coins_minted);

        moveos_std::context::drop_test_context(source_ctx);
    }


    #[test]
    fun test_test_extract() {
        let source_ctx = rooch_framework::genesis::init_for_test();
        let coin_info = register_fake_coin(&mut source_ctx, 9);
        let coins_minted = mint_extend<FakeCoin>(coin_info, 100);

        let extracted = extract(&mut coins_minted, 25);
        assert!(value(&coins_minted) == 75, 0);
        assert!(value(&extracted) == 25, 1);

        burn_extend(coin_info, coins_minted);
        burn_extend(coin_info, extracted);

        moveos_std::context::drop_test_context(source_ctx);
    }


    #[test]
    public fun test_is_registered() {
        let ctx = rooch_framework::genesis::init_for_test();
        assert!(!is_registered<FakeCoin>(&ctx), 0);

        register_fake_coin(&mut ctx, 9);
        assert!(is_registered<FakeCoin>(&ctx), 1);
        moveos_std::context::drop_test_context(ctx);
    }

    #[test]
    fun test_zero() {
        let zero = zero<FakeCoin>();
        assert!(value(&zero) == 0, 1);
        destroy_zero(zero);
    }
    
}
