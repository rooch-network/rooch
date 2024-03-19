#[test_only]
/// This test module is used to test the account_coin_store module.
module rooch_framework::account_coin_store_test{
    use std::signer;
    use std::string;
    
    use moveos_std::object::{Self, Object};
    use rooch_framework::coin::{Self, CoinInfo,
        supply, value, mint_extend, burn_extend
    };
    use rooch_framework::account as account_entry;
    use rooch_framework::account_coin_store::{Self, transfer, withdraw, withdraw_extend,
        is_accept_coin, do_accept_coin, set_auto_accept_coin, balance, deposit};

    #[test_only]
    struct FakeCoin has key, store {}

    #[test_only]
    fun register_fake_coin(
        
        decimals: u8,
    ) : Object<CoinInfo<FakeCoin>> {
        coin::register_extend<FakeCoin>(
            string::utf8(b"Fake coin"),
            string::utf8(b"FCD"),
            decimals,
        )
    }

    #[test_only]
    fun mint_and_deposit(coin_info_obj: &mut Object<CoinInfo<FakeCoin>>, to_address: address, amount: u256) {
        let coins_minted = coin::mint_extend<FakeCoin>(coin_info_obj, amount);
        account_coin_store::deposit(to_address, coins_minted);
    }

    #[test(source = @rooch_framework, destination = @0x55)]
    fun test_end_to_end(
        source: signer,
        destination: signer,
    ) {
        rooch_framework::genesis::init_for_test();
        let source_addr = signer::address_of(&source);
        let destination_addr = signer::address_of(&destination);
        

        let coin_info_obj = register_fake_coin(9);

        account_entry::create_account_for_testing(signer::address_of(&destination));

        let coins_minted = mint_extend<FakeCoin>(&mut coin_info_obj, 100);
        deposit(source_addr, coins_minted);

        let coin = withdraw<FakeCoin>(&source, 10);
        assert!(value(&coin) == 10, 7);
        deposit(destination_addr, coin);

        transfer<FakeCoin>(&source, destination_addr, 50);

        assert!(balance<FakeCoin>(source_addr) == 40, 4);
        assert!(balance<FakeCoin>(destination_addr) == 60, 5);
        object::transfer(coin_info_obj, @rooch_framework);
        
        
    }

    
    #[test(source = @rooch_framework)]
    fun test_withdraw_from(
        source: signer,
    ) {
        rooch_framework::genesis::init_for_test();
        let source_addr = signer::address_of(&source);

        let coin_info_obj = register_fake_coin(9);

        let coins_minted = mint_extend<FakeCoin>(&mut coin_info_obj, 100);
        deposit(source_addr, coins_minted);
        assert!(balance<FakeCoin>(source_addr) == 100, 0);
        assert!(supply<FakeCoin>(object::borrow(&coin_info_obj)) == 100, 1);

        let coin = withdraw_extend<FakeCoin>(source_addr, 10);
        burn_extend<FakeCoin>(&mut coin_info_obj, coin);
        assert!(balance<FakeCoin>(source_addr) == 90, 2);
        assert!(supply<FakeCoin>(object::borrow(&coin_info_obj)) == 90, 3);
        object::transfer(coin_info_obj, @rooch_framework);
        
    }


    #[test(framework = @rooch_framework)]
    fun test_accept_twice_should_not_fail(framework: signer) {
        rooch_framework::genesis::init_for_test();
        let coin_info_obj = register_fake_coin(9);

        // Registering twice should not fail.
        do_accept_coin<FakeCoin>(&framework);
        do_accept_coin<FakeCoin>(&framework);
        assert!(is_accept_coin<FakeCoin>(@rooch_framework), 1);
        object::transfer(coin_info_obj, @rooch_framework);
        
    }

    #[test(source1 = @0x33, source2 = @0x66)]
    #[expected_failure(abort_code = 1, location = rooch_framework::account_coin_store)]
    fun test_deposit_coin_after_turnoff_auto_accept_coin_flag_should_fail(source1: signer, source2: signer,) {
        rooch_framework::genesis::init_for_test();

        let source1_addr = signer::address_of(&source1);
        let source2_addr = signer::address_of(&source2);
        
        
        let coin_info_obj = register_fake_coin(9);

        let mint_coins1 = mint_extend<FakeCoin>(&mut coin_info_obj, 10);
        let mint_coins2 = mint_extend<FakeCoin>(&mut coin_info_obj, 20);

        account_entry::create_account_for_testing(source1_addr);
        account_entry::create_account_for_testing(source2_addr);

        // source1 default deposit should succ
        deposit(source1_addr, mint_coins1);

        // source2 turnoff auto accept coin flag, deposit should fail
        set_auto_accept_coin(&source2, false);
        deposit(source2_addr, mint_coins2);
        object::transfer(coin_info_obj, @rooch_framework);
        
    }

    #[test(source1 = @0x33, source2 = @0x66)]
    fun test_deposit_coin_after_turnoff_auto_accept_coin_flag_and_accept_coin_should_succ(source1: signer, source2: signer,) {
        rooch_framework::genesis::init_for_test();
    
        let source1_addr = signer::address_of(&source1);
        let source2_addr = signer::address_of(&source2);
        
        
        let coin_info_obj = register_fake_coin(9);

        let mint_coins1 = mint_extend<FakeCoin>(&mut coin_info_obj, 10);
        let mint_coins2 = mint_extend<FakeCoin>(&mut coin_info_obj, 20);

        account_entry::create_account_for_testing(source1_addr);
        account_entry::create_account_for_testing(source2_addr);

        // source1 default deposit should succ
        deposit(source1_addr, mint_coins1);

        // source2 turnoff auto accept coin flag, deposit should fail
        set_auto_accept_coin(&source2, false);

        // source2 accept coin, deposit should succ
        do_accept_coin<FakeCoin>(&source2);
        deposit(source2_addr, mint_coins2);

        object::transfer(coin_info_obj, @rooch_framework);
        
    }

    #[test(source = @rooch_framework, destination = @0x55)]
    #[expected_failure(abort_code = 2, location = moveos_std::object)]
    fun test_fail_transfer(
        source: signer,
        destination: signer,
    ) {
        rooch_framework::genesis::init_for_test();
        let source_addr = signer::address_of(&source);
        let destination_addr = signer::address_of(&destination);
        

        let coin_info_obj = register_fake_coin(9);
        assert!(supply<FakeCoin>(object::borrow(&coin_info_obj)) == 0, 0);

        let coins_minted = mint_extend<FakeCoin>(&mut coin_info_obj, 100);
        deposit(source_addr, coins_minted);
        transfer<FakeCoin>(&source, destination_addr, 50);

        object::transfer(coin_info_obj, @rooch_framework);
        
        
    }
}