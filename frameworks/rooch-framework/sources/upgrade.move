// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

module rooch_framework::upgrade {

    use moveos_std::gas_schedule::update_gas_schedule;
    use moveos_std::signer::module_signer;
    use moveos_std::account::create_signer_for_system;
    use rooch_framework::onchain_config;

    friend rooch_framework::genesis;

    const MoveosStdAccount: address = @0x2;

    /// Event for framework upgrades
    struct GasUpgradeEvent has drop, store, copy {
    }

    entry fun upgrade_gas_schedule(account: &signer, gas_schedule_config: vector<u8>) {
        onchain_config::ensure_admin(account);

        let system = module_signer<GasUpgradeEvent>();
        let moveos_std_signer = create_signer_for_system(&system, MoveosStdAccount);
        update_gas_schedule(&moveos_std_signer, gas_schedule_config);
    }
}
