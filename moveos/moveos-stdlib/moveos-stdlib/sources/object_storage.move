/// AccountStorage is part of the StorageAbstraction
/// It is used to store the objects

module moveos_std::object_storage {
    use moveos_std::tx_context::{TxContext};
    use moveos_std::raw_table;
    use moveos_std::object::{Self, Object, ObjectID};
    use std::option;

    friend moveos_std::account_storage;

    struct ObjectStorage has store {
        handle: address,
    }

    public fun new(ctx: &mut TxContext): ObjectStorage {
        ObjectStorage {
            handle: raw_table::new_table_handle(ctx),
        }
    }

    #[private_generic(T)]
    public fun borrow<T: key>(this: &ObjectStorage, object_id: ObjectID): &Object<T>{
        raw_table::borrow<ObjectID, Object<T>>(*&this.handle, object_id)
    }

    #[private_generic(T)]
    public fun borrow_mut<T: key>(this: &mut ObjectStorage, object_id: ObjectID): &mut Object<T>{
        raw_table::borrow_mut<ObjectID, Object<T>>(*&this.handle, object_id)
    }
    
    #[private_generic(T)]
    /// Remove object from object store, only the owner can move the object
    public fun remove<T: key>(this: &mut ObjectStorage, object_id: ObjectID): Object<T>{
        raw_table::remove<ObjectID, Object<T>>(*&this.handle, object_id)
    }

    const EObjectOwnerIsEmpty: u64 = 101;
    #[private_generic(T)]
    /// Add object to object store
    public(friend) fun add<T: key>(this: &mut ObjectStorage, obj: Object<T>) {
        assert!(option::is_some(&object::owner(&obj)), EObjectOwnerIsEmpty);
        raw_table::add<ObjectID, Object<T>>(*&this.handle, object::id(&obj), obj);
    }

    #[private_generic(T)]
    public fun transfer<T: key>(this: &mut ObjectStorage, obj: Object<T>, recipient: address) {
        object::set_address_owner(&mut obj, recipient);
        add(this, obj);
    }

    #[private_generic(T)]
    public fun freeze_object<T: key>(this: &mut ObjectStorage, obj: Object<T>) {
        add(this, obj);
    }

    #[private_generic(T)]
    public fun share_object<T: key>(this: &mut ObjectStorage, obj: Object<T>) {
        add(this, obj);
    }

    public fun contains<T: key>(this: &mut ObjectStorage, object_id: ObjectID): bool{
        raw_table::contains<ObjectID, T>(*&this.handle, object_id)
    }
 
}
