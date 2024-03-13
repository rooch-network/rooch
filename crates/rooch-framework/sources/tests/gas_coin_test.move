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
        rooch_framework::genesis::init_for_test();
        assert!(coin::is_registered<GasCoin>(), 1000);
        
    }

    #[test]
    fun test_gas_coin_mint(){
        rooch_framework::genesis::init_for_test();
        let gas_coin = gas_coin::mint_for_test(1000u256);
        gas_coin::burn(gas_coin);
        
    }

    #[test(user = @0x42)]
    fun test_faucet(user: address){
        rooch_framework::genesis::init_for_test();
        account_entry::create_account_for_testing(user);
        let init_gas = 9999u256;
        gas_coin::faucet_for_test(user, init_gas); 
        std::debug::print(&gas_coin::balance(user));
        assert!(gas_coin::balance(user) == init_gas, 1000);
        
    }

}
