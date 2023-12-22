// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

module rooch_framework::onchain_config {

    use moveos_std::context::{Self, Context};
    use moveos_std::object;

    friend rooch_framework::upgrade;
    friend rooch_framework::genesis;

    /// OnchainConfig is framework configurations stored on chain.
    struct OnchainConfig has key {
        framework_version: u64,
        sequencer: address,
    }

    public(friend) fun genesis_init(ctx: &mut Context, _genesis_account: &signer, sequencer: address){
        let config = OnchainConfig{
            framework_version: 0,
            sequencer: sequencer
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
}
