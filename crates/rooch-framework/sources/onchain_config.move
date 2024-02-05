// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

module rooch_framework::onchain_config {

    use moveos_std::object_id;
    use std::string::String;
    use moveos_std::bcs;
    use moveos_std::context::{Self, Context};
    use moveos_std::object;
    use std::vector;
    use moveos_std::signer;

    friend rooch_framework::upgrade;
    friend rooch_framework::genesis;

    const ErrorNotSequencer: u64 = 1;

    struct GasScheduleUpdated has store, copy, drop {
        last_updated: u64
    }

    #[data_struct]
    struct GasEntry has store, copy, drop {
        key: String,
        val: u64,
    }

    #[data_struct]
    struct GasSchedule has key, copy, drop, store {
        feature_version: u64,
        entries: vector<GasEntry>,
    }

    /// OnchainConfig is framework configurations stored on chain.
    struct OnchainConfig has key {
        framework_version: u64,
        sequencer: address,
    }

    public(friend) fun genesis_init(ctx: &mut Context, _genesis_account: &signer, sequencer: address, gas_schedule_blob: vector<u8>){
        let gas_schedule = GasSchedule {
            feature_version: 0,
            entries: vector::empty<GasEntry>()
        };

        if (vector::length(&gas_schedule_blob) > 0) {
            gas_schedule = bcs::from_bytes<GasSchedule>(gas_schedule_blob);
        };

        let config = OnchainConfig{
            framework_version: 0,
            sequencer,
        };
        let obj = context::new_named_object(ctx, config);
        object::transfer_extend(obj, @rooch_framework);

        let obj = context::new_named_object(ctx, gas_schedule);
        object::transfer_extend(obj, @rooch_framework);
    }

    public fun sequencer(ctx: &Context): address {
        onchain_config(ctx).sequencer
    }

    public(friend) fun update_framework_version(ctx: &mut Context) {
        let config = onchain_config_mut(ctx);
        config.framework_version = config.framework_version + 1;
    }

    public fun framework_version(ctx: &Context): u64 {
        onchain_config(ctx).framework_version
    }

    fun onchain_config_mut(ctx: &mut Context): &mut OnchainConfig {
        let object_id = object_id::named_object_id<OnchainConfig>();
        let obj = object::borrow_mut_object_extend<OnchainConfig>(object_id);
        object::borrow_mut(obj)
    }

    public fun onchain_config(ctx: &Context): &OnchainConfig {
        let object_id = object_id::named_object_id<OnchainConfig>();
        let obj = object::borrow_object<OnchainConfig>(object_id);
        object::borrow(obj)
    }

    entry fun update_onchain_gas_schedule(ctx: &mut Context, account: &signer, gas_schedule_blob: vector<u8>) {
        let sender_address = signer::address_of(account);
        assert!(sender_address == Self::sequencer(ctx), ErrorNotSequencer);

        let gas_schedule = GasSchedule {
            feature_version: 0,
            entries: vector::empty<GasEntry>()
        };

        if (vector::length(&gas_schedule_blob) > 0) {
            gas_schedule = bcs::from_bytes<GasSchedule>(gas_schedule_blob);
        };

        context::add(ctx, GasScheduleUpdated {last_updated: 1});

        let obj = context::new_named_object(ctx, gas_schedule);
        object::transfer_extend(obj, @rooch_framework);
    }

    public fun onchain_gas_schedule(ctx: &Context): &GasSchedule {
        let object_id = object_id::named_object_id<GasSchedule>();
        let obj = object::borrow_object<GasSchedule>(object_id);
        object::borrow(obj)
    }
}
