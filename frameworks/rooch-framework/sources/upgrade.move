// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

module rooch_framework::upgrade {
    use std::vector;

    use moveos_std::gas_schedule::update_gas_schedule;
    use moveos_std::signer::module_signer;
    use moveos_std::signer;
    use moveos_std::event;
    use moveos_std::bcs;
    use moveos_std::account::create_signer_for_system;
    use moveos_std::module_store;
    use moveos_std::object;
    use rooch_framework::onchain_config;

    friend rooch_framework::genesis;

    const ErrorNotSequencer: u64 = 1;
    const ErrorCapabilityAlreadyExists: u64 = 2;
    const ErrorNoAccess: u64 = 3;

    const MoveStdAccount: address = @0x1;
    const MoveosStdAccount: address = @0x2;
    const RoochFrameworkAccount: address = @rooch_framework;
    const BitcoinMoveAccount: address = @0x4;

    #[data_struct]
    struct StdlibPackage has store, copy, drop {
        package_name: std::string::String,
        genesis_account: address,
        modules: vector<vector<u8>>,
    }

    #[data_struct]
    /// Collection of framework packages. The struct must keep the same with the Rust definition.
    struct Stdlib has store, copy, drop {
        packages: vector<StdlibPackage>,
    }

    /// Upgrade capability
    struct UpgradeCap has key, store {}

    /// Event for framework upgrades
    struct FrameworkUpgradeEvent has drop, store, copy {
        version: u64,
    }

    public(friend) fun genesis_init(sequencer: address){
        let cap_obj = object::new_named_object(UpgradeCap{});
        object::transfer(cap_obj, sequencer);
    }

    /// Aqcuires the upgrade capability for the sequencer.
    /// Only used for framework upgrading.
    /// TODO: remove this function when reset genesis.
    public fun new_upgrade_cap_for_upgrade(sequencer: &signer) {
        let sender_address = signer::address_of(sequencer);
        assert!(sender_address == onchain_config::sequencer(), ErrorNotSequencer);
        let id = object::named_object_id<UpgradeCap>();
        assert!(!object::exists_object(id), ErrorCapabilityAlreadyExists);
        genesis_init(sender_address);
    }

    fun has_upgrade_access(account: address): bool {
        let id = object::named_object_id<UpgradeCap>();
        if (!object::exists_object(id)) {
            return false
        };
        let cap = object::borrow_object<UpgradeCap>(id);
        object::owner(cap) == account
    }

    // Deprecated upgrade function. 
    // TODO: remove this function when reset genesis, and rename the following function to `upgrade_v2_entry` to `upgrade_entry`.
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

    /// Upgrade the framework package
    /// `package_bytes` is the serialized `StdlibPackage`
    entry fun upgrade_v2_entry(account: &signer, package_bytes: vector<u8>) {
        let sender_address = signer::address_of(account);
        assert!(has_upgrade_access(sender_address), ErrorNoAccess);
        let system = module_signer<FrameworkUpgradeEvent>();

        let stdlib = bcs::from_bytes<Stdlib>(package_bytes);
        let len = vector::length(&stdlib.packages);
        while (len > 0) {
            let package = vector::pop_back(&mut stdlib.packages);
            if (package.genesis_account == MoveStdAccount || 
                package.genesis_account == MoveosStdAccount ||
                package.genesis_account == RoochFrameworkAccount ||
                package.genesis_account == BitcoinMoveAccount) 
            {
                let package_signer = create_signer_for_system(&system, package.genesis_account);
                module_store::publish_modules_entry(&package_signer, package.modules);
            };
            len = len - 1;
        };

        onchain_config::update_framework_version();
        event::emit<FrameworkUpgradeEvent>(FrameworkUpgradeEvent { version: onchain_config::framework_version() });
    }

    entry fun upgrade_gas_schedule(account: &signer, gas_schedule_config: vector<u8>) {
        let sender_address = signer::address_of(account);
        assert!(sender_address == onchain_config::sequencer(), ErrorNotSequencer);

        let system = module_signer<FrameworkUpgradeEvent>();
        let moveos_std_signer = create_signer_for_system(&system, MoveosStdAccount);
        update_gas_schedule(&moveos_std_signer, gas_schedule_config);
    }
}
