module quick_start_counter::quick_start_counter {
    use moveos_std::account;
    use moveos_std::context::{Context};

    struct Counter has key {
        count_value: u64
    }

    fun init(ctx: &mut Context, account: &signer) {
        account::move_resource_to(ctx, account, Counter { count_value: 0 });
    }

    entry fun increase(ctx: &mut Context) {
        let counter = account::borrow_mut_resource<Counter>(ctx, @quick_start_counter);
        counter.count_value = counter.count_value + 1;
    }
}