module counter_addr::counter {
    use moveos_std::account_storage;
    use moveos_std::context::Context;

    struct Counter has key {
        value: u64
    }

    fun init(ctx: &mut Context, account: &signer) {
        account_storage::global_move_to(ctx, account, Counter { value: 0 })
    }

    fun increase(ctx: &mut Context) {
        let counter = account_storage::global_borrow_mut<Counter>(ctx, @counter_addr);
        counter.value = counter.value + 1;
    }

    entry fun incr(ctx: &mut Context) {
        increase(ctx);
    }
}