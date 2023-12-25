// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

module rooch_framework::upgrade {

    use moveos_std::signer;
    use moveos_std::event;
    use moveos_std::context::{Self, Context};
    use rooch_framework::onchain_config;
    use rooch_framework::account::create_signer;

    const ErrorNotSequencer: u64 = 1;

    const MoveStdAccount: address = @0x1;
    const MoveosStdAccount: address = @0x2;
    const RoochFrameworkAccount: address = @rooch_framework;
    const BitcoinMoveAccount: address = @0x4;

    /// Event for framework upgrades
    struct FrameworkUpgradeEvent has drop, store {
        version: u64,
    }

    entry fun upgrade_entry(ctx: &mut Context, account: &signer, 
        move_std_bundles: vector<vector<u8>>,
        moveos_std_bundles: vector<vector<u8>>,
        rooch_framework_bundles: vector<vector<u8>>,
        bitcoin_move_bundles: vector<vector<u8>>,
    ) {
        let sender_address = signer::address_of(account);
        assert!(sender_address == onchain_config::sequencer(ctx), ErrorNotSequencer);

        let std_signer = create_signer(MoveStdAccount);
        context::publish_modules_entry(ctx, &std_signer, move_std_bundles);

        let moveos_std_signer = create_signer(MoveosStdAccount);
        context::publish_modules_entry(ctx, &moveos_std_signer, moveos_std_bundles);

        let framework_signer = create_signer(RoochFrameworkAccount);
        context::publish_modules_entry(ctx, &framework_signer, rooch_framework_bundles);

        let bitcoin_move_signer = create_signer(BitcoinMoveAccount);
        context::publish_modules_entry(ctx, &bitcoin_move_signer, bitcoin_move_bundles);

        onchain_config::update_framework_version(ctx);
        event::emit<FrameworkUpgradeEvent>(FrameworkUpgradeEvent { version: onchain_config::framework_version(ctx) });
    }
}
