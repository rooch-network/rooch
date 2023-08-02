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

    const GlobalObjectStorageHandle: address = @0x0;

    struct ObjectStorage has store {
        handle: ObjectID,
    }

    public fun new(ctx: &mut TxContext): ObjectStorage {
        ObjectStorage {
            handle: raw_table::new_table_handle(ctx),
        }
    }

    /// Create a new ObjectStorage with a given handle.
    public(friend) fun new_with_id(handle: ObjectID): ObjectStorage {
        ObjectStorage {
            handle,
        }
    }

    /// The global object storage's table handle should be 0x0
    public(friend) fun global_object_storage_handle(): ObjectID {
        object_id::address_to_object_id(GlobalObjectStorageHandle)
    }

    #[private_generics(T)]
    /// Borrow Object from object store with object_id
    public fun borrow<T: key>(self: &ObjectStorage, object_id: ObjectID): &Object<T> {
        raw_table::borrow<ObjectID, Object<T>>(&self.handle, object_id)
    }

    #[private_generics(T)]
    /// Borrow mut Object from object store with object_id
    public fun borrow_mut<T: key>(self: &mut ObjectStorage, object_id: ObjectID): &mut Object<T> {
        raw_table::borrow_mut<ObjectID, Object<T>>(&self.handle, object_id)
    }

    #[private_generics(T)]
    /// Remove object from object store
    public fun remove<T: key>(self: &mut ObjectStorage, object_id: ObjectID): Object<T> {
        raw_table::remove<ObjectID, Object<T>>(&self.handle, object_id)
    }

    #[private_generics(T)]
    /// Add object to object store
    public fun add<T: key>(self: &mut ObjectStorage, obj: Object<T>) {
        raw_table::add<ObjectID, Object<T>>(&self.handle, object::id(&obj), obj);
    }

    public fun contains(self: &ObjectStorage, object_id: ObjectID): bool {
        raw_table::contains<ObjectID>(&self.handle, object_id)
    }

    /// Destroy a ObjectStroage. The ObjectStorage must be empty to succeed.
    public fun destroy_empty(self: ObjectStorage) {
        let ObjectStorage { handle } = self;
        raw_table::destroy_empty(&handle)
    }

    #[test_only]
    /// Testing only: allow to drop oject storage
    public fun drop_object_storage(self: ObjectStorage) {
        test_helper::destroy<ObjectStorage>(self);
    }

    #[test_only]
    struct TestObject has key {
        f: u8
    }

    #[test_only]
    struct TestObject2 has key {
        f: u8
    }

    #[test(sender = @0x42)]
    fun test_object_storage(sender: address) {
        let ctx = moveos_std::tx_context::new_test_context(sender);
        let os = new(&mut ctx);
        let object = object::new(&mut ctx, sender, TestObject { f: 1 });
        let object_id = object::id(&object);
        add(&mut os, object);
        assert!(contains(&os, object_id), 1000);

        let object2 = object::new(&mut ctx, sender, TestObject2 { f: 1 });
        let object_id2 = object::id(&object2);
        // The object_id2 is not in the object storage
        assert!(!contains(&os, object_id2), 1001);

        let object_ref = borrow<TestObject>(&os, object_id);
        let test_obj_ref = object::borrow<TestObject>(object_ref);
        assert!(test_obj_ref.f == 1, 1002);

        let object = remove<TestObject>(&mut os, object_id);
        let (_id, _owner, test_object) = object::unpack(object);
        let TestObject { f } = test_object;
        assert!(f == 1, 1003);
        assert!(!contains(&os, object_id), 1004);

        drop_object_storage(os);
        let (_id, _owner, test_object2) = object::unpack(object2);
        let TestObject2 { f: _f } = test_object2;
    }

    #[test(sender = @0x42)]
    #[expected_failure]
    fun test_borrow_not_exist_failure(sender: signer) {
        let sender_addr = std::signer::address_of(&sender);
        let ctx = moveos_std::tx_context::new_test_context(sender_addr);
        let os = new(&mut ctx);
        let object = object::new(&mut ctx, sender_addr, TestObject { f: 1 });
        let object_id = object::id(&object);

        let _obj_ref = borrow<TestObject>(&os, object_id);
        drop_object_storage(os);
        let (_id, _owner, test_object) = object::unpack(object);
        let TestObject { f : _f } = test_object;
    }

    #[test(sender = @0x42)]
    #[expected_failure]
    fun test_double_remove_failure(sender: signer) {
        let sender_addr = std::signer::address_of(&sender);
        let ctx = moveos_std::tx_context::new_test_context(sender_addr);
        let os = new(&mut ctx);
        let object = object::new(&mut ctx, sender_addr, TestObject { f: 1 });
        let object_id = object::id(&object);
        add<TestObject>(&mut os, object);
        let obj_rem1 = remove<TestObject>(&mut os, object_id);
        let obj_rem2 = remove<TestObject>(&mut os, object_id);
        drop_object_storage(os);
        let (_id, _owner, test_object1) = object::unpack(obj_rem1);
        let TestObject { f : _f } = test_object1;
        let (_id, _owner, test_object2) = object::unpack(obj_rem2);
        let TestObject { f : _f } = test_object2;
    }

    #[test(sender = @0x42)]
    #[expected_failure]
    fun test_remove_not_exist_failure(sender: signer) {
        let sender_addr = std::signer::address_of(&sender);
        let ctx = moveos_std::tx_context::new_test_context(sender_addr);
        let os = new(&mut ctx);
        let object = object::new(&mut ctx, sender_addr, TestObject { f: 1 });
        let object_id = object::id(&object);

        let obj_rem = remove<TestObject>(&mut os, object_id);
        drop_object_storage(os);
        let (_id, _owner, test_object) = object::unpack(object);
        let TestObject { f : _f } = test_object;
        let (_id, _owner, test_object_rem) = object::unpack(obj_rem);
        let TestObject { f : _f } = test_object_rem;
    }
}
