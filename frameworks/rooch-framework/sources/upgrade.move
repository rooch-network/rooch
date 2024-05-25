// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

module rooch_framework::upgrade {

    use moveos_std::signer::module_signer;
    use moveos_std::signer;
    use moveos_std::event;
    
    use moveos_std::account::create_signer_for_system;
    use moveos_std::module_store;
    use rooch_framework::onchain_config;

    const ErrorNotSequencer: u64 = 1;

    const MoveStdAccount: address = @0x1;
    const MoveosStdAccount: address = @0x2;
    const RoochFrameworkAccount: address = @rooch_framework;
    const BitcoinMoveAccount: address = @0x4;

    /// Event for framework upgrades
    struct FrameworkUpgradeEvent has drop, store, copy {
        version: u64,
    }

    entry fun upgrade_entry(account: &signer, 
        move_std_bundles: vector<vector<u8>>,
        moveos_std_bundles: vector<vector<u8>>,
        rooch_framework_bundles: vector<vector<u8>>,
        bitcoin_move_bundles: vector<vector<u8>>,
    ) {
        let sender_address = signer::address_of(account);
        assert!(sender_address == onchain_config::sequencer(), ErrorNotSequencer);

        let system = module_signer<FrameworkUpgradeEvent>();
        let std_signer = create_signer_for_system(&system, MoveStdAccount);
        module_store::publish_modules_entry(&std_signer, move_std_bundles);

        let moveos_std_signer = create_signer_for_system(&system, MoveosStdAccount);
        module_store::publish_modules_entry(&moveos_std_signer, moveos_std_bundles);

        let framework_signer = create_signer_for_system(&system, RoochFrameworkAccount);
        module_store::publish_modules_entry(&framework_signer, rooch_framework_bundles);

        let bitcoin_move_signer = create_signer_for_system(&system, BitcoinMoveAccount);
        module_store::publish_modules_entry(&bitcoin_move_signer, bitcoin_move_bundles);

        onchain_config::update_framework_version();
        event::emit<FrameworkUpgradeEvent>(FrameworkUpgradeEvent { version: onchain_config::framework_version() });
    }
}
