// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

module rooch_framework::onchain_config {

    use std::vector;
    use moveos_std::object;
    use moveos_std::features;
    use moveos_std::move_module;
    use moveos_std::signer;
    use rooch_framework::chain_id;

    friend rooch_framework::upgrade;
    friend rooch_framework::genesis;

    const ErrorNotSequencer: u64 = 1;

    /// OnchainConfig is framework configurations stored on chain.
    struct OnchainConfig has key {
        framework_version: u64,
        sequencer: address,
    }

    public(friend) fun genesis_init(genesis_account: &signer, sequencer: address){
        let config = OnchainConfig{
            framework_version: 0,
            sequencer,
        };
        let obj = object::new_named_object(config);
        object::transfer_extend(obj, @rooch_framework);
        set_code_features(genesis_account);
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

    public fun onchain_config(): &OnchainConfig {
        let object_id = object::named_object_id<OnchainConfig>();
        let obj = object::borrow_object<OnchainConfig>(object_id);
        object::borrow(obj)
    }

    /******  APIs for update module publishing allowlist. ******/
    public entry fun add_to_publishing_allowlist(account: &signer, publisher: address) {
        let sender = signer::address_of(account);
        assert!(sender == sequencer(), ErrorNotSequencer);
        let system_account = signer::module_signer<OnchainConfig>();
        let allowlist = move_module::borrow_mut_allowlist();
        move_module::add_to_allowlist(allowlist, &system_account, publisher);
    }

    public entry fun remove_from_publishing_allowlist(account: &signer, publisher: address) {
        let sender = signer::address_of(account);
        assert!(sender == sequencer(), ErrorNotSequencer);
        let system_account = signer::module_signer<OnchainConfig>();
        let allowlist = move_module::borrow_mut_allowlist();
        move_module::remove_from_allowlist(allowlist, &system_account, publisher);
    }

    fun onchain_config_mut(): &mut OnchainConfig {
        let object_id = object::named_object_id<OnchainConfig>();
        let obj = object::borrow_mut_object_extend<OnchainConfig>(object_id);
        object::borrow_mut(obj)
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
        } else {
            vector::push_back(&mut enables, features::get_module_publishing_allowlist_feature());
        };

        features::change_feature_flags(framework, enables, vector[]);
    }

}
