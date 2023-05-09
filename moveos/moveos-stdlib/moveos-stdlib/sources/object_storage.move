/// AccountStorage is part of the StorageAbstraction
/// It is used to store the objects

module moveos_std::object_storage {
    use moveos_std::tx_context::{TxContext};
    use moveos_std::raw_table;
    use moveos_std::object::{Self, Object, ObjectID};

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
    
    #[private_generic(T)]
    /// Add object to object store
    public fun add<T: key>(this: &mut ObjectStorage, obj: Object<T>) {
        raw_table::add<ObjectID, Object<T>>(*&this.handle, object::id(&obj), obj);
    }

    public fun contains<T: key>(this: &mut ObjectStorage, object_id: ObjectID): bool{
        raw_table::contains<ObjectID, T>(*&this.handle, object_id)
    }

    #[test_only]
    /// Testing only: allow to drop oject storage
    public fun drop_object_storage(this: &mut ObjectStorage) {
        raw_table::drop_unchecked(this.handle);
        _ = this;
    }

    // native fun drop_object_storage(table_handle: address);
 
}
