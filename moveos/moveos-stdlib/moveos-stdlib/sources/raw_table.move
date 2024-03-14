// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

/// Raw Key Value table. This is the basic of storage abstraction.
/// This type table doesn't care about the key and value types. We leave the data type checking to the Native implementation.
/// This type table is for internal global storage, so all functions are friend.

module moveos_std::raw_table {
    use moveos_std::object_id::ObjectID;

    friend moveos_std::object;

    /// The key already exists in the table
    const ErrorAlreadyExists: u64 = 1;
    /// Can not found the key in the table
    const ErrorNotFound: u64 = 2;
    /// Duplicate operation on the table
    const ErrorDuplicateOperation: u64 = 3;
    /// The table is not empty
    const ErrorNotEmpty: u64 = 4;
    /// The table already exists
    const ErrorTableAlreadyExists: u64 = 5;


    /// Add a new entry to the table. Aborts if an entry for this
    /// key already exists. The entry itself is not stored in the
    /// table, and cannot be discovered from it.
    public(friend) fun add<K: copy + drop, V>(table_handle: ObjectID, key: K, val: V) {
        add_box<K, V, Box<V>>(table_handle, key, Box {val} );
    }

    /// Acquire an immutable reference to the value which `key` maps to.
    /// Aborts if there is no entry for `key`.
    public(friend) fun borrow<K: copy + drop, V>(table_handle: ObjectID, key: K): &V {
        &borrow_box<K, V, Box<V>>(table_handle, key).val
    }

    /// Acquire a mutable reference to the value which `key` maps to.
    /// Aborts if there is no entry for `key`.
    public(friend) fun borrow_mut<K: copy + drop, V>(table_handle: ObjectID, key: K): &mut V {
        &mut borrow_box_mut<K, V, Box<V>>(table_handle, key).val
    }

    /// Remove from `table` and return the value which `key` maps to.
    /// Aborts if there is no entry for `key`.
    public(friend) fun remove<K: copy + drop, V>(table_handle: ObjectID, key: K): V {
        let Box { val } = remove_box<K, V, Box<V>>(table_handle, key);
        val
    }

    /// Returns true if `table` contains an entry for `key`.
    public(friend) fun contains<K: copy + drop>(table_handle: ObjectID, key: K): bool {
        contains_box<K>(table_handle, key)
    }


    // ======================================================================================================
    // Internal API
    
    /// Wrapper for values. Required for making values appear as resources in the implementation.
    /// Because the GlobalValue in MoveVM must be a resource.
    struct Box<V> has key, drop, store {
        val: V
    }

    native fun add_box<K: copy + drop, V, B>(table_handle: ObjectID, key: K, val: Box<V>);

    native fun borrow_box<K: copy + drop, V, B>(table_handle: ObjectID, key: K): &Box<V>;

    native fun borrow_box_mut<K: copy + drop, V, B>(table_handle: ObjectID, key: K): &mut Box<V>;

    native fun contains_box<K: copy + drop>(table_handle: ObjectID, key: K): bool;

    native fun remove_box<K: copy + drop, V, B>(table_handle: ObjectID, key: K): Box<V>;

}
