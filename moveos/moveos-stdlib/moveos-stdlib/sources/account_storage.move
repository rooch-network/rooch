/// AccountStorage is part of the StorageAbstraction
/// It is used to store the account's resources and modules

module moveos_std::account_storage {

    use std::string::String;
    use std::signer;
    use std::bcs;
    use moveos_std::type_table::{Self, TypeTable};
    use moveos_std::table::{Self, Table};
    use moveos_std::object::{Self, Object};
    use moveos_std::object_storage::{Self, ObjectStorage};
    use moveos_std::storage_context::{Self, StorageContext};
    use moveos_std::tx_context;

    /// The account with the given address already exists
    const EAccountAlreadyExists: u64 = 0;

    /// The resource with the given type already exists
    const EResourceAlreadyExists: u64 = 1;

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

    fun borrow_account_storage(object_storage: &ObjectStorage, account: address): &Object<AccountStorage>{
        let object_id = object::address_to_object_id(account);
        object_storage::borrow<AccountStorage>(object_storage, object_id)
    }

    fun borrow_account_storage_mut(object_storage: &mut ObjectStorage, account: address): &mut Object<AccountStorage>{
        let object_id = object::address_to_object_id(account);
        object_storage::borrow_mut<AccountStorage>(object_storage, object_id)
    }

    /// Borrow a resource from the AccountStorage
    fun borrow_resource<T: key>(this: &Object<AccountStorage>): &T {
        let account_storage = object::borrow<AccountStorage>(this);
        type_table::borrow<T>(&account_storage.resources)
    }

    /// Borrow a mut resource from the AccountStorage
    fun borrow_mut_resource<T: key>(this: &mut Object<AccountStorage>): &mut T {
        let account_storage = object::borrow_mut<AccountStorage>(this);
        type_table::borrow_mut<T>(&mut account_storage.resources)
    }

    /// Add a resource to the account storage
    fun add_resource<T: key>(this: &mut Object<AccountStorage>, resource: T){
        let account_storage = object::borrow_mut(this);
        assert!(!type_table::contains<T>(&account_storage.resources), EResourceAlreadyExists);
        type_table::add(&mut account_storage.resources, resource);
    }

    /// Remove a resource from the account storage
    fun remove_resource<T: key>(this: &mut Object<AccountStorage>): T{
        let account_storage = object::borrow_mut(this);
        assert!(!type_table::contains<T>(&account_storage.resources), EResourceAlreadyExists);
        type_table::remove<T>(&mut account_storage.resources)
    }

    fun exists_resource<T: key>(this: &Object<AccountStorage>) : bool {
        let account_storage = object::borrow(this);
        type_table::contains<T>(&account_storage.resources)
    }

    fun exists_module(this: &Object<AccountStorage>, name: String) : bool {
        let account_storage = object::borrow(this);
        table::contains(&account_storage.modules, name)
    }

    // === Global storage functions ===

    #[private_generic(T)]
    /// Borrow a resource from the account's storage
    /// This function equates to `borrow_global<T>(address)` instruction in Move
    public fun global_borrow<T: key>(ctx: &StorageContext, account: address): &T {
        let object_storage = storage_context::object_storage(ctx);
        let account_storage = borrow_account_storage(object_storage, account);
        borrow_resource<T>(account_storage)
    }

    #[private_generic(T)]
    /// Borrow a mut resource from the account's storage
    /// This function equates to `borrow_global_mut<T>(address)` instruction in Move
    public fun global_borrow_mut<T: key>(ctx: &mut StorageContext, account: address): &mut T {
        let object_storage = storage_context::object_storage_mut(ctx);
        let account_storage = borrow_account_storage_mut(object_storage, account);
        borrow_mut_resource<T>(account_storage)
    }

    #[private_generic(T)]
    /// Move a resource to the account's storage
    /// This function equates to `move_to<T>(&signer, resource)` instruction in Move
    public fun global_move_to<T: key>(ctx: &mut StorageContext, account: &signer, resource: T){
        let account_address = signer::address_of(account);
        let account_storage = borrow_account_storage_mut(storage_context::object_storage_mut(ctx), account_address);
        add_resource(account_storage, resource);
    }

    #[private_generic(T)]
    /// Move a resource from the account's storage
    /// This function equates to `move_from<T>(address)` instruction in Move
    public fun global_move_from<T: key>(ctx: &mut StorageContext, account: address): T {
        let account_storage = borrow_account_storage_mut(storage_context::object_storage_mut(ctx), account);
        remove_resource<T>(account_storage)
    }

    #[private_generic(T)]
    /// Check if the account has a resource of the given type
    /// This function equates to `exists<T>(address)` instruction in Move
    public fun global_exists<T: key>(ctx: &mut StorageContext, account: address) : bool {
        let account_storage = borrow_account_storage(storage_context::object_storage(ctx), account);
        exists_resource<T>(account_storage)
    }

    // ==== Module functions ====

    //TODO find better name.
    /// Check if the account has a module with the given name
    public fun module_exists(ctx: &mut StorageContext, account: address, name: String): bool {
        let account_storage = borrow_account_storage(storage_context::object_storage(ctx), account);
        exists_module(account_storage, name) 
    }
    
    #[test]
    fun test_named_table_id() {
        assert!(named_table_id(@0xae43e34e51db9c833ab50dd9aa8b27106519e5bbfd533737306e7b69ef253647, NamedTableResource) == @0x04d8b5ccef4d5b55fa9371d1a9c344fcd4bd40dd9f32dd1d94696775fe3f3013, 1000);
        assert!(named_table_id(@0xae43e34e51db9c833ab50dd9aa8b27106519e5bbfd533737306e7b69ef253647, NamedTableModule) == @0xead64c5e724c9d52b0eb792b350d56001f1fe0dc2dec0e2e713420daba18109a, 1001);
    }
}