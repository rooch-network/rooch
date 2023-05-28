module rooch_examples::counter {
    use std::signer;
    use moveos_std::account_storage;
    use moveos_std::storage_context;

    struct Counter has key, store {
        value: u64,
    }

    public fun init_(account: &signer) {
        let sender_addr = signer::address_of(account);
        let ctx = storage_context::new_context(sender_addr,b"counter");
        account_storage::create_account_storage(&mut ctx, sender_addr);
        account_storage::global_move_to(&mut ctx, account, Counter {
            value: 0,
        });
    }

    public fun increase_() acquires Counter {
        let ctx = storage_context::new_context(@rooch_examples,b"counter");
        let counter = account_storage::borrow_global_mut<Counter>(ctx, @rooch_examples);
        counter.value = counter.value + 1;
    }

    public entry fun init(account: signer) {
        Self::init_(&account)
    }

    public entry fun increase() acquires Counter {
        Self::increase_()
    }

    public fun value(): u64 acquires Counter {
        let ctx = storage_context::new_context(@rooch_examples,b"counter");
        let counter = account_storage::borrow_global<Counter>(ctx,@rooch_examples);
        counter.value
    }
}