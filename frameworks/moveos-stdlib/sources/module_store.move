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

    friend moveos_std::genesis;
    
    /// Not allow to publish module
    const ErrorNotAllowToPublish: u64 = 1;

    /// Allowlist for module function invocation
    struct Allowlist has key, store {
        /// Allow list for publishing modules
        publisher: vector<address>,

    }

    /// Used to store packages.
    /// A package is an Object, and the package id is the module address.
    /// Packages are child objects of ModuleStore.
    struct ModuleStore has key {}

    /// Used to store modules.
    /// Modules are the Package's dynamic fields, with the module name as the key.
    struct Package has key {}

    public fun module_store_id(): ObjectID {
        object::named_object_id<ModuleStore>()
    }

    /// Create a new module object space
    public(friend) fun init_module_store() {
        // The ModuleStore object will initialize before the genesis.

        let allowlist = object::new_named_object(Allowlist { publisher: vector::empty() });
        object::to_shared(allowlist);
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

    /// Publish modules to the account's storage
    public fun publish_modules(module_store: &mut Object<ModuleStore>, account: &signer, modules: vector<MoveModule>) {
        let account_address = signer::address_of(account);
        if (features::module_publishing_allowlist_enabled()) {
            ensure_publisher_in_allowlist(account_address);
        };
        
        let upgrade_flag = publish_modules_internal(module_store, account_address, modules);
        // Store ModuleUpgradeFlag in tx_context which will be fetched in VM in Rust, 
        // and then announce to the VM that the code loading cache should be considered outdated. 
        tx_context::set_module_upgrade_flag(upgrade_flag);
    }
   
    /// Entry function to publish modules
    /// The order of modules must be sorted by dependency order.
    public entry fun publish_modules_entry(account: &signer, modules: vector<vector<u8>>) {
        let n_modules = vector::length(&modules);
        let i = 0;
        let module_vec = vector::empty<MoveModule>();
        while (i < n_modules) {
            let code_bytes = vector::pop_back(&mut modules);
            let m = move_module::new(code_bytes);
            vector::push_back(&mut module_vec, m);
            i = i + 1;
        };
        let module_store = borrow_mut_module_store(); 
        Self::publish_modules(module_store, account, module_vec);
    }

    /// Publish modules to the module object's storage
    /// Return true if the modules are upgraded
    public(friend) fun publish_modules_internal(
        module_object: &mut Object<ModuleStore>, package_id: address, modules: vector<MoveModule>
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
            create_package(module_object, package_id, owner);
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
        is_upgrade
    }

    fun create_package(module_object: &mut Object<ModuleStore>, package_id: address, owner: address) {
        //We directly use the package_id as the field key, do not need to hash
        let package = object::new_with_parent_and_key(module_object, package_id, Package {});
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

    /// Add an account to the allowlist. Only account in allowlist can publish modules.
    /// This is only valid when module_publishing_allowlist_enabled feature is enabled.
    public fun add_to_allowlist(account: &signer, publisher: address) {
        let sender = signer::address_of(account);
        core_addresses::assert_system_reserved_address(sender);
        
        let allowlist = borrow_mut_allowlist();
        if (!vector::contains(&allowlist.publisher, &publisher)) {
            vector::push_back(&mut allowlist.publisher, publisher);
        };
    }

    /// Remove an account from the allowlist.
    public fun remove_from_allowlist(account: &signer, publisher: address) {
        let sender = signer::address_of(account);
        core_addresses::assert_system_reserved_address(sender);
        let allowlist = borrow_mut_allowlist();
        let _ = vector::remove_value(&mut allowlist.publisher, &publisher);
    }

    /// Check if an account is in the allowlist.
    public fun is_in_allowlist(publisher: address): bool {
        let allowlist = borrow_allowlist();
        vector::contains(&allowlist.publisher, &publisher)
    }

    fun ensure_publisher_in_allowlist(publisher: address) {
        if (core_addresses::is_system_reserved_address(publisher)) {
            return
        };
        assert!(is_in_allowlist(publisher), ErrorNotAllowToPublish);
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
        let module_object = borrow_mut_module_store();
        let module_bytes = COUNTER_MV_BYTES;
        let m: MoveModule = move_module::new(module_bytes);

        Self::publish_modules(module_object, account, vector::singleton(m));
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
        let module_object = borrow_mut_module_store();
        let module_bytes = COUNTER_MV_BYTES;
        let m: MoveModule = move_module::new(module_bytes);
        Self::publish_modules(module_object, sender, vector::singleton(m));
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

        let module_object = borrow_mut_module_store();
        let module_bytes = COUNTER_MV_BYTES;
        let m: MoveModule = move_module::new(module_bytes);
        Self::publish_modules(module_object, account, vector::singleton(m));
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
        
        let module_object = borrow_mut_module_store();
        let module_bytes = COUNTER_MV_BYTES;
        let m: MoveModule = move_module::new(module_bytes);

        publish_modules(module_object, account, vector::singleton(m));
        publish_modules(module_object, account, vector::singleton(m));

        let package_obj_id = package_obj_id(signer::address_of(account));
        let package = object::take_object_extend<Package>(package_obj_id);
        freeze_package(package);
        publish_modules(module_object, account, vector::singleton(m));
    }
}

