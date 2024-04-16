// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

#[test_only]
module rooch_framework::onchain_config_test{
    use std::vector;
    use rooch_framework::onchain_config;

    #[test]
    fun test_gas_schedule() {
        rooch_framework::genesis::init_for_test();

        let gas_schedule = onchain_config::onchain_gas_schedule();
        assert!(vector::length(onchain_config::gas_schedule_entries(gas_schedule)) == 0, 1000);
        let entries = vector::empty();
        vector::push_back(&mut entries, onchain_config::new_gas_entry(std::ascii::string(b"test1"), 1));
        let gas_schedule_config = onchain_config::new_gas_schedule_config(entries);
        onchain_config::update_onchain_gas_schedule_for_testing(gas_schedule_config);
        let gas_schedule2 = onchain_config::onchain_gas_schedule();
        let entries2 = onchain_config::gas_schedule_entries(gas_schedule2);
        assert!(vector::length(entries2) == 1, 1002);
    }
}