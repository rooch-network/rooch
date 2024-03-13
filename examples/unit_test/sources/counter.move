module unit_test::unit_test {
    use moveos_std::account;
    use moveos_std::signer;
    
    struct Counter has key {
        count_value: u64
    }

    fun init(account: &signer) {
        account::move_resource_to(account, Counter { count_value: 0 });
    }

    entry fun increase(account: &signer) {
        let account_addr = signer::address_of(account);
        let counter = account::borrow_mut_resource<Counter>(account_addr);
        counter.count_value = counter.count_value + 1;
    }

    #[test(account = @0x42)]
    fun test_counter(account: &signer) {
        let account_addr = signer::address_of(account);
        account::move_resource_to(account, Counter { count_value: 0 });

        let counter = account::borrow_resource<Counter>(account_addr);
        assert!(counter.count_value == 0, 999);
        // assert!(counter.count_value == 2, 999);
        increase(account);
        let counter = account::borrow_resource<Counter>(account_addr);
        assert!(counter.count_value == 1, 1000);
    }
}
