// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

module rooch_examples::timestamp {
    use std::signer;

    use moveos_std::account_storage;
    use moveos_std::context::Context;

    #[test_only]
    use rooch_framework::account;

    struct CurrentTimeMicroseconds has key {
        microseconds: u64,
    }

    /// Conversion factor between seconds and microseconds
    const MICRO_CONVERSION_FACTOR: u64 = 1000000;

    /// The blockchain is not in an operating state yet
    const ErrorNotOperating: u64 = 1;
    /// An invalid timestamp was provided
    const ErrorInvalidTimestamp: u64 = 2;

    public(friend) fun set_time_has_started(framework: &signer, ctx: &mut Context) {
        assert!(signer::address_of(framework) == @moveos_std, ErrorNotOperating);
        let timer = CurrentTimeMicroseconds { microseconds: 0 };
        account_storage::global_move_to(ctx, framework, timer);
    }

    public(friend) fun update_global_time(
        timestamp: u64, ctx: &mut Context
    )   {
        let global_timer_mut_ref = account_storage::global_borrow_mut<CurrentTimeMicroseconds>(ctx, @moveos_std);
        global_timer_mut_ref.microseconds = timestamp;
    }

    #[test_only]
    public fun set_time_has_started_for_testing(ctx: &mut Context) {
        if (!account_storage::global_exists<CurrentTimeMicroseconds>(ctx, @moveos_std)) {
            set_time_has_started(&account::create_signer_for_test(@moveos_std),ctx);
        };
    }

    public fun now_microseconds(ctx: & Context): u64 {
       account_storage::global_borrow<CurrentTimeMicroseconds>(ctx, @moveos_std).microseconds
    }


    public fun now_seconds(ctx: & Context): u64  {
        now_microseconds(ctx) / MICRO_CONVERSION_FACTOR
    }

    #[test_only]
    public fun update_global_time_for_test(timestamp_microsecs: u64,ctx: &mut Context)  {
        let global_timer_mut_ref = account_storage::global_borrow_mut<CurrentTimeMicroseconds>(ctx, @moveos_std);
        let now = global_timer_mut_ref.microseconds;
        assert!(now < timestamp_microsecs, ErrorInvalidTimestamp);
        global_timer_mut_ref.microseconds = timestamp_microsecs;
    }

    #[test_only]
    public fun update_global_time_for_test_secs(timestamp_seconds: u64,ctx: &mut Context)   {
        update_global_time_for_test(timestamp_seconds * MICRO_CONVERSION_FACTOR,ctx);
    }

    #[test_only]
    public fun fast_forward_seconds(timestamp_seconds: u64,ctx: &mut Context)   {
        update_global_time_for_test_secs(now_seconds(ctx) + timestamp_seconds,ctx);
    }
}
