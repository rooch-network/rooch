// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

module rooch_framework::onchain_config {

    use std::option;
    use std::option::Option;
    use std::string::String;
    use moveos_std::bcs;
    use moveos_std::context::{Self, Context};
    use moveos_std::object;
    use std::vector;

    friend rooch_framework::upgrade;
    friend rooch_framework::genesis;

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
        gas_schedule: Option<GasSchedule>,
    }

    public(friend) fun genesis_init(ctx: &mut Context, _genesis_account: &signer, sequencer: address, gas_schedule_blob: vector<u8>){
        let gas_schedule = option::none<GasSchedule>();

        if (vector::length(&gas_schedule_blob) > 0) {
            gas_schedule = option::some(bcs::from_bytes<GasSchedule>(gas_schedule_blob));
        };

        let config = OnchainConfig{
            framework_version: 0,
            sequencer,
            gas_schedule,
        };
        let obj = context::new_named_object(ctx, config);
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
        let object_id = object::named_object_id<OnchainConfig>();
        let obj = context::borrow_mut_object_extend<OnchainConfig>(ctx, object_id);
        object::borrow_mut(obj)
    }

    public fun onchain_config(ctx: &Context): &OnchainConfig {
        let object_id = object::named_object_id<OnchainConfig>();
        let obj = context::borrow_object<OnchainConfig>(ctx, object_id);
        object::borrow(obj)
    }

    public fun onchain_gas_schedule(ctx: &Context): &Option<GasSchedule> {
        &onchain_config(ctx).gas_schedule
    }
}
