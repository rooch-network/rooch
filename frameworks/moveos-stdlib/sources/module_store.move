// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

/// `module_store` provide object to manage packages and modules.
module moveos_std::module_store {
    use std::vector;
    use std::string::String;
    use moveos_std::core_addresses;
    use moveos_std::object::{Self, ObjectID, Object};
    use moveos_std::tx_context;
    use moveos_std::signer;
    use moveos_std::move_module::{Self, MoveModule};
    use moveos_std::features;
    use moveos_std::bcs;
    use moveos_std::event;

    friend moveos_std::genesis;
    
    /// Not allow to publish module
    const ErrorNotAllowToPublish: u64 = 1;
    /// Have no permission to upgrade package
    const ErrorNoUpgradePermission: u64 = 2;
    /// Upgrade cap issued already
    const ErrorUpgradeCapIssued: u64 = 3;

    /// Allowlist for module function invocation
    struct Allowlist has key, store {
        /// Allow list for packages
        packages: vector<address>,
    }

    /// Used to store packages.
    /// A package is an Object, and the package id is the module address.
    /// Packages are child objects of ModuleStore.
    struct ModuleStore has key {}

    /// Used to store modules.
    /// Modules are the Package's dynamic fields, with the module name as the key.
    struct Package has key {
        /// The package version, starts from 1.
        version: u64
    }

    #[data_struct]
    /// This is a data struct to store package data, which is the same with the Rust definition.
    /// When building package, the package data will be stored in this struct and be serialized,
    /// we then deserialize package in Move.
    struct PackageData has store, copy, drop {
        package_name: std::string::String,
        /// The address of the package to be published.
        /// This must be same as every module's address in the package.
        package_id: address,
        /// bytecode of modules.
        modules: vector<vector<u8>>,
    }

    /// Package upgrade capability
    struct UpgradeCap has key, store {
        /// Package id that the upgrade cap is issued for.
        package_id: address,
    }

    /// Event for package upgrades. New published modules will also trigger this event.
    struct UpgradeEvent has drop, store, copy {
        package_id: address,
        version: u64,
    }

    public fun module_store_id(): ObjectID {
        object::named_object_id<ModuleStore>()
    }

    /// Create a new module object space
    public(friend) fun init_module_store() {
        // The ModuleStore object will initialize before the genesis.

        let allowlist = object::new_named_object(Allowlist { packages: vector::empty() });
        object::to_shared(allowlist);
    }

    /// Issue an UpgradeCap for any package by the system accounts.
    public fun issue_upgrade_cap_by_system(system: &signer, package_id: address, owner: address) {
        core_addresses::assert_system_reserved(system);
        assert!(!is_upgrade_cap_issued(package_id), ErrorUpgradeCapIssued);
        let upgrade_cap_obj = object::new_account_named_object<UpgradeCap>(package_id, UpgradeCap { package_id });
        object::transfer_extend(upgrade_cap_obj, owner);
    }

    /// Issue an UpgradeCap for package under the sender's account. Then transfer the ownership to the owner.
    /// This is used to issue an upgrade cap before first publishing.
    public fun issue_upgrade_cap(sender: &signer, owner: address) {
        let package_id = signer::address_of(sender);
        assert!(!is_upgrade_cap_issued(package_id), ErrorUpgradeCapIssued);
        let upgrade_cap_obj = object::new_account_named_object<UpgradeCap>(package_id, UpgradeCap { package_id });
        object::transfer_extend(upgrade_cap_obj, owner);
    }

    public fun is_upgrade_cap_issued(package_id: address): bool {
        let id = object::account_named_object_id<UpgradeCap>(package_id);
        object::exists_object(id)
    }

    public fun borrow_module_store(): &Object<ModuleStore> {
        object::borrow_object(module_store_id())
    }

    public fun borrow_mut_module_store(): &mut Object<ModuleStore> {
        object::borrow_mut_object_shared(module_store_id())
    }

    // ==== Module functions ====
    
    public fun package_obj_id(package_id: address): ObjectID{
        let module_store_id = module_store_id();
        //Package object directly use the package address as the field key, do not need to hash
        object::child_id(module_store_id, package_id)
    }

    public fun exists_package(package_id: address): bool {
        let package_obj_id = package_obj_id(package_id);
        exists_package_obj(package_obj_id)
    }

    fun exists_package_obj(package_obj_id: ObjectID): bool{
        object::exists_object(package_obj_id)
    }

    /// Check if module exists
    /// package_id: the address of the package
    /// name: the name of the module
    public fun exists_module(package_id: address, name: String): bool {
        let package_obj_id = package_obj_id(package_id);
        if (!exists_package_obj(package_obj_id)) {
            return false
        };
        let package = borrow_package(package_obj_id);
        object::contains_field(package, name)
    }
  
    /// Entry function to publish package
    /// The order of modules must be sorted by dependency order.
    public entry fun publish_package_entry(account: &signer, package_bytes: vector<u8>) {
        let sender_address = signer::address_of(account);
        let package_data = bcs::from_bytes<PackageData>(package_bytes);

        // check if the package id is in allowlist
        if (features::module_publishing_allowlist_enabled()) {
            ensure_package_id_in_allowlist(package_data.package_id);
        };

        // check first publishing
        let first_publishing = !exists_package(package_data.package_id);
        if (first_publishing) {
            if (!is_upgrade_cap_issued(package_data.package_id)) {
                assert!(sender_address == package_data.package_id, ErrorNoUpgradePermission);
                let upgrade_cap_obj = object::new_account_named_object<UpgradeCap>(
                    package_data.package_id, UpgradeCap { package_id: package_data.package_id });
                object::transfer_extend(upgrade_cap_obj, sender_address);
            } else {
                assert!(has_upgrade_permission(package_data.package_id, sender_address), ErrorNoUpgradePermission);
            }
        } else {
            assert!(has_upgrade_permission(package_data.package_id, sender_address), ErrorNoUpgradePermission);
        };


        // convert module bytes to MoveModule
        let n_modules = vector::length(&package_data.modules);
        let i = 0;
        let module_vec = vector::empty<MoveModule>();
        while (i < n_modules) {
            let code_bytes = vector::pop_back(&mut package_data.modules);
            let m = move_module::new(code_bytes);
            vector::push_back(&mut module_vec, m);
            i = i + 1;
        };


        let module_store = borrow_mut_module_store(); 
        let upgrade_flag = publish_modules_internal(module_store, package_data.package_id, module_vec);
        // Store ModuleUpgradeFlag in tx_context which will be fetched in VM in Rust, 
        // and then announce to the VM that the code loading cache should be considered outdated. 
        tx_context::set_module_upgrade_flag(upgrade_flag);
    }

    public fun package_version(package_id: address): u64 {
        let package_obj_id = package_obj_id(package_id);
        let package = borrow_package(package_obj_id);
        let version = object::borrow(package).version;
        version
    }

    /// Publish modules to the module object's storage
    /// Return true if the modules are upgraded
    public(friend) fun publish_modules_internal(
        module_store_object: &mut Object<ModuleStore>, package_id: address, modules: vector<MoveModule>
    ) : bool {
        let i = 0;
        let len = vector::length(&modules);
        let (module_names, module_names_with_init_fn, indices) = move_module::sort_and_verify_modules(&modules, package_id);
        let package_obj_id = package_obj_id(package_id);
        let is_upgrade = true;
        if (!exists_package_obj(package_obj_id)) {
            //Note: the owner of the package is the package_id itself, not the tx sender.
            //Now, the package_id should be the same as the sender,
            //In the future, we will support publishing modules via DAO.
            let owner = package_id;
            create_package(module_store_object, package_id, owner);
            is_upgrade = false;
        };
        let package = borrow_mut_package(package_obj_id);

        while (i < len) {
            let module_name = vector::pop_back(&mut module_names);
            let index = vector::pop_back(&mut indices);
            let m = vector::borrow(&modules, index);

            // The module already exists, which means we are upgrading the module
            if (object::contains_field(package, module_name)) {
                let old_m = remove_module(package, module_name);
                move_module::check_comatibility(m, &old_m);
            } else {
                // request init function invoking
                if (vector::contains(&module_names_with_init_fn, &module_name)) {
                    let module_id = move_module::module_id_from_name(package_id, module_name);
                    move_module::request_init_functions(vector::singleton(module_id));
                }
            };
            add_module(package, module_name, *m);
            i = i + 1;
        };

        // version increment
        let version = version_increase(package);

        event::emit<UpgradeEvent>(UpgradeEvent { package_id, version });

        is_upgrade
    }

    fun create_package(module_store_object: &mut Object<ModuleStore>, package_id: address, owner: address) {
        //We directly use the package_id as the field key, do not need to hash
        let package = object::new_with_parent_and_key(module_store_object, package_id, Package { version: 0 });
        object::transfer_extend(package, owner);   
    }

    fun borrow_package(package_obj_id: ObjectID): &Object<Package> {
        object::borrow_object<Package>(package_obj_id)
    }

    fun borrow_mut_package(package_obj_id: ObjectID): &mut Object<Package> {
        object::borrow_mut_object_extend<Package>(package_obj_id)
    }

    fun add_module(package: &mut Object<Package>, name: String, mod: MoveModule) {
        object::add_field(package, name, mod);
    }

    fun remove_module(package: &mut Object<Package>, name: String): MoveModule {
        object::remove_field(package, name)
    }

    fun version_increase(package: &mut Object<Package>): u64 {
        let pkg = object::borrow_mut(package);
        pkg.version = pkg.version + 1;
        let version = pkg.version;
        version
    }

    fun borrow_allowlist(): &Allowlist {
        let allowlist_id = object::named_object_id<Allowlist>();
        let allowlist_obj = object::borrow_object(allowlist_id);
        object::borrow<Allowlist>(allowlist_obj)
    }

    fun borrow_mut_allowlist(): &mut Allowlist {
        let allowlist_id = object::named_object_id<Allowlist>();
        let allowlist_obj = object::borrow_mut_object_shared(allowlist_id);
        object::borrow_mut<Allowlist>(allowlist_obj)
    }

    public fun freeze_package(package: Object<Package>) {
        object::to_frozen(package);
    }

    /************************ allowlist functions *************************/

    /// Add a package id to the allowlist. Only package id in allowlist can publish modules.
    /// This is only valid when module_publishing_allowlist_enabled feature is enabled.
    public fun add_to_allowlist(account: &signer, package_id: address) {
        let sender = signer::address_of(account);
        core_addresses::assert_system_reserved_address(sender);
        
        let allowlist = borrow_mut_allowlist();
        if (!vector::contains(&allowlist.packages, &package_id)) {
            vector::push_back(&mut allowlist.packages, package_id);
        };
    }

    /// Remove a package id from the allowlist.
    public fun remove_from_allowlist(account: &signer, package_id: address) {
        let sender = signer::address_of(account);
        core_addresses::assert_system_reserved_address(sender);
        let allowlist = borrow_mut_allowlist();
        let _ = vector::remove_value(&mut allowlist.packages, &package_id);
    }

    /// Check if a package id is in the allowlist.
    public fun is_in_allowlist(package_id: address): bool {
        let allowlist = borrow_allowlist();
        vector::contains(&allowlist.packages, &package_id)
    }

    fun ensure_package_id_in_allowlist(package_id: address) {
        if (core_addresses::is_system_reserved_address(package_id)) {
            return
        };
        assert!(is_in_allowlist(package_id), ErrorNotAllowToPublish);
    }

    /// Check if the account has the permission to upgrade the package with the package_id.
    public fun has_upgrade_permission(package_id: address, account: address): bool {
        let id = object::account_named_object_id<UpgradeCap>(package_id);
        if (!object::exists_object(id)) {
            return false
        };
        let cap = object::borrow_object<UpgradeCap>(id);
        object::owner(cap) == account
    }

    /// Ensure the account has the permission to upgrade the package with the package_id.
    public fun ensure_upgrade_permission(package_id: address, account: &signer) {
        let has_permission = Self::has_upgrade_permission(package_id, signer::address_of(account));
        assert!(has_permission, ErrorNoUpgradePermission)
    }

    //The following is the bytes and hex of the compiled module: example/counter/sources/counter.move with account 0x42
    // Run the follow commands to get the bytecode of the module
    //./target/debug/rooch move build -p examples/counter -d
    //xxd -c 99999 -p examples/counter/build/counter/bytecode_modules/counter.mv
    #[test_only]
    const COUNTER_MV_BYTES: vector<u8> = x"a11ceb0b060000000b01000402040403082b04330605391c07557908ce0140068e02220ab002050cb502640d9903020000010100020c000003000000000400000000050100000006010000000700020001080506010c01090700010c010a0508010c0504060407040001060c01030107080001080001050107090002060c09000106090007636f756e746572076163636f756e7407436f756e74657208696e63726561736509696e6372656173655f04696e69740d696e69745f666f725f746573740576616c756513626f72726f775f6d75745f7265736f75726365106d6f76655f7265736f757263655f746f0f626f72726f775f7265736f757263650000000000000000000000000000000000000000000000000000000000000042000000000000000000000000000000000000000000000000000000000000000205200000000000000000000000000000000000000000000000000000000000000042000201070300010400000211010201010000030c070038000c000a00100014060100000000000000160b000f0015020200000000050b0006000000000000000012003801020301000000050b0006000000000000000012003801020401000000050700380210001402000000";

    #[test_only]
    fun drop_module_store(self: Object<ModuleStore>) {
        let ModuleStore {} = object::drop_unchecked(self);
    }

    #[test(account=@0x42)]
    fun test_publish_modules(account: &signer) {
        init_module_store();
        features::init_feature_store_for_test();
        let _ = account;
        let module_bytes = COUNTER_MV_BYTES;

        let pkg_data = PackageData {
            package_name: std::string::utf8(b"counter"),
            package_id: @0x42,
            modules: vector::singleton(module_bytes),
        };

        Self::publish_package_entry(account, bcs::to_bytes(&pkg_data));
        assert!(exists_module(@0x42, std::string::utf8(b"counter")), 1);
    }

    #[test(sender=@0x42)]
    #[expected_failure(abort_code = ErrorNotAllowToPublish, location = Self)]
    fun test_publish_modules_without_access(sender: &signer) {
        init_module_store();
        features::init_feature_store_for_test();
        features::change_feature_flags_for_test(
            vector[features::get_module_publishing_allowlist_feature()], 
            vector[]
        );
        let module_bytes = COUNTER_MV_BYTES;
        let pkg_data = PackageData {
            package_name: std::string::utf8(b"counter"),
            package_id: @0x42,
            modules: vector::singleton(module_bytes),
        };
        Self::publish_package_entry(sender, bcs::to_bytes(&pkg_data));
    }

    #[test(account=@0x42)]
    fun test_publish_modules_with_access(account: &signer) {
        init_module_store();
        features::init_feature_store_for_test();
        features::change_feature_flags_for_test(
            vector[features::get_module_publishing_allowlist_feature()], 
            vector[]
        );
        let system_account = signer::module_signer<Allowlist>();
        add_to_allowlist(&system_account, signer::address_of(account));

        let module_bytes = COUNTER_MV_BYTES;
        let pkg_data = PackageData {
            package_name: std::string::utf8(b"counter"),
            package_id: @0x42,
            modules: vector::singleton(module_bytes),
        };
        Self::publish_package_entry(account, bcs::to_bytes(&pkg_data));
    }

    #[test(_account=@moveos_std)]
    fun test_add_and_remove_allowlist(_account: &signer) {
        init_module_store();
        let system_account = signer::module_signer<Allowlist>();

        assert!(!is_in_allowlist(@0x42), 1);
        add_to_allowlist(&system_account, @0x42);
        assert!(is_in_allowlist(@0x42), 2);

        remove_from_allowlist(&system_account, @0x42);
        assert!(!is_in_allowlist(@0x42), 3);
    }

    #[test(account=@0x42)]
    #[expected_failure(abort_code = 9, location = object)]
    fun test_frozen_package(account: &signer) {
        init_module_store();
        features::init_feature_store_for_test();
        
        let module_bytes = COUNTER_MV_BYTES;

        let pkg_data = PackageData {
            package_name: std::string::utf8(b"counter"),
            package_id: @0x42,
            modules: vector::singleton(module_bytes),
        };
        let args = bcs::to_bytes(&pkg_data);
        Self::publish_package_entry(account, args);
        Self::publish_package_entry(account, args);

        let package_obj_id = package_obj_id(signer::address_of(account));
        let package = object::take_object_extend<Package>(package_obj_id);
        freeze_package(package);
        Self::publish_package_entry(account, args);
    }
}

