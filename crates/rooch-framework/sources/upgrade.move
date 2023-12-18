// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

module rooch_framework::upgrade {

    use std::signer;
    use moveos_std::event;
    use moveos_std::context::{Self, Context};
    use rooch_framework::onchain_config;

    const ErrorNotSequencer: u64 = 1;

    /// Event for framework upgrades
    struct FrameworkUpgradeEvent has drop, store {
        version: u64,
    }

    entry fun upgrade_entry(ctx: &mut Context, account: &signer, modules: vector<vector<u8>>) {
        let sender_address = signer::address_of(account);
        assert!(sender_address == onchain_config::sequencer(ctx), ErrorNotSequencer);
        context::publish_modules_entry(ctx, account, modules);
        onchain_config::update_framework_version(ctx);

        event::emit<FrameworkUpgradeEvent>(FrameworkUpgradeEvent { version: onchain_config::framework_version(ctx) });
    }

//    struct U8Event has drop, store {
//         value: u8
//    }
//    public entry fun emit_u8(_ctx: &mut Context, value: u8) {
//       event::emit<U8Event>(U8Event { value });
//    }
}
