// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

#[test_only]
/// This test module is used to test the transfer entry functions
module rooch_framework::transfer_test{

    use std::string;
    use std::option;
    use moveos_std::object::{Self, Object};
    use rooch_framework::transfer;
    use rooch_framework::gas_coin::{Self, RGas};
    use rooch_framework::multichain_address::{Self, MultiChainAddress};
    use rooch_framework::bitcoin_address;
    use rooch_framework::address_mapping;
    use rooch_framework::coin::{Self, TreasuryCap};
    use rooch_framework::account_coin_store;
    use rooch_framework::account as account_entry;

    struct TestStruct has key, store{
        value: u64,
    }

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
    fun init_from_account(from: address): signer {
        rooch_framework::genesis::init_for_test();
        let from_signer = account_entry::create_account_for_testing(from);
        let init_gas = 9999u256;
        gas_coin::faucet_for_test(from, init_gas); 
        assert!(gas_coin::balance(from) == init_gas, 1000);
        from_signer
    }

    #[test(from = @0x42, to = @0x43)]
    fun test_transfer_coin(from: address, to: address){
        let from_signer = init_from_account(from);

        let original_balance = gas_coin::balance(from);
        let amount = 11u256;
        transfer::transfer_coin<RGas>(&from_signer, to, amount);

        assert!(gas_coin::balance(from) == original_balance - amount, 1001);
        assert!(gas_coin::balance(to) == amount, 1002);
        
    }

    #[test_only]
    fun transfer_coin_to_multichain_address(from: address, to: MultiChainAddress){
        let from_signer = init_from_account(from);
        
        let to_address_opt = address_mapping::resolve(to);
        let to_address = option::destroy_some(to_address_opt);

        let original_balance = gas_coin::balance(from);
        let amount = 11u256;
        transfer::transfer_coin_to_multichain_address<RGas>(&from_signer, multichain_address::multichain_id(&to), *multichain_address::raw_address(&to), amount);

        assert!(gas_coin::balance(from) == original_balance - amount, 1002);
        assert!(gas_coin::balance(to_address) == amount, 1003);

        //transfer again
        transfer::transfer_coin_to_multichain_address<RGas>(&from_signer, multichain_address::multichain_id(&to), *multichain_address::raw_address(&to), amount);
        assert!(gas_coin::balance(to_address) == amount*2, 1004);        
    }

    #[test(from = @0x42)]
    fun test_transfer_coin_to_multichain_address(from: address){
        let btc_address = bitcoin_address::from_string(&std::string::utf8(b"bc1q9ymlna2efqx5arvcszu633rzfxq77ce9c3z34l"));
        let multichain_address = multichain_address::from_bitcoin(btc_address);
        transfer_coin_to_multichain_address(from, multichain_address);
    }

    #[test(from = @0x42)]
    fun test_transfer_coin_to_bitcoin_address(from: address){
        let from_signer = init_from_account(from);
        let bitcoin_address_str = std::string::utf8(b"bc1q9ymlna2efqx5arvcszu633rzfxq77ce9c3z34l");
        let btc_address = bitcoin_address::from_string(&bitcoin_address_str);
        let to_rooch_address = bitcoin_address::to_rooch_address(&btc_address);

        let original_balance = gas_coin::balance(from);
        transfer::transfer_coin_to_bitcoin_address<RGas>(&from_signer, bitcoin_address_str, 11u256);
        assert!(gas_coin::balance(from) == original_balance - 11u256, 1002);
        assert!(gas_coin::balance(to_rooch_address) == 11u256, 1003);

        let opt_addr = address_mapping::resolve_bitcoin(to_rooch_address);
        assert!(option::is_some(&opt_addr), 1004);
        let addr = option::destroy_some(opt_addr);
        assert!(addr == btc_address, 1005);
    }

    #[test(from_addr= @0x33, to_addr= @0x66)]
    fun test_transfer_fake_coin(from_addr: address, to_addr: address) {
        rooch_framework::genesis::init_for_test();
        let treasury_cap = register_fake_coin(9);

        let from = account_entry::create_account_for_testing(from_addr);
        let _ = account_entry::create_account_for_testing(to_addr);

        let amount = 100u256;
        mint_and_deposit(&mut treasury_cap, from_addr, amount);
        transfer::transfer_coin<FakeCoin>(&from, to_addr, 50u256);

        assert!(account_coin_store::balance<FakeCoin>(to_addr) == 50u256, 1001);
        object::transfer(treasury_cap, @rooch_framework);
    }

    #[test(from_addr= @0x33, to_addr= @0x66)]
    fun test_transfer_object(from_addr: address, to_addr: address) {
        rooch_framework::genesis::init_for_test();
      
        let from = account_entry::create_account_for_testing(from_addr);
        let _ = account_entry::create_account_for_testing(to_addr);
        let obj = object::new(TestStruct{value: 100});
        let object_id = object::id(&obj);
        object::transfer(obj, from_addr);

        let obj = object::take_object<TestStruct>(&from, object_id);
        transfer::transfer_object<TestStruct>(to_addr, obj);
        
        let obj = object::borrow_object<TestStruct>(object_id);
        assert!(object::owner(obj)== to_addr, 1001);
    }
}