/// AccountStorage is part of the StorageAbstraction
/// It is used to store the account's resources and modules

module moveos_std::account_storage {

    use std::string::String;
    use std::signer;
    use std::bcs;
    use std::vector;
    use moveos_std::type_table::{Self, TypeTable};
    use moveos_std::table::{Self, Table};
    use moveos_std::object;
    use moveos_std::object_storage::{Self, ObjectStorage};
    use moveos_std::storage_context::{Self, StorageContext};
    use moveos_std::tx_context;

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
        modules: Table<String, vector<u8>>,
    }

    //Ensure the NamedTableID generate use same method with Rust code
    fun named_table_id(account: address, table_type: u64): address{
        tx_context::derive_id(bcs::to_bytes(&account), table_type)
    }

    /// Create a new account storage space
    public fun create_account_storage(ctx: &mut StorageContext, account: address) {
        let object_id = object::address_to_object_id(account);
        let account_storage = AccountStorage {
            resources: type_table::new_with_id(named_table_id(account, NamedTableResource)),
            modules: table::new_with_id(named_table_id(account, NamedTableModule)),
        };
        let object_storage = storage_context::object_storage_mut(ctx);
        assert!(!object_storage::contains<AccountStorage>(object_storage, object_id), EAccountAlreadyExists);
        let object = object::new_with_id(object_id, account, account_storage);
        object_storage::add(object_storage, object);
    }

    /// check if account storage eixst
    public fun exist_account_storage(ctx: &mut StorageContext, account: address): bool {
        let object_id = object::address_to_object_id(account);
        let object_storage = storage_context::object_storage_mut(ctx);
        object_storage::contains<AccountStorage>(object_storage, object_id)
    }

    //TODO the resource and module table's id is determined by the account address, so we can use the account address to get the table id
    //And don't need to borrow the account storage from the object storage, but if we create the table every time, how to drop the table?
    fun borrow_account_storage(object_storage: &ObjectStorage, account: address): &AccountStorage{
        let object_id = object::address_to_object_id(account);
        let object = object_storage::borrow<AccountStorage>(object_storage, object_id);
        object::borrow(object)
    }

    fun borrow_account_storage_mut(object_storage: &mut ObjectStorage, account: address): &mut AccountStorage{
        let object_id = object::address_to_object_id(account);
        let object = object_storage::borrow_mut<AccountStorage>(object_storage, object_id);
        object::borrow_mut(object)
    }

    /// Borrow a resource from the AccountStorage
    fun borrow_resource_from_account_storage<T: key>(this: &AccountStorage): &T {
        type_table::borrow_internal<T>(&this.resources)
    }

    /// Borrow a mut resource from the AccountStorage
    fun borrow_mut_resource_from_account_storage<T: key>(this: &mut AccountStorage): &mut T {
        type_table::borrow_mut_internal<T>(&mut this.resources)
    }

    /// Add a resource to the account storage
    fun add_resource_to_account_storage<T: key>(this: &mut AccountStorage, resource: T){
        //TODO should let the type_table native add function to check the resource is exists?
        assert!(!type_table::contains_internal<T>(&this.resources), EResourceAlreadyExists);
        type_table::add_internal(&mut this.resources, resource);
    }

    /// Remove a resource from the account storage
    fun remove_resource_from_account_storage<T: key>(this: &mut AccountStorage): T{
        assert!(!type_table::contains_internal<T>(&this.resources), EResourceAlreadyExists);
        type_table::remove<T>(&mut this.resources)
    }

    fun exists_resource_at_account_storage<T: key>(this: &AccountStorage) : bool {
        type_table::contains<T>(&this.resources)
    }

    fun exists_module_at_account_storage(this: &AccountStorage, name: String) : bool {
        table::contains(&this.modules, name)
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
        let account_storage = borrow_account_storage(storage_context::object_storage(ctx), account);
        exists_resource_at_account_storage<T>(account_storage)
    }

    // ==== Module functions ====

    /// Check if the account has a module with the given name
    public fun exists_module(ctx: &StorageContext, account: address, name: String): bool {
        let account_storage = borrow_account_storage(storage_context::object_storage(ctx), account);
        exists_module_at_account_storage(account_storage, name) 
    }

    /// Publish modules to the account's storage
    public fun publish_modules(ctx: &mut StorageContext, account: &signer, modules: vector<vector<u8>>) {
        let account_address = signer::address_of(account);
        let account_storage = borrow_account_storage_mut(storage_context::object_storage_mut(ctx), account_address);
        let i = 0;
        let len = vector::length(&modules);
        let module_names = verify_modules(&modules, account_address);
        while (i < len) {
            let name = vector::pop_back(&mut module_names);
            let m = vector::pop_back(&mut modules);
            table::add(&mut account_storage.modules, name, m);
        }
    }

    // This is a native function that verifies the modules and returns their names
    // This function need to ensure the module's bytecode is valid and the module id is matching the account address.
    fun verify_modules(_modules: &vector<vector<u8>>, _account_address: address): vector<String> {
        //TODO implement native verify modules
        abort 0
    }
    
    #[test]
    fun test_named_table_id() {
        assert!(named_table_id(@0xae43e34e51db9c833ab50dd9aa8b27106519e5bbfd533737306e7b69ef253647, NamedTableResource) == @0x04d8b5ccef4d5b55fa9371d1a9c344fcd4bd40dd9f32dd1d94696775fe3f3013, 1000);
        assert!(named_table_id(@0xae43e34e51db9c833ab50dd9aa8b27106519e5bbfd533737306e7b69ef253647, NamedTableModule) == @0xead64c5e724c9d52b0eb792b350d56001f1fe0dc2dec0e2e713420daba18109a, 1001);
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
}