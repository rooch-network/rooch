// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

module rooch_framework::onchain_config {

    use std::vector;
    use std::ascii::String;
    use moveos_std::bcs;
    use moveos_std::tx_context;
    use moveos_std::object;
    use moveos_std::signer;
    use moveos_std::features;
    use rooch_framework::chain_id;

    friend rooch_framework::upgrade;
    friend rooch_framework::genesis;

    const ErrorNotSequencer: u64 = 1;
    const ErrorInvalidGasScheduleEntries: u64 = 2;

    struct GasScheduleUpdated has store, copy, drop {
        last_updated: u64
    }

    #[data_struct]
    struct GasEntry has store, copy, drop {
        key: String,
        val: u64,
    }

    struct GasSchedule has key {
        schedule_version: u64,
        entries: vector<GasEntry>,
    }

    #[data_struct]
    struct GasScheduleConfig has copy, drop, store{
        entries: vector<GasEntry>,
    }

    /// OnchainConfig is framework configurations stored on chain.
    struct OnchainConfig has key {
        framework_version: u64,
        sequencer: address,
    }

    public(friend) fun genesis_init(genesis_account: &signer, sequencer: address, gas_schedule_config: GasScheduleConfig){

        let gas_schedule = GasSchedule {
            schedule_version: 0,
            entries: gas_schedule_config.entries,
        };

        let config = OnchainConfig{
            framework_version: 0,
            sequencer,
        };
        let obj = object::new_named_object(config);
        object::transfer_extend(obj, @rooch_framework);

        let obj = object::new_named_object(gas_schedule);
        object::transfer_extend(obj, @rooch_framework);

        set_code_features(genesis_account);
    }

    public fun new_gas_schedule_config(entries: vector<GasEntry>): GasScheduleConfig {
        GasScheduleConfig {entries}
    }

    public fun new_gas_entry(key: String, val: u64): GasEntry {
        GasEntry {key, val}
    }

    public fun sequencer(): address {
        onchain_config().sequencer
    }

    public(friend) fun update_framework_version() {
        let config = onchain_config_mut();
        config.framework_version = config.framework_version + 1;
    }

    public fun framework_version(): u64 {
        onchain_config().framework_version
    }

    fun onchain_config_mut(): &mut OnchainConfig {
        let object_id = object::named_object_id<OnchainConfig>();
        let obj = object::borrow_mut_object_extend<OnchainConfig>(object_id);
        object::borrow_mut(obj)
    }

    public fun onchain_config(): &OnchainConfig {
        let object_id = object::named_object_id<OnchainConfig>();
        let obj = object::borrow_object<OnchainConfig>(object_id);
        object::borrow(obj)
    }

    entry fun update_onchain_gas_schedule_entry(account: &signer, gas_schedule_config: vector<u8>) {
        let sender_address = signer::address_of(account);
        assert!(sender_address == Self::sequencer(), ErrorNotSequencer);
        assert!(vector::length(&gas_schedule_config) > 0, ErrorInvalidGasScheduleEntries);

        let gas_schedule_config = bcs::from_bytes<GasScheduleConfig>(gas_schedule_config);
        update_onchain_gas_schedule(gas_schedule_config);
    }


    fun update_onchain_gas_schedule(gas_schedule_config: GasScheduleConfig) {

        let object_id = object::named_object_id<GasSchedule>();
        let obj = object::borrow_mut_object_extend<GasSchedule>(object_id);
        let gas_schedule = object::borrow_mut(obj);

        gas_schedule.schedule_version = gas_schedule.schedule_version + 1;
        gas_schedule.entries = gas_schedule_config.entries;

        let system = moveos_std::signer::module_signer<GasScheduleUpdated>();
        tx_context::add_attribute_via_system(&system, GasScheduleUpdated {last_updated: 1});
    }

    public fun onchain_gas_schedule(): &GasSchedule {
        let object_id = object::named_object_id<GasSchedule>();
        let obj = object::borrow_object<GasSchedule>(object_id);
        object::borrow(obj)
    }

    public fun gas_schedule_version(schedule: &GasSchedule): u64 {
        schedule.schedule_version
    }

    public fun gas_schedule_entries(schedule: &GasSchedule): &vector<GasEntry> {
        &schedule.entries
    }

    fun set_code_features(framework: &signer) {
        let enables = vector::empty<u64>();
        
        // TODO: change features
        if (chain_id::is_local()) {
            vector::push_back(&mut enables, features::get_localnet_feature());
            vector::push_back(&mut enables, features::get_devnet_feature());
            vector::push_back(&mut enables, features::get_testnet_feature());
        } else if (chain_id::is_dev()) {
            vector::push_back(&mut enables, features::get_devnet_feature());
            vector::push_back(&mut enables, features::get_testnet_feature());
        } else if (chain_id::is_test()) {
            vector::push_back(&mut enables, features::get_testnet_feature());
        };
        if (!chain_id::is_main()) {
            vector::push_back(&mut enables, features::get_module_template_feature());
        };

        features::change_feature_flags(framework, enables, vector[]);
    }

    #[test_only]
    public fun update_onchain_gas_schedule_for_testing(gas_schedule_config: GasScheduleConfig) {
        update_onchain_gas_schedule(gas_schedule_config);
    }
}
