// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

module moveos_std::object_table {
    use moveos_std::object_id::ObjectID;
    use moveos_std::object_id;
    use moveos_std::context::{Self, Context};
    use moveos_std::object::{Self, Object};

    /// A Table for storing objects
    struct ObjectTable<phantom T> has key, store {
        handle: ObjectID,
    }

    /// Create a new Table.
    public fun new<T: key>(ctx: &mut Context): ObjectTable<T> {
        let handle = object_id::address_to_object_id(context::fresh_address(ctx));
        let obj = object::new_table_with_id(handle);
        object::transfer(obj, @moveos_std);

        ObjectTable {
            handle,
        }
    }

    /// Add a new Object to the table.
    public fun add<T>(table: &mut ObjectTable<T>, obj: Object<T>) {
        let object_id = object::id(&obj);
        object::add_field<ObjectID, Object<T>>(table.handle, object_id, obj)
    }

    /// Acquire an immutable reference to the Object<T> with `object_id`.
    /// Aborts if there is no entry for `object_id`.
    public fun borrow<T>(table: &ObjectTable<T>, object_id: ObjectID): &Object<T> {
        object::borrow_field<ObjectID, Object<T>>(table.handle, object_id)
    }
 
    /// Acquire a mutable reference to the Object<T> with `object_id`.
    /// Aborts if there is no entry for `object_id`.
    public fun borrow_mut<T>(table: &mut ObjectTable<T>, object_id: ObjectID): &mut Object<T> {
        object::borrow_mut_field<ObjectID, Object<T>>(table.handle, object_id)
    }

    /// Remove from `table` and return the Object<T>  with `object_id`.
    /// Aborts if there is no entry for `object_id`.
    public fun remove<T>(table: &mut ObjectTable<T>, object_id: ObjectID): Object<T> {
        object::remove_field<ObjectID, Object<T>>(table.handle, object_id)
    }

    /// Returns true if `table` contains an entry for `object_id`.
    public fun contains<T>(table: &ObjectTable<T>, object_id: ObjectID): bool {
        object::contains_field<ObjectID>(table.handle, object_id)
    }

    /// Destroy a table. Aborts if the table is not empty.
    public fun destroy_empty<T>(table: ObjectTable<T>) {
        let ObjectTable { handle } = table;
        object::destroy_empty_table(handle)
    }

    /// Returns the size of the table 
    public fun length<T>(table: &ObjectTable<T>): u64 {
        object::table_length(table.handle)
    }

    /// Returns true iff the table is empty (if `length` returns `0`)
    public fun is_empty<T>(table: &ObjectTable<T>): bool {
        object::table_length(table.handle) == 0
    }


    #[test_only]
    /// Testing only: allows to drop a table even if it is not empty.
    public fun drop_unchecked<T>(table: ObjectTable<T>) {
        let ObjectTable { handle } = table;
        object::drop_unchecked_table(handle)
    }


    /// Returns table handle of `table`.
    public fun handle<T>(table: &ObjectTable<T>): &ObjectID {
        &table.handle
    }

    #[test_only]
    struct TestStruct has key{
        v: u64,
    }

    #[test(sender = @0x42)]
    fun test_all(sender: signer) {
        let sender_addr = std::signer::address_of(&sender);
        let ctx = moveos_std::context::new_test_context(sender_addr);
        let t = new<TestStruct>(&mut ctx);
        
        let obj = context::new_object(&mut ctx, TestStruct{v: 0});
        let object_id = object::id(&obj);

        assert!(!contains(&t, object_id), 1000);
       
        add(&mut t, obj);
       
        assert!(contains(&t, object_id), 1001);
        {
            let obj = borrow_mut(&mut t, object_id);
            object::borrow_mut(obj).v = 2;
        };
        {
            let obj = borrow(&t, object_id);
            assert!(object::borrow(obj).v == 2, 1002);
        };
        
        let obj = remove(&mut t, object_id);
        assert!(!contains(&t, object_id), 1003);
        let TestStruct{v:_} = object::remove(obj);
        drop_unchecked(t);
        moveos_std::context::drop_test_context(ctx);
    }
}