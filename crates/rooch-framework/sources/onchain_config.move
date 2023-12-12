// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

module rooch_framework::onchain_config {

    use moveos_std::context::{Self, Context};
    use moveos_std::object;

    /// OnchainConfig is framework configurations stored on chain.
    struct OnchainConfig has copy,store,drop{
        sequencer: address,
    }

    public(friend) fun genesis_init(ctx: &mut Context, _genesis_account: &signer, sequencer: address){
        let config = OnchainConfig{
            sequencer: sequencer
        };
        let obj = context::new_named_object(ctx, config);
        object::transfer_extend(obj, @rooch_framework);
    }
}
