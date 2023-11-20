// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

/// This module keeps a global wall clock that stores the current Unix time in milliseconds.
/// It interacts with the other modules in the following ways:
/// * genesis: to initialize the timestamp
/// * L1 block: update the timestamp via L1s block header timestamp
/// * TickTransaction: update the timestamp via the time offset in the TickTransaction(TODO)
module rooch_framework::timestamp {
   
    use std::error;
    use moveos_std::object;
    use moveos_std::context::{Self, Context};

    friend rooch_framework::genesis;
    friend rooch_framework::ethereum_light_client;
    friend rooch_framework::bitcoin_light_client;

    /// A object holding the current Unix time in milliseconds
    struct Timestamp has key {
        milliseconds: u64,
    }

    /// Conversion factor between seconds and milliseconds
    const MILLI_CONVERSION_FACTOR: u64 = 1000;

    /// An invalid timestamp was provided
    const ErrorInvalidTimestamp: u64 = 1;

    public(friend) fun genesis_init(ctx: &mut Context, _genesis_account: &signer, initial_time_milliseconds: u64) {
        let timestamp = Timestamp { milliseconds: initial_time_milliseconds };
        let obj = context::new_named_object(ctx, timestamp);
        object::transfer_extend(obj, @rooch_framework);
    }

    /// Updates the global clock time, if the new time is smaller than the current time, aborts.
    public(friend) fun update_global_time(ctx: &mut Context, timestamp_milliseconds: u64) {
        let current_timestamp = timestamp_mut(ctx); 
        let now = current_timestamp.milliseconds;
        assert!(now < timestamp_milliseconds, error::invalid_argument(ErrorInvalidTimestamp));
        current_timestamp.milliseconds = timestamp_milliseconds;
    }

    /// Tries to update the global clock time, if the new time is smaller than the current time, ignores the update, and returns false.
    public(friend) fun try_update_global_time(ctx: &mut Context, timestamp_milliseconds: u64) : bool {
        let current_timestamp = timestamp_mut(ctx); 
        let now = current_timestamp.milliseconds;
        if(now < timestamp_milliseconds) {
            current_timestamp.milliseconds = timestamp_milliseconds;
            true
        }else{
            false
        }
    }

    fun timestamp_mut(ctx: &mut Context): &mut Timestamp {
        let object_id = object::named_object_id<Timestamp>();
        let obj = context::borrow_mut_object_extend<Timestamp>(ctx, object_id);
        object::borrow_mut(obj)
    }

    public fun timestamp(ctx: &Context): &Timestamp {
        let object_id = object::named_object_id<Timestamp>();
        let obj = context::borrow_object<Timestamp>(ctx, object_id);
        object::borrow(obj)
    }

    public fun milliseconds(self: &Timestamp): u64 {
        self.milliseconds
    }

    public fun seconds(self: &Timestamp): u64 {
        self.milliseconds / MILLI_CONVERSION_FACTOR
    }

    /// Gets the current time in milliseconds.
    public fun now_milliseconds(ctx: &Context): u64 {
        let timestamp = timestamp(ctx);
        timestamp.milliseconds
    }

    /// Gets the current time in seconds.
    public fun now_seconds(ctx: &Context): u64 {
        now_milliseconds(ctx) / MILLI_CONVERSION_FACTOR
    }

    public fun seconds_to_milliseconds(seconds: u64): u64 {
        seconds * MILLI_CONVERSION_FACTOR
    }

    #[test_only]
    public fun update_global_time_for_test(ctx: &mut Context, timestamp_milliseconds: u64){
        update_global_time(ctx, timestamp_milliseconds);
    }

    #[test_only]
    public fun update_global_time_for_test_secs(ctx: &mut Context, timestamp_seconds: u64) {
        update_global_time(ctx, timestamp_seconds * MILLI_CONVERSION_FACTOR);
    }

    #[test_only]
    public fun fast_forward_seconds_for_test(ctx: &mut Context, timestamp_seconds: u64) {
        fast_forward_seconds(ctx, timestamp_seconds)
    }

    fun fast_forward_seconds(ctx: &mut Context, timestamp_seconds: u64) {
        let now_milliseconds = now_milliseconds(ctx);
        update_global_time(ctx, now_milliseconds + (timestamp_seconds * MILLI_CONVERSION_FACTOR));
    }

    /// Fast forwards the clock by the given number of seconds, but only if the chain is in local mode.
    //TODO find a better way to do this, maybe some module that is only available in local chain?
    public entry fun fast_forward_seconds_for_local(ctx: &mut Context, timestamp_seconds: u64) {
        assert!(rooch_framework::chain_id::is_local(ctx), error::invalid_argument(ErrorInvalidTimestamp));
        fast_forward_seconds(ctx, timestamp_seconds);
    }
}
