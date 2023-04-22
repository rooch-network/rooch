/// AccountStorage is part of the StorageAbstraction
/// It is used to store the account's resources and modules

module moveos_std::account_storage {

    use std::string::String;
    use moveos_std::any_table;
    use moveos_std::table;
    use moveos_std::type_info::{Self, TypeInfo};
    use moveos_std::object::{Self, Object, ObjectStore};
    use moveos_std::storage_context::{Self,StorageContext};

    /// The account with the given address already exists
    const EAccountAlreadyExists: u64 = 0;

    /// The resource with the given type already exists
    const EResourceAlreadyExists: u64 = 1;

    struct AccountStorage has key {
        resources: any_table::Table<TypeInfo>,
        modules: table::Table<String, vector<u8>>,
    }

    /// Create a new account storage space
    public fun create_account_storage(ctx: &mut StorageContext, account: address) {
        let object_id = object::address_to_object_id(account);
        let tx_ctx = storage_context::tx_context_mut(ctx);
        let account_storage = AccountStorage {
            resources: any_table::new(tx_ctx),
            modules: table::new(),
        };
        let object_store = storage_context::object_store_mut(ctx);
        assert!(!object::contains(object_store, object_id), EAccountAlreadyExists);
        let object = object::new_with_id(object_id, account, account_storage);
        object::add(object_store, object);
    }

    public fun borrow_account_storage(object_store: &ObjectStore, account: address): &Object<AccountStorage>{
        let object_id = object::address_to_object_id(account);
        object::borrow<AccountStorage>(object_store, object_id)
    }

    /// Borrow a resource from the account storage
    /// This function equates to `borrow_global<T>(account)` instruction in Move
    public fun borrow_resource<T: key>(this: &Object<AccountStorage>): &T {
        let account_storage = object::borrow_value(this);
        any_table::borrow(&account_storage.resources, type_info::type_of<T>())
    }

    /// Add a resource to the account storage
    /// This function equates to `move_to<T>(signer, resource)` instruction in Move
    public fun add_resource<T: key>(this: &mut Object<AccountStorage>, resource: T){
        let type_info = type_info::type_of<T>();
        let account_storage = object::borrow_value_mut(this);
        assert!(!any_table::contains(&account_storage.resources, type_info), EResourceAlreadyExists);
        any_table::add(&mut account_storage.resources, type_info, resource);
    }


}