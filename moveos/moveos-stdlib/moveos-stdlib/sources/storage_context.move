/// AccountStorage is part of the StorageAbstraction
/// It is used to store the objects

module moveos_std::storage_context {
    use moveos_std::raw_table;
    use moveos_std::object::{Self, Object};
    use moveos_std::object_id::{Self, ObjectID};
    #[test_only]
    use moveos_std::test_helper;
    #[test_only]
    use moveos_std::tx_context::TxContext;

    friend moveos_std::account_storage;
    friend moveos_std::context;

    const GlobalObjectStorageHandle: address = @0x0;

    struct StorageContext has store {
        handle: ObjectID,
    }

    /// Create a new StorageContext with a given ObjectID.
    public(friend) fun new_with_id(handle: ObjectID): StorageContext {
        StorageContext {
            handle,
        }
    }

    /// The global object storage's table handle should be `0x0`
    public(friend) fun global_object_storage_handle(): ObjectID {
        object_id::address_to_object_id(GlobalObjectStorageHandle)
    }

    /// Borrow object from storage context with object_id
    public(friend) fun borrow<T: key>(self: &StorageContext, object_id: ObjectID): &Object<T> {
        raw_table::borrow<ObjectID, Object<T>>(&self.handle, object_id)
    }

    /// Borrow mut object from storage context with object_id
    public(friend) fun borrow_mut<T: key>(self: &mut StorageContext, object_id: ObjectID): &mut Object<T> {
        raw_table::borrow_mut<ObjectID, Object<T>>(&self.handle, object_id)
    }

    /// Remove object from storage context
    public(friend) fun remove<T: key>(self: &mut StorageContext, object_id: ObjectID): Object<T> {
        raw_table::remove<ObjectID, Object<T>>(&self.handle, object_id)
    }

    /// Add object to storage context
    public(friend) fun add<T: key>(self: &mut StorageContext, obj: Object<T>) {
        raw_table::add<ObjectID, Object<T>>(&self.handle, object::id(&obj), obj);
    }

    /// Determine whether the object exixts
    public(friend) fun contains(self: &StorageContext, object_id: ObjectID): bool {
        raw_table::contains<ObjectID>(&self.handle, object_id)
    }


    #[test_only]
    /// Testing only: allow to drop oject storage
    public fun drop_object_storage(self: StorageContext) {
        test_helper::destroy<StorageContext>(self);
    }

    #[test_only]
    /// There is only one instance: the global object storage.
    /// This `new` function is only used for testing
    public fun new(ctx: &mut TxContext): StorageContext {
        StorageContext {
            handle: raw_table::new_table_handle(ctx),
        }
    }

    #[test_only]    
    /// Destroy a ObjectStroage. The StorageContext must be empty to succeed.
    public fun destroy_empty(self: StorageContext) {
        let StorageContext { handle } = self;
        raw_table::destroy_empty(&handle)
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
