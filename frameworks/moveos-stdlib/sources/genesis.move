// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

module moveos_std::genesis {
    use std::option;
    use moveos_std::module_store;
    use moveos_std::features;
    use moveos_std::tx_context;
    use moveos_std::gas_schedule::{Self, GasScheduleConfig};
    use moveos_std::timestamp;

    const ErrorGenesisInit: u64 = 1;

    /// GenesisContext is a genesis init parameters in the TxContext.
    struct GenesisContext has copy,store,drop{
        /// genesis timestamp in microseconds
        timestamp: u64,
    }

    fun init(){
        
        let genesis_context_option = tx_context::get_attribute<GenesisContext>();
        assert!(option::is_some(&genesis_context_option), ErrorGenesisInit);
        let genesis_context = option::extract(&mut genesis_context_option);
        //Ensure the genesis timestamp.
        assert!(genesis_context.timestamp == timestamp::now_milliseconds(), ErrorGenesisInit);
        module_store::init_module_store();
        features::init_feature_store();
        let gas_config_option = tx_context::get_attribute<GasScheduleConfig>();
        assert!(option::is_some(&gas_config_option), ErrorGenesisInit);
        gas_schedule::genesis_init(option::extract(&mut gas_config_option));
    }

    #[test_only]
    /// init the genesis context for test
    public fun init_for_test(){
        let genesis_account = moveos_std::signer::module_signer<GenesisContext>();
        tx_context::add_attribute_via_system(&genesis_account, GenesisContext{timestamp: 0});
        tx_context::add_attribute_via_system(&genesis_account, gas_schedule::new_gas_schedule_config(gas_schedule::initial_max_gas_amount(),std::vector::empty()));
        init()
    }
}