// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

module rooch_framework::onchain_config {

    use std::vector;
    use moveos_std::object;
    use moveos_std::features;
    use moveos_std::module_store;
    use moveos_std::signer;
    use rooch_framework::chain_id;

    friend rooch_framework::upgrade;
    friend rooch_framework::genesis;

    const ErrorNotAdmin: u64 = 1;

    /// OnchainConfig is framework configurations stored on chain.
    struct OnchainConfig has key {
        framework_version: u64,
        sequencer: address,
        rooch_dao: address,
    }

    /// ConfigUpdateCap is the capability for admin operations, such as update onchain configurations.
    struct ConfigUpdateCap has key, store {}


    public(friend) fun genesis_init(genesis_account: &signer, sequencer: address, rooch_dao: address){
        let config = OnchainConfig{
            framework_version: 0,
            sequencer,
            rooch_dao,
        };
        let obj = object::new_named_object(config);
        object::transfer_extend(obj, @rooch_framework);

        let admin_cap = object::new_named_object(ConfigUpdateCap{});
        object::transfer(admin_cap, rooch_dao);

        set_code_features(genesis_account);
    }

    public fun admin(): address {
        let object_id = object::named_object_id<ConfigUpdateCap>();
        let obj = object::borrow_object<ConfigUpdateCap>(object_id);
        object::owner(obj)
    }

    public fun ensure_admin(account: &signer) {
        let sender = signer::address_of(account);
        assert!(sender == admin(), ErrorNotAdmin);
    }
    
    public fun sequencer(): address {
        onchain_config().sequencer
    }

    public fun rooch_dao(): address {
        onchain_config().rooch_dao
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

    public fun exist_onchain_config(): bool {
        let object_id = object::named_object_id<OnchainConfig>();
        object::exists_object_with_type<OnchainConfig>(object_id)
    }

    /******  API for update module publishing allowlist. ******/

    /// When module_publishing_allowlist_feature is enabled, only packed_id in allowlist can be published.
    /// Add `package_id` to publishing allowlist.
    public entry fun add_to_publishing_allowlist(account: &signer, package_id: address) {
        ensure_admin(account);
        let system_account = signer::module_signer<OnchainConfig>();
        module_store::add_to_allowlist(&system_account, package_id);
    }

    /// Remove `package_id` from publishing allowlist.
    public entry fun remove_from_publishing_allowlist(account: &signer, package_id: address) {
        ensure_admin(account);
        let system_account = signer::module_signer<OnchainConfig>();
        module_store::remove_from_allowlist(&system_account, package_id);
    }
    /****** End of API for update module publishing allowlist. ******/

    /****** API for changing feature flags *******/

    /// Enable or disable features. You can find all feature flags in moveos_std::features.
    public entry fun change_feature_flags(account: &signer, enable: vector<u64>, disable: vector<u64>) {
        ensure_admin(account);
        let system_account = signer::module_signer<OnchainConfig>();
        features::change_feature_flags(&system_account, enable, disable);
    }

    /****** End of API for changing feature flags ******/


    fun onchain_config_mut(): &mut OnchainConfig {
        let object_id = object::named_object_id<OnchainConfig>();
        let obj = object::borrow_mut_object_extend<OnchainConfig>(object_id);
        object::borrow_mut(obj)
    }

    fun set_code_features(framework: &signer) {
        let enables = vector::empty<u64>();
        
        if (chain_id::is_local()) {
            vector::push_back(&mut enables, features::get_localnet_feature());
            vector::push_back(&mut enables, features::get_devnet_feature());
            vector::push_back(&mut enables, features::get_testnet_feature());
            vector::push_back(&mut enables, features::get_value_size_gas_feature());
            vector::push_back(&mut enables, features::get_compatibility_checker_v2_feature());
        } else if (chain_id::is_dev()) {
            vector::push_back(&mut enables, features::get_devnet_feature());
            vector::push_back(&mut enables, features::get_testnet_feature());
            vector::push_back(&mut enables, features::get_value_size_gas_feature());
            vector::push_back(&mut enables, features::get_compatibility_checker_v2_feature());
        } else if (chain_id::is_test()) {
            vector::push_back(&mut enables, features::get_testnet_feature());
            vector::push_back(&mut enables, features::get_value_size_gas_feature());
        };
        if (chain_id::is_main()) {
            vector::push_back(&mut enables, features::get_module_publishing_allowlist_feature());
        } else {
            vector::push_back(&mut enables, features::get_module_template_feature());
            vector::push_back(&mut enables, features::get_wasm_feature());
        };

        features::change_feature_flags(framework, enables, vector[]);
    }

}
