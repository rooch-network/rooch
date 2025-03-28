/// Defines feature flags for Rooch frameworks. Those are used in implementations of features in
/// the moveos-stdlib, rooch-framework and other frameworks.
///
/// ============================================================================================
/// Feature Flag Definitions
///
/// Each feature flag should come with documentation which justifies the need of the flag.
/// Introduction of a new feature flag requires approval of framework owners. Be frugal when
/// introducing new feature flags, as too many can make it hard to understand the code.
///
/// Note that removing a feature flag still requires the function which tests for the feature
/// to stay around for compatibility reasons, as it is a public function. However, once the 
/// feature flag is disabled, those functions can constantly return true.
module moveos_std::features {
    use std::vector;

    use moveos_std::core_addresses;
    use moveos_std::object;

    friend moveos_std::genesis;

    const EINVALID_FEATURE: u64 = 1;
    const EAPI_DISABLED: u64 = 2;

    /// The enabled features, represented by a bitset stored on chain.
    struct FeatureStore has key {
        features: vector<u8>,
    }

    public(friend) fun init_feature_store() {
        let feature_store = object::new_named_object(FeatureStore { features: vector::empty() });
        object::to_shared(feature_store);
    }

    #[test_only]
    public fun init_feature_store_for_test() {
        init_feature_store();
    }

    /// Enable or disable features. Only the framework signers can call this function.
    public fun change_feature_flags(framework: &signer, enable: vector<u64>, disable: vector<u64>) {
        core_addresses::assert_system_reserved(framework);
        change_feature_flags_internal(enable, disable);
    }

    #[test_only]
    public fun change_feature_flags_for_test(enable: vector<u64>, disable: vector<u64>) {
        change_feature_flags_internal(enable, disable);
    }
    #[test_only]
    public fun init_and_enable_all_features_for_test() {
        init_feature_store_for_test();
        change_feature_flags_internal(get_all_features(), vector::empty<u64>());
    }

    fun change_feature_flags_internal(enable: vector<u64>, disable: vector<u64>) {
        let features = borrow_mut_features();
        vector::for_each_ref(&enable, |feature| {
            set(features, *feature, true);
        });
        vector::for_each_ref(&disable, |feature| {
            set(features, *feature, false);
        });
    }

    /// Check whether the feature is enabled.
    /// All features are enabled for system reserved accounts.
    public fun is_enabled(feature: u64): bool {
        contains(borrow_features(), feature)
    }

    // --------------------------------------------------------------------------------------------
    // Available flags

    /// This feature will only be enabled on localnet.
    const LOCALNET: u64 = 1;
    public fun get_localnet_feature(): u64 { LOCALNET }
    public fun localnet_enabled(): bool {
        is_enabled(LOCALNET)
    }
    public fun ensure_localnet_enabled() {
        assert!(is_enabled(LOCALNET), EAPI_DISABLED);
    }

    /// This feature will only be enabled on devnet.
    const DEVNET: u64 = 2;
    public fun get_devnet_feature(): u64 { DEVNET }
    public fun devnet_enabled(): bool {
        is_enabled(DEVNET)
    }
    public fun ensure_devnet_enabled() {
        assert!(is_enabled(DEVNET), EAPI_DISABLED);
    }

    /// This feature will only be enabled on testnet, devnet or localnet.
    const TESTNET: u64 = 3;
    public fun get_testnet_feature(): u64 { TESTNET }
    public fun testnet_enabled(): bool {
        is_enabled(TESTNET)
    }
    public fun ensure_testnet_enabled() {
        assert!(is_enabled(TESTNET), EAPI_DISABLED);
    }

    /// Whether allowing replacing module's address, module identifier, struct identifier 
    /// and constant address.
    /// This feature is used for creating a new module through a module template bytes,
    /// thus developers can used to publish new modules in Move.
    const MODULE_TEMPLATE: u64 = 4;
    public fun get_module_template_feature(): u64 { MODULE_TEMPLATE }
    public fun module_template_enabled(): bool {
        is_enabled(MODULE_TEMPLATE)
    }
    public fun ensure_module_template_enabled() {
        assert!(is_enabled(MODULE_TEMPLATE), EAPI_DISABLED);
    }

    /// Whether enable the allowlist feature for publishing modules.
    const MODULE_PUBLISHING_ALLOWLIST: u64 = 5;
    public fun get_module_publishing_allowlist_feature(): u64 { MODULE_PUBLISHING_ALLOWLIST }
    public fun module_publishing_allowlist_enabled(): bool {
        is_enabled(MODULE_PUBLISHING_ALLOWLIST)
    }
    public fun ensure_module_publishing_allowlist_enabled() {
        assert!(is_enabled(MODULE_PUBLISHING_ALLOWLIST), EAPI_DISABLED);
    }

    /// Whether enable the wasm feature.
    const WASM: u64 = 6;
    public fun get_wasm_feature(): u64 { WASM }
    public fun wasm_enabled(): bool {
        is_enabled(WASM)
    }
    public fun ensure_wasm_enabled() {
        assert!(is_enabled(WASM), EAPI_DISABLED);
    }

    /// Whether to enable size-based gas fee for adding field values
    const VALUE_SIZE_GAS: u64 = 7;
    public fun get_value_size_gas_feature(): u64 { VALUE_SIZE_GAS }
    public fun value_size_gas_enabled(): bool {
        is_enabled(VALUE_SIZE_GAS)
    }
    public fun ensure_value_size_gas_enabled() {
        assert!(is_enabled(VALUE_SIZE_GAS), EAPI_DISABLED);
    }

    /// Whether to enable compatibility checker v2
    const COMPATIBILITY_CHECKER_V2: u64 = 8;
    public fun get_compatibility_checker_v2_feature(): u64 { COMPATIBILITY_CHECKER_V2 }
    public fun compatibility_checker_v2_enabled(): bool {
        is_enabled(COMPATIBILITY_CHECKER_V2)
    }
    public fun ensure_compatibility_checker_v2_enabled() {
        assert!(is_enabled(COMPATIBILITY_CHECKER_V2), EAPI_DISABLED);
    }

    /// Helper for getting all features. 
    /// Update this once new feature added.
    public fun get_all_features(): vector<u64> {
        vector[
            LOCALNET,
            DEVNET,
            TESTNET,
            MODULE_TEMPLATE,
            MODULE_PUBLISHING_ALLOWLIST,
            WASM,
            VALUE_SIZE_GAS,
            COMPATIBILITY_CHECKER_V2,
        ]
    }
    // --------------------------------------------------------------------------------------------
    // Helpers

    fun borrow_features(): &vector<u8> {
        let feature_store_id = object::named_object_id<FeatureStore>();
        let feature_store = object::borrow_mut_object_shared<FeatureStore>(feature_store_id);
        &object::borrow(feature_store).features
    }

    fun borrow_mut_features(): &mut vector<u8> {
        let feature_store_id = object::named_object_id<FeatureStore>();
        let feature_store = object::borrow_mut_object_shared<FeatureStore>(feature_store_id);
        &mut object::borrow_mut(feature_store).features
    }

    /// Helper to include or exclude a feature flag.
    fun set(features: &mut vector<u8>, feature: u64, include: bool) {
        let byte_index = feature / 8;
        let bit_mask = 1 << ((feature % 8) as u8);
        while (vector::length(features) <= byte_index) {
            vector::push_back(features, 0)
        };
        let entry = vector::borrow_mut(features, byte_index);
        if (include)
            *entry = *entry | bit_mask
        else
            *entry = *entry & (0xff ^ bit_mask)
    }

        /// Helper to check whether a feature flag is enabled.
    fun contains(features: &vector<u8>, feature: u64): bool {
        let byte_index = feature / 8;
        let bit_mask = 1 << ((feature % 8) as u8);
        byte_index < vector::length(features) && (*vector::borrow(features, byte_index) & bit_mask) != 0
    }

    // --------------------------------------------------------------------------------------------
    // Tests

    #[test]
    fun test_feature_sets() {
        let features = vector[];
        set(&mut features, 1, true);
        set(&mut features, 5, true);
        set(&mut features, 17, true);
        set(&mut features, 23, true);
        assert!(contains(&features, 1), 0);
        assert!(contains(&features, 5), 1);
        assert!(contains(&features, 17), 2);
        assert!(contains(&features, 23), 3);
        set(&mut features, 5, false);
        set(&mut features, 17, false);
        assert!(contains(&features, 1), 0);
        assert!(!contains(&features, 5), 1);
        assert!(!contains(&features, 17), 2);
        assert!(contains(&features, 23), 3);
    }

    #[test(fx = @moveos_std)]
    fun test_change_feature_txn(fx: signer) {
        init_feature_store_for_test();
        change_feature_flags(&fx, vector[1, 9, 23], vector[]);
        assert!(is_enabled(1), 1);
        assert!(is_enabled(9), 2);
        assert!(is_enabled(23), 3);
        change_feature_flags(&fx, vector[17], vector[9]);
        assert!(is_enabled(1), 1);
        assert!(!is_enabled(9), 2);
        assert!(is_enabled(17), 3);
        assert!(is_enabled(23), 4);
    }

}