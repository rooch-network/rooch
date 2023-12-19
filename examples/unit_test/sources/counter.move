module unit_test::unit_test {
    use moveos_std::signer;
    use moveos_std::context::{Self, Context};

    struct Counter has key {
        count_value: u64
    }

    fun init(ctx: &mut Context, account: &signer) {
        context::move_resource_to(ctx, account, Counter { count_value: 0 });
    }

    entry fun increase(ctx: &mut Context, account: &signer) {
        let account_addr = signer::address_of(account);
        let counter = context::borrow_mut_resource<Counter>(ctx, account_addr);
        counter.count_value = counter.count_value + 1;
    }

    #[test(account = @0x42)]
    fun test_init(account: &signer) {
        let account_addr = signer::address_of(account);
        let tx_context = &mut context::new_test_context(account_addr);
        context::move_resource_to(tx_context, account, Counter { count_value: 0});
    }
}

// #[test_only]
// module unit_test::unit_test2 {
//     #[test]
//     fun test() {}
// }
