/// AccountStorage is part of the StorageAbstraction
/// It is used to store the objects

module moveos_std::object_storage {
    use moveos_std::tx_context::{TxContext};
    use moveos_std::raw_table;
    use moveos_std::object::{Self, Object};
    use moveos_std::object_id::{Self, ObjectID};
    #[test_only]
    use moveos_std::test_helper;

    friend moveos_std::account_storage;
    friend moveos_std::storage_context;

    const GlobalObjectStorageHandle : address = @0x0;

    struct ObjectStorage has store {
        handle: ObjectID,
    }

    public fun new(ctx: &mut TxContext): ObjectStorage {
        ObjectStorage {
            handle: raw_table::new_table_handle(ctx),
        }
    }

    /// Create a new ObjectStorage with a given handle.
    public(friend) fun new_with_id(handle: ObjectID): ObjectStorage{
        ObjectStorage {
            handle,
        }
    }

    /// The global object storage's table handle should be 0x0
    public(friend) fun global_object_storage_handle() : ObjectID {
        object_id::address_to_object_id(GlobalObjectStorageHandle)
    }

    #[private_generics(T)]
    /// Borrow Object from object store with object_id
    public fun borrow<T: key>(this: &ObjectStorage, object_id: ObjectID): &Object<T>{
        raw_table::borrow<ObjectID, Object<T>>(&this.handle, object_id)
    }

    #[private_generics(T)]
    /// Borrow mut Object from object store with object_id
    public fun borrow_mut<T: key>(this: &mut ObjectStorage, object_id: ObjectID): &mut Object<T>{
        raw_table::borrow_mut<ObjectID, Object<T>>(&this.handle, object_id)
    }
    
    #[private_generics(T)]
    /// Remove object from object store
    public fun remove<T: key>(this: &mut ObjectStorage, object_id: ObjectID): Object<T>{
        raw_table::remove<ObjectID, Object<T>>(&this.handle, object_id)
    }
    
    #[private_generics(T)]
    /// Add object to object store
    public fun add<T: key>(this: &mut ObjectStorage, obj: Object<T>) {
        raw_table::add<ObjectID, Object<T>>(&this.handle, object::id(&obj), obj);
    } 

    public fun contains<T: key>(this: &ObjectStorage, object_id: ObjectID): bool{
        raw_table::contains<ObjectID, Object<T>>(&this.handle, object_id)
    }

    #[test_only]
    /// Testing only: allow to drop oject storage
    public fun drop_object_storage(this: ObjectStorage) {
        // raw_table::drop_unchecked<ObjectID>(this.handle);
        // let ObjectStorage { handle: _} = this;

        test_helper::destroy<ObjectStorage>(this);
    }
    #[test_only]
    struct TestObject has key{
        f: u8
    }
     #[test_only]
    struct TestObject2 has key{
        f: u8
    }
    #[test(sender=@0x42)]
    fun test_object_storage(sender: address) {
        let ctx = moveos_std::tx_context::new_test_context(sender);
        let os = new(&mut ctx);
        let object = object::new(&mut ctx, sender, TestObject{f: 1});
        let object_id = object::id(&object);
        add(&mut os, object);
        assert!(contains<TestObject>(&os, object_id), 1000);

        //FIXME https://github.com/rooch-network/rooch/issues/112 
        //assert!(!contains<TestObject2>(&os, object_id), 1001);
        
        let object_ref = borrow<TestObject>(&os, object_id);
        let test_obj_ref = object::borrow<TestObject>(object_ref);
        assert!(test_obj_ref.f == 1, 1002);
        
        let object = remove<TestObject>(&mut os, object_id);
        let (_id, _owner, test_object) = object::unpack(object);
        let TestObject{f} = test_object;
        assert!(f == 1, 1003);
        assert!(!contains<TestObject>(&os, object_id), 1004);

        drop_object_storage(os);
    }
}
