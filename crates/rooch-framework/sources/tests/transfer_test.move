// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

#[test_only]
/// This test module is used to test the trasfer entry functions
module rooch_framework::transfer_test{

    use std::option;
    use rooch_framework::account;
    use rooch_framework::transfer;
    use rooch_framework::gas_coin::{Self, GasCoin};
    use rooch_framework::multichain_address::{Self, MultiChainAddress};
    use rooch_framework::ethereum_address;
    use rooch_framework::address_mapping;


    #[test(from = @0x42, to = @0x43)]
    fun test_transfer_coin(from: address, to: address){
        let genesis_ctx = rooch_framework::genesis::init_for_test();
        let from_signer = account::create_account_for_test(&mut genesis_ctx, from);
        let init_gas = 9999u256;
        gas_coin::faucet_for_test(&mut genesis_ctx, from, init_gas); 
        assert!(gas_coin::balance(&genesis_ctx, from) == init_gas, 1000);

        let amount = 11u256;
        transfer::transfer_coin<GasCoin>(&mut genesis_ctx, &from_signer, to, amount);

        assert!(gas_coin::balance(&genesis_ctx, from) == init_gas - amount, 1001);
        assert!(gas_coin::balance(&genesis_ctx, to) == amount, 1002);
        moveos_std::context::drop_test_context(genesis_ctx);
    }

    #[test_only]
    fun test_transfer_coin_to_multichain_address(from: address, to: MultiChainAddress){
        let genesis_ctx = rooch_framework::genesis::init_for_test();
        let from_signer = account::create_account_for_test(&mut genesis_ctx, from);
        let init_gas = 9999u256;
        gas_coin::faucet_for_test(&mut genesis_ctx, from, init_gas); 
        assert!(gas_coin::balance(&genesis_ctx, from) == init_gas, 1000);

        let amount = 11u256;
        transfer::transfer_coin_to_multichain_address<GasCoin>(&mut genesis_ctx, &from_signer, multichain_address::multichain_id(&to), *multichain_address::raw_address(&to), amount);

        let to_address_opt = address_mapping::resolve(&genesis_ctx, to);
        assert!(option::is_some(&to_address_opt), 1001);
        let to_address = option::extract(&mut to_address_opt);

        assert!(gas_coin::balance(&genesis_ctx, from) == init_gas - amount, 1002);
        assert!(gas_coin::balance(&genesis_ctx, to_address) == amount, 1003);

        //transfer again
        transfer::transfer_coin_to_multichain_address<GasCoin>(&mut genesis_ctx, &from_signer, multichain_address::multichain_id(&to), *multichain_address::raw_address(&to), amount);
        assert!(gas_coin::balance(&genesis_ctx, to_address) == amount*2, 1004);
        
        moveos_std::context::drop_test_context(genesis_ctx);
    }

     #[test(from = @0x42)]
    fun test_transfer_coin_to_eth_address(from: address){
        let eth_address = ethereum_address::from_bytes(x"1111111111111111111111111111111111111111");
        let multichain_address = multichain_address::from_eth(eth_address);
        test_transfer_coin_to_multichain_address(from, multichain_address);
    }
}
