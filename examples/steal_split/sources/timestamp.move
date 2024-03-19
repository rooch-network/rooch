// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

module rooch_examples::timestamp {
    use std::signer;
    use moveos_std::account;

    

    struct CurrentTimeMicroseconds has key, store {
        microseconds: u64,
    }

    /// Conversion factor between seconds and microseconds
    const MICRO_CONVERSION_FACTOR: u64 = 1000000;

    /// The blockchain is not in an operating state yet
    const ErrorNotOperating: u64 = 1;
    /// An invalid timestamp was provided
    const ErrorInvalidTimestamp: u64 = 2;

    public(friend) fun set_time_has_started(framework: &signer, ) {
        assert!(signer::address_of(framework) == @moveos_std, ErrorNotOperating);
        let timer = CurrentTimeMicroseconds { microseconds: 0 };
        account::move_resource_to(framework, timer);
    }

    public(friend) fun update_global_time(
        timestamp: u64, 
    )   {
        let global_timer_mut_ref = account::borrow_mut_resource<CurrentTimeMicroseconds>(@moveos_std);
        global_timer_mut_ref.microseconds = timestamp;
    }

    #[test_only]
    public fun set_time_has_started_for_testing() {
        if (!account::exists_resource<CurrentTimeMicroseconds>(@moveos_std)) {
            set_time_has_started(&account::create_signer_for_testing(@moveos_std));
        };
    }

    public fun now_microseconds(): u64 {
       account::borrow_resource<CurrentTimeMicroseconds>(@moveos_std).microseconds
    }


    public fun now_seconds(): u64  {
        now_microseconds() / MICRO_CONVERSION_FACTOR
    }

    #[test_only]
    public fun update_global_time_for_test(timestamp_microsecs: u64,)  {
        let global_timer_mut_ref = account::borrow_mut_resource<CurrentTimeMicroseconds>(@moveos_std);
        let now = global_timer_mut_ref.microseconds;
        assert!(now < timestamp_microsecs, ErrorInvalidTimestamp);
        global_timer_mut_ref.microseconds = timestamp_microsecs;
    }

    #[test_only]
    public fun update_global_time_for_test_secs(timestamp_seconds: u64,)   {
        update_global_time_for_test(timestamp_seconds * MICRO_CONVERSION_FACTOR);
    }

    #[test_only]
    public fun fast_forward_seconds(timestamp_seconds: u64,)   {
        update_global_time_for_test_secs(now_seconds() + timestamp_seconds);
    }
}
