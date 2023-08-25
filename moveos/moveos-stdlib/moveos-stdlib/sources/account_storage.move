/// AccountStorage is part of the StorageAbstraction
/// It is used to store the account's resources and modules

module moveos_std::account_storage {

    use std::string::String;
    use std::signer;
    use moveos_std::bcs;
    use std::vector;
    use moveos_std::type_table::{Self, TypeTable};
    use moveos_std::table::{Self, Table};
    use moveos_std::object;
    use moveos_std::object_id::{Self, ObjectID};
    use moveos_std::object_storage::{Self, ObjectStorage};
    use moveos_std::storage_context::{Self, StorageContext};
    use moveos_std::tx_context;
    use moveos_std::move_module::{Self, MoveModule};

    /// The account with the given address already exists
    const EAccountAlreadyExists: u64 = 0;

    /// The resource with the given type already exists
    const EResourceAlreadyExists: u64 = 1;
    /// The resource with the given type not exists 
    const EResourceNotExists: u64 = 2;

    const NamedTableResource: u64 = 0;
    const NamedTableModule: u64 = 1;

    struct AccountStorage has key {
        resources: TypeTable,
        modules: Table<String, MoveModule>,
    }

    // Used to indicate module upgrading in this tx and then 
    // setting mark_loader_cache_as_invalid() in VM, which announce to 
    // the VM that the code loading cache should be considered outdated. 
    struct ModuleUpgradeFlag has copy, drop, store {
        is_upgrade: bool,
    }

    //Ensure the NamedTableID generate use same method with Rust code
    public fun named_table_id(account: address, table_type: u64): ObjectID{
        object_id::address_to_object_id(tx_context::derive_id(bcs::to_bytes(&account), table_type))
    }

    /// Create a new account storage space
    public fun create_account_storage(ctx: &mut StorageContext, account: address) {
        let object_id = object_id::address_to_object_id(account);
        let account_storage = AccountStorage {
            resources: type_table::new_with_id(named_table_id(account, NamedTableResource)),
            modules: table::new_with_id(named_table_id(account, NamedTableModule)),
        };
        let object_storage = storage_context::object_storage_mut(ctx);
        assert!(!object_storage::contains(object_storage, object_id), EAccountAlreadyExists);
        let object = object::new_with_id(object_id, account, account_storage);
        object_storage::add(object_storage, object);
    }

    /// check if account storage eixst
    public fun exist_account_storage(ctx: &StorageContext, account: address): bool {
        let object_id = object_id::address_to_object_id(account);
        let object_storage = storage_context::object_storage(ctx);
        object_storage::contains(object_storage, object_id)
    }

    public fun ensure_account_storage(ctx: &mut StorageContext, account: address) {
        if (!exist_account_storage(ctx, account)) {
            create_account_storage(ctx, account);
        }
    }

    //TODO the resource and module table's id is determined by the account address, so we can use the account address to get the table id
    //And don't need to borrow the account storage from the object storage, but if we create the table every time, how to drop the table?
    fun borrow_account_storage(object_storage: &ObjectStorage, account: address): &AccountStorage{
        let object_id = object_id::address_to_object_id(account);
        let object = object_storage::borrow<AccountStorage>(object_storage, object_id);
        object::borrow(object)
    }

    fun borrow_account_storage_mut(object_storage: &mut ObjectStorage, account: address): &mut AccountStorage{
        let object_id = object_id::address_to_object_id(account);
        let object = object_storage::borrow_mut<AccountStorage>(object_storage, object_id);
        object::borrow_mut(object)
    }

    /// Borrow a resource from the AccountStorage
    fun borrow_resource_from_account_storage<T: key>(self: &AccountStorage): &T {
        type_table::borrow<T>(&self.resources)
    }

    /// Borrow a mut resource from the AccountStorage
    fun borrow_mut_resource_from_account_storage<T: key>(self: &mut AccountStorage): &mut T {
        type_table::borrow_mut<T>(&mut self.resources)
    }

    /// Add a resource to the account storage
    fun add_resource_to_account_storage<T: key>(self: &mut AccountStorage, resource: T){
        //TODO should let the type_table native add function to check the resource is exists?
        assert!(!type_table::contains<T>(&self.resources), EResourceAlreadyExists);
        type_table::add(&mut self.resources, resource);
    }

    /// Remove a resource from the account storage
    fun remove_resource_from_account_storage<T: key>(self: &mut AccountStorage): T {
        assert!(type_table::contains<T>(&self.resources), EResourceNotExists);
        type_table::remove<T>(&mut self.resources)
    }

    fun exists_resource_at_account_storage<T: key>(self: &AccountStorage) : bool {
        type_table::contains<T>(&self.resources)
    }

    fun exists_module_at_account_storage(self: &AccountStorage, name: String) : bool {
        table::contains(&self.modules, name)
    }

    // === Global storage functions ===

    #[private_generics(T)]
    /// Borrow a resource from the account's storage
    /// This function equates to `borrow_global<T>(address)` instruction in Move
    public fun global_borrow<T: key>(ctx: &StorageContext, account: address): &T {
        let object_storage = storage_context::object_storage(ctx);
        let account_storage = borrow_account_storage(object_storage, account);
        borrow_resource_from_account_storage<T>(account_storage)
    }

    #[private_generics(T)]
    /// Borrow a mut resource from the account's storage
    /// This function equates to `borrow_global_mut<T>(address)` instruction in Move
    public fun global_borrow_mut<T: key>(ctx: &mut StorageContext, account: address): &mut T {
        let object_storage = storage_context::object_storage_mut(ctx);
        let account_storage = borrow_account_storage_mut(object_storage, account);
        borrow_mut_resource_from_account_storage<T>(account_storage)
    }

    #[private_generics(T)]
    /// Move a resource to the account's storage
    /// This function equates to `move_to<T>(&signer, resource)` instruction in Move
    public fun global_move_to<T: key>(ctx: &mut StorageContext, account: &signer, resource: T){
        let account_address = signer::address_of(account);
        //Auto create the account storage when move resource to the account
        ensure_account_storage(ctx, account_address);
        let account_storage = borrow_account_storage_mut(storage_context::object_storage_mut(ctx), account_address);
        add_resource_to_account_storage(account_storage, resource);
    }

    #[private_generics(T)]
    /// Move a resource from the account's storage
    /// This function equates to `move_from<T>(address)` instruction in Move
    public fun global_move_from<T: key>(ctx: &mut StorageContext, account: address): T {
        let account_storage = borrow_account_storage_mut(storage_context::object_storage_mut(ctx), account);
        remove_resource_from_account_storage<T>(account_storage)
    }

    #[private_generics(T)]
    /// Check if the account has a resource of the given type
    /// This function equates to `exists<T>(address)` instruction in Move
    public fun global_exists<T: key>(ctx: &StorageContext, account: address) : bool {
        if (exist_account_storage(ctx, account)) {
            let account_storage = borrow_account_storage(storage_context::object_storage(ctx), account);
            exists_resource_at_account_storage<T>(account_storage)
        }else{
            false
        }
    }

    // ==== Module functions ====

    /// Check if the account has a module with the given name
    public fun exists_module(ctx: &StorageContext, account: address, name: String): bool {
        let account_storage = borrow_account_storage(storage_context::object_storage(ctx), account);
        exists_module_at_account_storage(account_storage, name) 
    }

    /// Publish modules to the account's storage
    public fun publish_modules(ctx: &mut StorageContext, account: &signer, modules: vector<MoveModule>) {
        let account_address = signer::address_of(account);
        let account_storage = borrow_account_storage_mut(storage_context::object_storage_mut(ctx), account_address);
        let i = 0;
        let len = vector::length(&modules);
        let (module_names, module_names_with_init_fn) = move_module::verify_modules(&modules, account_address);
        
        let upgrade_flag = false;
        while (i < len) {
            let name = vector::pop_back(&mut module_names);
            let m = vector::pop_back(&mut modules);   

            // The module already exists, which means we are upgrading the module
            if (table::contains(&account_storage.modules, name)) {
                let old_m = table::remove(&mut account_storage.modules, name);
                move_module::check_comatibility(&m, &old_m);
                upgrade_flag = true;
            } else {
                // request init function invoking
                move_module::request_init_functions(module_names_with_init_fn, account_address);
            };
            table::add(&mut account_storage.modules, name, m);
            i = i + 1;
        };
        
        // Store ModuleUpgradeFlag in tx_context which will be fetched in VM in Rust, 
        // and then announce to the VM that the code loading cache should be considered outdated. 
        let tx_ctx = storage_context::tx_context_mut(ctx); 
        if (!tx_context::contains<ModuleUpgradeFlag>(tx_ctx)) {
            tx_context::add(tx_ctx, ModuleUpgradeFlag { is_upgrade: upgrade_flag });
        }
    }
    
    public entry fun publish_modules_entry(ctx: &mut StorageContext, account: &signer, modules: vector<vector<u8>>) {
        let n_modules = vector::length(&modules);
        let i = 0;
        let module_vec = vector::empty<MoveModule>();
        while (i < n_modules) {
            let code_bytes = vector::pop_back(&mut modules);
            let m = move_module::new(code_bytes);
            vector::push_back(&mut module_vec, m);
            i = i + 1;
        };
        publish_modules(ctx, account, module_vec);
    }

    #[test]
    fun test_named_table_id() {
        assert!(named_table_id(@0xae43e34e51db9c833ab50dd9aa8b27106519e5bbfd533737306e7b69ef253647, NamedTableResource) == object_id::address_to_object_id(@0x04d8b5ccef4d5b55fa9371d1a9c344fcd4bd40dd9f32dd1d94696775fe3f3013), 1000);
        assert!(named_table_id(@0xae43e34e51db9c833ab50dd9aa8b27106519e5bbfd533737306e7b69ef253647, NamedTableModule) == object_id::address_to_object_id(@0xead64c5e724c9d52b0eb792b350d56001f1fe0dc2dec0e2e713420daba18109a), 1001);
    }

    #[test_only]
    struct Test has key{
        addr: address,
        version: u64
    }

    #[test(sender=@0x42)]
    fun test_account_storage(sender: signer){
        let sender_addr = signer::address_of(&sender);
        let ctx = storage_context::new_test_context(sender_addr);
        create_account_storage(&mut ctx, sender_addr);
        global_move_to(&mut ctx, &sender, Test{
            addr: sender_addr,
            version: 1,
        });
        storage_context::drop_test_context(ctx);
    }

    #[test(sender=@0x42)]
    fun test_move_to_account_storage(sender: signer){
        let sender_addr = signer::address_of(&sender);
        let ctx = storage_context::new_test_context(sender_addr);
        create_account_storage(&mut ctx, sender_addr);
        global_move_to(&mut ctx, &sender, Test{
            addr: sender_addr,
            version: 1,
        });
        storage_context::drop_test_context(ctx);
    }

    #[test(sender=@0x42)]
    fun test_move_from_account_storage(sender: signer){
        let sender_addr = signer::address_of(&sender);
        let ctx = storage_context::new_test_context(sender_addr);
        create_account_storage(&mut ctx, sender_addr);
        global_move_to(&mut ctx, &sender, Test{
            addr: sender_addr,
            version: 1,
        });
        let Test {
            addr,
            version
        } = global_move_from<Test>(&mut ctx, sender_addr);
        assert!(addr == sender_addr, 0x10);
        assert!(version == 1, 0x11);
        storage_context::drop_test_context(ctx);
    }

    #[test(sender=@0x42)]
    #[expected_failure(abort_code = 0x0, location = Self)]
    fun test_failure_repeatedly_create_account_storage(sender: signer){
        let sender_addr = signer::address_of(&sender);
        let ctx = storage_context::new_test_context(sender_addr);
        create_account_storage(&mut ctx, sender_addr);
        create_account_storage(&mut ctx, sender_addr);
        storage_context::drop_test_context(ctx);
    }

    #[test(sender=@0x42)]
    #[expected_failure(abort_code = 0x1, location = Self)]
    fun test_failure_repeatedly_move_to_account_storage(sender: signer){
        let sender_addr = signer::address_of(&sender);
        let ctx = storage_context::new_test_context(sender_addr);
        create_account_storage(&mut ctx, sender_addr);
        global_move_to(&mut ctx, &sender, Test{
            addr: sender_addr,
            version: 1,
        });
        global_move_to(&mut ctx, &sender, Test{
            addr: sender_addr,
            version: 1,
        });
        storage_context::drop_test_context(ctx);
    }

    #[test(sender=@0x42)]
    #[expected_failure(abort_code = 0x2, location = Self)]
    fun test_failure_repeatedly_move_from_account_storage(sender: signer){
        let sender_addr = signer::address_of(&sender);
        let ctx = storage_context::new_test_context(sender_addr);
        create_account_storage(&mut ctx, sender_addr);
        global_move_to(&mut ctx, &sender, Test{
            addr: sender_addr,
            version: 1,
        });
        let Test {
            addr: _,
            version: _
        } = global_move_from<Test>(&mut ctx, sender_addr);
        let Test {
            addr: _,
            version: _
        } = global_move_from<Test>(&mut ctx, sender_addr);
        storage_context::drop_test_context(ctx);
    }

    #[test(sender=@0x42)]
    fun test_global_borrow_account_storage(sender: signer){
        let sender_addr = signer::address_of(&sender);
        let ctx = storage_context::new_test_context(sender_addr);
        create_account_storage(&mut ctx, sender_addr);
        global_move_to(&mut ctx, &sender, Test{
            addr: sender_addr,
            version: 1,
        });

        let ref_test = global_borrow<Test>(& ctx, sender_addr);
        assert!( ref_test.version == 1, 1);
        assert!( ref_test.addr == sender_addr, 2);
        storage_context::drop_test_context(ctx);
    }

    #[test(sender=@0x42)]
    fun test_global_borrow_mut_account_storage(sender: signer){
        let sender_addr = signer::address_of(&sender);
        let ctx = storage_context::new_test_context(sender_addr);
        create_account_storage(&mut ctx, sender_addr);
        global_move_to(&mut ctx, &sender, Test{
            addr: sender_addr,
            version: 1,
        });

        let ref_test = global_borrow_mut<Test>(&mut ctx, sender_addr);
        assert!( ref_test.version == 1, 1);
        assert!( ref_test.addr == sender_addr, 2);

        ref_test.version = 2;
        assert!( ref_test.version == 2, 3);
        storage_context::drop_test_context(ctx);
    }

    #[test(sender=@0x42)]
    #[expected_failure(abort_code = 393218, location = moveos_std::raw_table)]
    fun test_failure_global_borrow_account_storage(sender: signer){
        let sender_addr = signer::address_of(&sender);
        let ctx = storage_context::new_test_context(sender_addr);
        create_account_storage(&mut ctx, sender_addr);
        global_borrow<Test>(&mut ctx, sender_addr);
        storage_context::drop_test_context(ctx);
    }

    #[test(sender=@0x42)]
    #[expected_failure(abort_code = 393218, location = moveos_std::raw_table)]
    fun test_failure_global_borrow_mut_account_storage(sender: signer){
        let sender_addr = signer::address_of(&sender);
        let ctx = storage_context::new_test_context(sender_addr);
        create_account_storage(&mut ctx, sender_addr);
        global_borrow_mut<Test>(&mut ctx, sender_addr);
        storage_context::drop_test_context(ctx);
    }

    #[test(sender=@0x42)]
    fun test_exist_account_storage(sender: signer){
        let sender_addr = signer::address_of(&sender);
        let ctx = storage_context::new_test_context(sender_addr);
        assert!(exist_account_storage(&ctx , sender_addr) == false, 1);
        storage_context::drop_test_context(ctx);
    }

    #[test(sender=@0x42)]
    fun test_ensure_account_storage(sender: signer){
        let sender_addr = signer::address_of(&sender);
        let ctx = storage_context::new_test_context(sender_addr);
        ensure_account_storage(&mut ctx , sender_addr);
        assert!(exist_account_storage(&ctx , sender_addr), 1);
        storage_context::drop_test_context(ctx);
    }

    #[test(sender=@0x42)]
    fun test_ensure_move_from_and_exists(sender: signer){
        let sender_addr = signer::address_of(&sender);
        let ctx = storage_context::new_test_context(sender_addr);
        let test_exists = global_exists<Test>(&ctx, sender_addr);
        assert!(!test_exists, 1);
        global_move_to(&mut ctx, &sender, Test{
            addr: sender_addr,
            version: 1,
        });
        let test_exists = global_exists<Test>(&ctx, sender_addr);
        assert!(test_exists, 2);
        let test = global_move_from<Test>(&mut ctx, sender_addr); 
        let test_exists = global_exists<Test>(&ctx, sender_addr);
        assert!(!test_exists, 3);
        let Test{
            addr: _,
            version: _
        } = test;
        storage_context::drop_test_context(ctx);
    }
}