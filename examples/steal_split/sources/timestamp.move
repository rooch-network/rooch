module steal_split::timestamp {
    use std::signer;
    use moveos_std::account_storage;
    use moveos_std::storage_context::StorageContext;
    #[test_only]
    use rooch_framework::account;

    struct CurrentTimeMicroseconds has key {
        microseconds: u64,
    }

    /// Conversion factor between seconds and microseconds
    const MICRO_CONVERSION_FACTOR: u64 = 1000000;

    /// The blockchain is not in an operating state yet
    const ENOT_OPERATING: u64 = 1;
    /// An invalid timestamp was provided
    const EINVALID_TIMESTAMP: u64 = 2;

    public(friend) fun set_time_has_started(framework: &signer, ctx: &mut StorageContext) {
        assert!(signer::address_of(framework) == @moveos_std, ENOT_OPERATING);
        let timer = CurrentTimeMicroseconds { microseconds: 0 };
        account_storage::global_move_to(ctx, framework, timer);
    }

    public(friend) fun update_global_time(
        timestamp: u64, ctx: &mut StorageContext
    )   {
        let global_timer_mut_ref = account_storage::global_borrow_mut<CurrentTimeMicroseconds>(ctx, @moveos_std);
        global_timer_mut_ref.microseconds = timestamp;
    }

    #[test_only]
    public fun set_time_has_started_for_testing(ctx: &mut StorageContext) {
        if (!account_storage::global_exists<CurrentTimeMicroseconds>(ctx, @moveos_std)) {
            set_time_has_started(&account::create_signer_for_test(@moveos_std),ctx);
        };
    }

    public fun now_microseconds(ctx: & StorageContext): u64 {
       account_storage::global_borrow<CurrentTimeMicroseconds>(ctx, @moveos_std).microseconds
    }


    public fun now_seconds(ctx: & StorageContext): u64  {
        now_microseconds(ctx) / MICRO_CONVERSION_FACTOR
    }

    #[test_only]
    public fun update_global_time_for_test(timestamp_microsecs: u64,ctx: &mut StorageContext)  {
        let global_timer_mut_ref = account_storage::global_borrow_mut<CurrentTimeMicroseconds>(ctx, @moveos_std);
        let now = global_timer_mut_ref.microseconds;
        assert!(now < timestamp_microsecs, EINVALID_TIMESTAMP);
        global_timer_mut_ref.microseconds = timestamp_microsecs;
    }

    #[test_only]
    public fun update_global_time_for_test_secs(timestamp_seconds: u64,ctx: &mut StorageContext)   {
        update_global_time_for_test(timestamp_seconds * MICRO_CONVERSION_FACTOR,ctx);
    }

    #[test_only]
    public fun fast_forward_seconds(timestamp_seconds: u64,ctx: &mut StorageContext)   {
        update_global_time_for_test_secs(now_seconds(ctx) + timestamp_seconds,ctx);
    }
}
