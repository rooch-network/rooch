// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

module moveos_std::genesis {
    use std::option;
    use moveos_std::move_module;
    use moveos_std::features;
    use moveos_std::tx_context;
    use moveos_std::gas_schedule::{Self, GasScheduleConfig};

    const ErrorGenesisInit: u64 = 1;

    fun init(){
        move_module::init_module_store();
        features::init_feature_store();
        let gas_config_option = tx_context::get_attribute<GasScheduleConfig>();
        assert!(option::is_some(&gas_config_option), ErrorGenesisInit);
        gas_schedule::genesis_init(option::extract(&mut gas_config_option));
    }

    #[test_only]
    /// Just for get the genesis account
    struct GenesisContext{}

    #[test_only]
    /// init the genesis context for test
    public fun init_for_test(){
        let genesis_account = moveos_std::signer::module_signer<GenesisContext>();
        tx_context::add_attribute_via_system(&genesis_account, gas_schedule::new_gas_schedule_config(std::vector::empty()));
        init()
    }
}