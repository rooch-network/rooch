// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

#[test_only]
/// This test module is used to test the gas coin
module rooch_framework::gas_coin_test{

    use rooch_framework::account as account_entry;
    use rooch_framework::coin;
    use rooch_framework::gas_coin::{Self, GasCoin};

    #[test]
    fun test_gas_coin_init(){
        let genesis_ctx = rooch_framework::genesis::init_for_test();
        assert!(coin::is_registered<GasCoin>(&genesis_ctx), 1000);
        moveos_std::context::drop_test_context(genesis_ctx);
    }

    #[test]
    fun test_gas_coin_mint(){
        let genesis_ctx = rooch_framework::genesis::init_for_test();
        let gas_coin = gas_coin::mint_for_test(&mut genesis_ctx, 1000u256);
        gas_coin::burn(&mut genesis_ctx, gas_coin);
        moveos_std::context::drop_test_context(genesis_ctx);
    }

    #[test(user = @0x42)]
    fun test_faucet(user: address){
        let genesis_ctx = rooch_framework::genesis::init_for_test();
        account_entry::create_account_for_test(&mut genesis_ctx, user);
        let init_gas = 9999u256;
        gas_coin::faucet_for_test(&mut genesis_ctx, user, init_gas); 
        std::debug::print(&gas_coin::balance(&genesis_ctx, user));
        assert!(gas_coin::balance(&genesis_ctx, user) == init_gas, 1000);
        moveos_std::context::drop_test_context(genesis_ctx);
    }

}
