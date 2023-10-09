#[test_only]
/// This test module is used to test the coin logic in coin and account module.
module rooch_framework::coin_test{
    use std::signer;
    use std::string;
    use moveos_std::context::{Self, Context};
    use rooch_framework::account;
    use rooch_framework::coin;
    use rooch_framework::coin::{register_extend,
        supply, name, symbol, decimals, balance, value, mint_extend, burn_extend, freeze_coin_store_extend, unfreeze_coin_store_extend,
        is_coin_store_frozen, zero, destroy_zero, is_registered, deposit, extract, transfer, withdraw, withdraw_extend,
        is_account_accept_coin, do_accept_coin, set_auto_accept_coin
    };

    #[test_only]
    struct FakeCoin has key, store {}

    #[test_only]
    fun register_fake_coin(
        ctx: &mut Context,
        decimals: u8,
    ) {
        coin::register_extend<FakeCoin>(
            ctx,
            string::utf8(b"Fake coin"),
            string::utf8(b"FCD"),
            decimals,
        );
    }

    #[test_only]
    fun mint_and_deposit(ctx: &mut Context,to_address: address, amount: u256) {
        let coins_minted = coin::mint_extend<FakeCoin>(ctx, amount);
        coin::deposit(ctx, to_address, coins_minted);
    }

    #[test(source = @rooch_framework, destination = @0x55)]
    fun test_end_to_end(
        source: signer,
        destination: signer,
    ) {
        let source_ctx = rooch_framework::genesis::init_for_test();
        let source_addr = signer::address_of(&source);
        let destination_addr = signer::address_of(&destination);
        let destination_ctx = context::new_test_context_random(signer::address_of(&destination), b"test_tx1");

        let name = string::utf8(b"Fake coin");
        let symbol = string::utf8(b"FCD");
        let decimals = 9u8;

        register_extend<FakeCoin>(
            &mut source_ctx,
            name,
            symbol,
            decimals,
        );
        account::create_account_for_test(&mut destination_ctx, signer::address_of(&destination));
        
        assert!(supply<FakeCoin>(&source_ctx) == 0, 0);

        assert!(name<FakeCoin>(&source_ctx) == name, 1);
        assert!(symbol<FakeCoin>(&source_ctx) == symbol, 2);
        assert!(decimals<FakeCoin>(&source_ctx) == decimals, 3);

        let coins_minted = mint_extend<FakeCoin>(&mut source_ctx, 100);
        deposit(&mut source_ctx, source_addr, coins_minted);
        transfer<FakeCoin>(&mut source_ctx, &source, destination_addr, 50);

        assert!(balance<FakeCoin>(&source_ctx, source_addr) == 50, 4);
        assert!(balance<FakeCoin>(&destination_ctx, destination_addr) == 50, 5);
        assert!(supply<FakeCoin>(&source_ctx) == 100, 6);

        let coin = withdraw<FakeCoin>(&mut source_ctx, &source, 10);
        assert!(value(&coin) == 10, 7);
        burn_extend(&mut source_ctx, coin);
        assert!(supply<FakeCoin>(&source_ctx, ) == 90, 8);
        moveos_std::context::drop_test_context(source_ctx);
        moveos_std::context::drop_test_context(destination_ctx);
    }

    #[test(source = @rooch_framework, destination = @0x55)]
    fun test_end_to_end_no_supply(
        source: signer,
        destination: signer,
    ) {
        let source_ctx = rooch_framework::genesis::init_for_test();
        let source_addr = signer::address_of(&source);
        let destination_addr = signer::address_of(&destination);
        let destination_ctx = context::new_test_context_random(signer::address_of(&destination), b"test_tx1");

        register_fake_coin(&mut source_ctx, 9);

        account::create_account_for_test(&mut destination_ctx, signer::address_of(&destination));
        assert!(supply<FakeCoin>(&source_ctx) == 0, 0);

        let coins_minted = mint_extend<FakeCoin>(&mut source_ctx, 100);
        deposit<FakeCoin>(&mut source_ctx, source_addr, coins_minted);
        transfer<FakeCoin>(&mut source_ctx, &source, destination_addr, 50);

        assert!(balance<FakeCoin>(&source_ctx, source_addr) == 50, 1);
        assert!(balance<FakeCoin>(&destination_ctx, destination_addr) == 50, 2);
        assert!(supply<FakeCoin>(&source_ctx) > 0, 3);

        let coin = withdraw<FakeCoin>(&mut source_ctx, &source, 10);
        burn_extend(&mut source_ctx, coin);
        assert!(supply<FakeCoin>(&source_ctx) > 0, 4);

        moveos_std::context::drop_test_context(source_ctx);
        moveos_std::context::drop_test_context(destination_ctx);
    }

    #[test]
    #[expected_failure(abort_code = 524289, location = rooch_framework::coin)]
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

    #[test(source = @rooch_framework, destination = @0x55)]
    #[expected_failure(abort_code = 393218, location = moveos_std::raw_table)]
    fun test_fail_transfer(
        source: signer,
        destination: signer,
    ) {
        let source_ctx = rooch_framework::genesis::init_for_test();
        let source_addr = signer::address_of(&source);
        let destination_addr = signer::address_of(&destination);
        let destination_ctx = context::new_test_context_random(signer::address_of(&destination), b"test_tx1");

        register_fake_coin(&mut source_ctx, 9);
        assert!(supply<FakeCoin>(&source_ctx) == 0, 0);

        let coins_minted = mint_extend<FakeCoin>(&mut source_ctx, 100);
        deposit(&mut source_ctx, source_addr, coins_minted);
        transfer<FakeCoin>(&mut source_ctx, &source, destination_addr, 50);

        moveos_std::context::drop_test_context(source_ctx);
        moveos_std::context::drop_test_context(destination_ctx);
    }

    #[test(source = @rooch_framework)]
    fun test_withdraw_from(
        source: signer,
    ) {
        let source_ctx = rooch_framework::genesis::init_for_test();
        let source_addr = signer::address_of(&source);

        register_fake_coin(&mut source_ctx, 9);

        let coins_minted = mint_extend<FakeCoin>(&mut source_ctx, 100);
        deposit(&mut source_ctx, source_addr, coins_minted);
        assert!(balance<FakeCoin>(&source_ctx, source_addr) == 100, 0);
        assert!(supply<FakeCoin>(&source_ctx) == 100, 1);

        let coin = withdraw_extend<FakeCoin>(&mut source_ctx, source_addr, 10);
        burn_extend<FakeCoin>(&mut source_ctx, coin);
        assert!(balance<FakeCoin>(&source_ctx, source_addr) == 90, 2);
        assert!(supply<FakeCoin>(&source_ctx) == 90, 3);

        moveos_std::context::drop_test_context(source_ctx);
    }

    #[test]
    #[expected_failure(abort_code = 65539, location = rooch_framework::coin)]
    public fun test_destroy_non_zero(
    ) {
        let source_ctx = rooch_framework::genesis::init_for_test();

        register_fake_coin(&mut source_ctx, 9);
        let coins_minted = mint_extend<FakeCoin>(&mut source_ctx, 100);
        destroy_zero(coins_minted);

        moveos_std::context::drop_test_context(source_ctx);
    }


    #[test(source = @rooch_framework)]
    fun test_test_extract(
        source: signer,
    ) {
        let source_ctx = rooch_framework::genesis::init_for_test();
        let source_addr = signer::address_of(&source);
        register_fake_coin(&mut source_ctx, 9);
        let coins_minted = mint_extend<FakeCoin>(&mut source_ctx, 100);

        let extracted = extract(&mut coins_minted, 25);
        assert!(value(&coins_minted) == 75, 0);
        assert!(value(&extracted) == 25, 1);

        deposit(&mut source_ctx, source_addr, coins_minted);
        deposit(&mut source_ctx, source_addr, extracted);

        assert!(balance<FakeCoin>(&source_ctx, source_addr) == 100, 2);

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

    #[test(account = @rooch_framework)]
    public fun test_is_coin_store_frozen(account: signer) {
        let ctx = rooch_framework::genesis::init_for_test();
        let addr = signer::address_of(&account);
        // An non do_accept_coined account is has a frozen coin store by default
        assert!(!is_coin_store_frozen<FakeCoin>(&ctx, addr), 1);

        register_fake_coin(&mut ctx, 9);

        assert!(!is_coin_store_frozen<FakeCoin>(&ctx, addr), 1);
        // freeze account
        freeze_coin_store_extend<FakeCoin>(&mut ctx, addr);
        assert!(is_coin_store_frozen<FakeCoin>(&ctx, addr), 1);

        // unfreeze account
        unfreeze_coin_store_extend<FakeCoin>(&mut ctx, addr);
        assert!(!is_coin_store_frozen<FakeCoin>(&ctx, addr), 1);

        moveos_std::context::drop_test_context(ctx);
    }

    #[test]
    fun test_zero() {
        let zero = zero<FakeCoin>();
        assert!(value(&zero) == 0, 1);
        destroy_zero(zero);
    }

    #[test(account = @rooch_framework)]
    fun test_burn_frozen(account: signer) {
        let ctx = rooch_framework::genesis::init_for_test();
        let account_addr = signer::address_of(&account);
        register_fake_coin(&mut ctx, 9);

        let coins_minted = mint_extend<FakeCoin>(&mut ctx, 100);
        deposit(&mut ctx, account_addr, coins_minted);

        freeze_coin_store_extend<FakeCoin>(&mut ctx, account_addr);
        let coin = withdraw_extend<FakeCoin>(&mut ctx, account_addr, 100);
        burn_extend(&mut ctx, coin);

        moveos_std::context::drop_test_context(ctx);
    }

    #[test(account = @rooch_framework)]
    #[expected_failure(abort_code = 327687, location = rooch_framework::coin)]
    fun test_withdraw_frozen(account: signer) {
        let ctx = rooch_framework::genesis::init_for_test();
        let account_addr = signer::address_of(&account);
        register_fake_coin(&mut ctx, 9);

        freeze_coin_store_extend<FakeCoin>(&mut ctx, account_addr);
        let coin = withdraw<FakeCoin>(&mut ctx, &account, 10);
        burn_extend(&mut ctx, coin);

        moveos_std::context::drop_test_context(ctx);
    }

    #[test(account = @rooch_framework)]
    #[expected_failure(abort_code = 327687, location = rooch_framework::coin)]
    fun test_deposit_frozen(account: signer) {
        let ctx = rooch_framework::genesis::init_for_test();
        let account_addr = signer::address_of(&account);
        register_fake_coin(&mut ctx, 9);

        let coins_minted = mint_extend<FakeCoin>(&mut ctx, 100);
        freeze_coin_store_extend<FakeCoin>(&mut ctx, account_addr);
        deposit(&mut ctx, account_addr, coins_minted);

        moveos_std::context::drop_test_context(ctx);
    }

    #[test(account = @rooch_framework)]
    fun test_deposit_widthdraw_unfrozen(account: signer) {
        let ctx = rooch_framework::genesis::init_for_test();
        let account_addr = signer::address_of(&account);
        register_fake_coin(&mut ctx, 9);

        let coins_minted = mint_extend<FakeCoin>(&mut ctx, 100);
        freeze_coin_store_extend<FakeCoin>(&mut ctx, account_addr);
        unfreeze_coin_store_extend<FakeCoin>(&mut ctx, account_addr);
        deposit(&mut ctx, account_addr, coins_minted);

        freeze_coin_store_extend<FakeCoin>(&mut ctx, account_addr);
        unfreeze_coin_store_extend<FakeCoin>(&mut ctx, account_addr);
        let coin = withdraw<FakeCoin>(&mut ctx, &account, 10);
        burn_extend(&mut ctx, coin);

        moveos_std::context::drop_test_context(ctx);
    }


    #[test(framework = @rooch_framework)]
    fun test_accept_twice_should_not_fail(framework: signer) {
        let ctx = rooch_framework::genesis::init_for_test();
        register_fake_coin(&mut ctx, 9);

        // Registering twice should not fail.
        do_accept_coin<FakeCoin>(&mut ctx, &framework);
        do_accept_coin<FakeCoin>(&mut ctx, &framework);
        assert!(is_account_accept_coin<FakeCoin>(&ctx, @rooch_framework), 1);

        moveos_std::context::drop_test_context(ctx);
    }

    #[test(source1 = @0x33, source2 = @0x66)]
    #[expected_failure(abort_code = 393224, location = rooch_framework::coin)]
    fun test_deposit_coin_after_turnoff_auto_accept_coin_flag_should_fail(source1: signer, source2: signer,) {
        let ctx = rooch_framework::genesis::init_for_test();

        let source1_addr = signer::address_of(&source1);
        let source2_addr = signer::address_of(&source2);
        let source1_ctx = context::new_test_context_random(source1_addr, b"test_tx2");
        let source2_ctx = context::new_test_context_random(source2_addr, b"test_tx3");
        register_fake_coin(&mut ctx, 9);

        let mint_coins1 = mint_extend<FakeCoin>(&mut ctx, 10);
        let mint_coins2 = mint_extend<FakeCoin>(&mut ctx, 20);

        account::create_account_for_test(&mut source1_ctx, source1_addr);
        account::create_account_for_test(&mut source2_ctx, source2_addr);

        // source1 default deposit should succ
        deposit(&mut source1_ctx, source1_addr, mint_coins1);

        // source2 turnoff auto accept coin flag, deposit should fail
        set_auto_accept_coin(&mut source2_ctx, &source2, false);
        deposit(&mut source2_ctx, source2_addr, mint_coins2);

        moveos_std::context::drop_test_context(ctx);
        moveos_std::context::drop_test_context(source1_ctx);
        moveos_std::context::drop_test_context(source2_ctx);
    }

    #[test(source1 = @0x33, source2 = @0x66)]
    fun test_deposit_coin_after_turnoff_auto_accept_coin_flag_and_accept_coin_should_succ(source1: signer, source2: signer,) {
        let ctx = rooch_framework::genesis::init_for_test();
    
        let source1_addr = signer::address_of(&source1);
        let source2_addr = signer::address_of(&source2);
        let source1_ctx = context::new_test_context_random(source1_addr, b"test_tx2");
        let source2_ctx = context::new_test_context_random(source2_addr, b"test_tx3");
        register_fake_coin(&mut ctx, 9);

        let mint_coins1 = mint_extend<FakeCoin>(&mut ctx, 10);
        let mint_coins2 = mint_extend<FakeCoin>(&mut ctx, 20);

        account::create_account_for_test(&mut source1_ctx, source1_addr);
        account::create_account_for_test(&mut source2_ctx, source2_addr);

        // source1 default deposit should succ
        deposit(&mut source1_ctx, source1_addr, mint_coins1);

        // source2 turnoff auto accept coin flag, deposit should fail
        set_auto_accept_coin(&mut source2_ctx, &source2, false);

        // source2 accept coin, deposit should succ
        do_accept_coin<FakeCoin>(&mut source2_ctx, &source2);
        deposit(&mut source2_ctx, source2_addr, mint_coins2);


        moveos_std::context::drop_test_context(ctx);
        moveos_std::context::drop_test_context(source1_ctx);
        moveos_std::context::drop_test_context(source2_ctx);
    }

    #[test(from_addr= @0x33, to_addr= @0x66)]
    fun test_transfer_to_no_exists_account(from_addr: address, to_addr: address) {
        let ctx = rooch_framework::genesis::init_for_test();
        register_fake_coin(&mut ctx, 9);

        let from = account::create_account_for_test(&mut ctx, from_addr);
        //assert!(!account::exists_at(&ctx, to_addr), 1000);
        //TODO remove this line after coin::transfer can auto create account
        account::create_account_for_test(&mut ctx, to_addr);

        let amount = 100u256;
        mint_and_deposit(&mut ctx, from_addr, amount);
        transfer<FakeCoin>(&mut ctx, &from, to_addr, 50u256);
        assert!(account::exists_at(&ctx, to_addr), 1000);
        assert!(balance<FakeCoin>(&ctx, to_addr) == 50u256, 1001);
        moveos_std::context::drop_test_context(ctx);
    }
}