/// This module keeps a global wall clock that stores the current Unix time in milliseconds.
/// It interacts with the other modules in the following ways:
/// * genesis: to initialize the timestamp
/// * block: to reach consensus on the global wall clock time
module rooch_framework::timestamp {
    use std::error;
    use rooch_framework::core_addresses;

    friend rooch_framework::genesis;

    /// A singleton resource holding the current Unix time in milliseconds
    struct CurrentTimeMilliseconds has key {
        milliseconds: u64,
    }

    /// Conversion factor between seconds and milliseconds
    const MILLI_CONVERSION_FACTOR: u64 = 1000;

    /// The blockchain is not in an operating state yet
    const ENotOperating: u64 = 1;
    /// An invalid timestamp was provided
    const EInvalidTimestamp: u64 = 2;

    // Initialize the global wall clock time resource.
    public(friend) fun initialize(account: &signer, genesis_timestamp: u64) {
        // Only callable by the genesis address
        core_addresses::assert_rooch_genesis(account);
        let timer = CurrentTimeMilliseconds {milliseconds: genesis_timestamp};
        move_to<CurrentTimeMilliseconds>(account, timer);
    }

    /// Updates the wall clock time by consensus. Requires VM privilege and will be invoked during block prologue.
    public fun update_global_time(
        account: &signer,
        proposer: address,
        timestamp: u64
    ) acquires CurrentTimeMilliseconds {
        // Can only be invoked by VM signer.
        core_addresses::assert_vm(account);

        let global_timer = borrow_global_mut<CurrentTimeMilliseconds>(@rooch_framework);
        let now = global_timer.milliseconds;
        if (proposer == @vm_reserved) {
            // NIL block with null address as proposer. Timestamp must be equal.
            assert!(now == timestamp, error::invalid_argument(EInvalidTimestamp));
        } else {
            // Normal block. Time must advance
            assert!(now < timestamp, error::invalid_argument(EInvalidTimestamp));
            global_timer.milliseconds = timestamp;
        };
    }

    #[test_only]
    public fun initialize_timestamp_for_testing(account: &signer) {
        if (!exists<CurrentTimeMilliseconds>(@rooch_framework)) {
            initialize(account, 0);
        };
    }

    #[view]
    /// Gets the current time in milliseconds.
    public fun now_milliseconds(): u64 acquires CurrentTimeMilliseconds {
        borrow_global<CurrentTimeMilliseconds>(@rooch_framework).milliseconds
    }

    #[view]
    /// Gets the current time in seconds.
    public fun now_seconds(): u64 acquires CurrentTimeMilliseconds {
        now_milliseconds() / MILLI_CONVERSION_FACTOR
    }

    #[test_only]
    public fun update_global_time_for_test(timestamp_microsecs: u64) acquires CurrentTimeMilliseconds {
        let global_timer = borrow_global_mut<CurrentTimeMilliseconds>(@rooch_framework);
        let now = global_timer.milliseconds;
        assert!(now < timestamp_microsecs, error::invalid_argument(EInvalidTimestamp));
        global_timer.milliseconds = timestamp_microsecs;
    }

    #[test_only]
    public fun update_global_time_for_test_secs(timestamp_seconds: u64) acquires CurrentTimeMilliseconds {
        update_global_time_for_test(timestamp_seconds * MILLI_CONVERSION_FACTOR);
    }

    #[test_only]
    public fun fast_forward_seconds(timestamp_seconds: u64) acquires CurrentTimeMilliseconds {
        update_global_time_for_test_secs(now_seconds() + timestamp_seconds);
    }
}
