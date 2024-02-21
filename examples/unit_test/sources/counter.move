module unit_test::unit_test {
    use moveos_std::account;
    use moveos_std::signer;
    use moveos_std::context::{Context};
    #[test_only]
    use moveos_std::context::drop_test_context;

    struct Counter has key {
        count_value: u64
    }

    fun init(ctx: &mut Context, account: &signer) {
        account::move_resource_to(ctx, account, Counter { count_value: 0 });
    }

    entry fun increase(ctx: &mut Context, account: &signer) {
        let account_addr = signer::address_of(account);
        let counter = account::borrow_mut_resource<Counter>(ctx, account_addr);
        counter.count_value = counter.count_value + 1;
    }

    #[test(account = @0x42)]
    fun test_counter(account: &signer) {
        let account_addr = signer::address_of(account);
        let ctx = context::new_test_context(account_addr);
        account::move_resource_to(&mut ctx, account, Counter { count_value: 0 });

        let counter = account::borrow_resource<Counter>(&ctx, account_addr);
        assert!(counter.count_value == 0, 999);
        // assert!(counter.count_value == 2, 999);

        increase(&mut ctx, account);
        let counter = account::borrow_resource<Counter>(&ctx, account_addr);
        assert!(counter.count_value == 1, 1000);

        drop_test_context(ctx);
    }
}
