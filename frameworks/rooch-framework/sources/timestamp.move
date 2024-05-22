// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

/// This module keeps a global wall clock that stores the current Unix time in milliseconds.
/// It interacts with the other modules in the following ways:
/// * genesis: to initialize the timestamp
/// * L1 block: update the timestamp via L1s block header timestamp
/// * L2 transactions: update the timestamp via L2 transaction's timestamp 
module rooch_framework::timestamp {

    use moveos_std::object;
    use moveos_std::signer;
    use moveos_std::core_addresses;

    friend rooch_framework::genesis;
    friend rooch_framework::transaction_validator;

    /// A object holding the current Unix time in milliseconds
    struct Timestamp has key {
        milliseconds: u64,
    }

    /// Conversion factor between seconds and milliseconds
    const MILLI_CONVERSION_FACTOR: u64 = 1000;

    /// An invalid timestamp was provided
    const ErrorInvalidTimestamp: u64 = 1;
    const ErrorNotGenesisAddress: u64 = 2;

    public(friend) fun genesis_init(_genesis_account: &signer, initial_time_milliseconds: u64) {
        let timestamp = Timestamp { milliseconds: initial_time_milliseconds };
        let obj = object::new_named_object(timestamp);
        object::transfer_extend(obj, @rooch_framework);
    }

    /// Updates the global clock time, if the new time is smaller than the current time, aborts.
    public(friend) fun update_global_time(timestamp_milliseconds: u64) {
        let current_timestamp = timestamp_mut(); 
        let now = current_timestamp.milliseconds;
        assert!(now < timestamp_milliseconds, ErrorInvalidTimestamp);
        current_timestamp.milliseconds = timestamp_milliseconds;
    }

    /// Tries to update the global clock time, if the new time is smaller than the current time, ignores the update, and returns false.
    /// Only the framework genesis account can update the global clock time.
    public fun try_update_global_time(genesis_account: &signer, timestamp_milliseconds: u64) : bool {
        let genesis_address = signer::address_of(genesis_account);
        assert!(core_addresses::is_system_reserved_address(genesis_address), ErrorNotGenesisAddress);
        try_update_global_time_internal(timestamp_milliseconds)
    }

    public(friend) fun try_update_global_time_internal(timestamp_milliseconds: u64) : bool {
        let current_timestamp = timestamp_mut(); 
        let now = current_timestamp.milliseconds;
        if(now < timestamp_milliseconds) {
            current_timestamp.milliseconds = timestamp_milliseconds;
            true
        }else{
            false
        }
    }

    fun timestamp_mut(): &mut Timestamp {
        let object_id = object::named_object_id<Timestamp>();
        let obj = object::borrow_mut_object_extend<Timestamp>(object_id);
        object::borrow_mut(obj)
    }

    public fun timestamp(): &Timestamp {
        let object_id = object::named_object_id<Timestamp>();
        let obj = object::borrow_object<Timestamp>(object_id);
        object::borrow(obj)
    }

    public fun milliseconds(self: &Timestamp): u64 {
        self.milliseconds
    }

    public fun seconds(self: &Timestamp): u64 {
        self.milliseconds / MILLI_CONVERSION_FACTOR
    }

    /// Gets the current time in milliseconds.
    public fun now_milliseconds(): u64 {
        let timestamp = timestamp();
        timestamp.milliseconds
    }

    /// Gets the current time in seconds.
    public fun now_seconds(): u64 {
        now_milliseconds() / MILLI_CONVERSION_FACTOR
    }

    public fun seconds_to_milliseconds(seconds: u64): u64 {
        seconds * MILLI_CONVERSION_FACTOR
    }

    fun fast_forward_seconds(timestamp_seconds: u64) {
        let now_milliseconds = now_milliseconds();
        update_global_time(now_milliseconds + (timestamp_seconds * MILLI_CONVERSION_FACTOR));
    }

    #[test_only]
    public fun update_global_time_for_test(timestamp_milliseconds: u64){
        update_global_time(timestamp_milliseconds);
    }

    #[test_only]
    public fun update_global_time_for_test_secs(timestamp_seconds: u64) {
        update_global_time(timestamp_seconds * MILLI_CONVERSION_FACTOR);
    }

    #[test_only]
    public fun fast_forward_seconds_for_test(timestamp_seconds: u64) {
        fast_forward_seconds(timestamp_seconds)
    }

    /// Fast forwards the clock by the given number of seconds, but only if the chain is in local mode.
    public entry fun fast_forward_seconds_for_local(timestamp_seconds: u64) {
        assert!(rooch_framework::chain_id::is_local(), ErrorInvalidTimestamp);
        fast_forward_seconds(timestamp_seconds);
    }
}
