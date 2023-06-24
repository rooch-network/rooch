/// Raw Key Value table. This is the basic of storage abstraction.
/// This type table doesn't care about the key and value types. We leave the data type checking to the Native implementation.
/// This type table if for design internal global storage, so all functions are friend.

module moveos_std::raw_table {
    use moveos_std::tx_context::{Self, TxContext};
    use moveos_std::object_id::ObjectID;
    
    friend moveos_std::table;
    friend moveos_std::type_table;
    friend moveos_std::object_storage;
    friend moveos_std::account_storage;

    struct TableInfo has key {
        // Table SMT root
        state_root: address,
    }
    
    /// Add a new entry to the table. Aborts if an entry for this
    /// key already exists. The entry itself is not stored in the
    /// table, and cannot be discovered from it.
    public(friend) fun add<K: copy + drop, V>(table_handle: &ObjectID, key: K, val: V) {
        add_box<K, V, Box<V>>(table_handle, key, Box {val} )
    }

    /// Acquire an immutable reference to the value which `key` maps to.
    /// Aborts if there is no entry for `key`.
    public(friend) fun borrow<K: copy + drop, V>(table_handle: &ObjectID, key: K): &V {
        &borrow_box<K, V, Box<V>>(table_handle, key).val
    }

    /// Acquire an immutable reference to the value which `key` maps to.
    /// Returns specified default value if there is no entry for `key`.
    public(friend) fun borrow_with_default<K: copy + drop, V>(table_handle: &ObjectID, key: K, default: &V): &V {
        if (!contains<K>(table_handle, copy key)) {
            default
        } else {
            borrow(table_handle, copy key)
        }
    }

    /// Acquire a mutable reference to the value which `key` maps to.
    /// Aborts if there is no entry for `key`.
    public(friend) fun borrow_mut<K: copy + drop, V>(table_handle: &ObjectID, key: K): &mut V {
        &mut borrow_box_mut<K, V, Box<V>>(table_handle, key).val
    }

    /// Acquire a mutable reference to the value which `key` maps to.
    /// Insert the pair (`key`, `default`) first if there is no entry for `key`.
    public(friend) fun borrow_mut_with_default<K: copy + drop, V: drop>(table_handle: &ObjectID, key: K, default: V): &mut V {
        if (!contains<K>(table_handle, copy key)) {
            add(table_handle, copy key, default)
        };
        borrow_mut(table_handle, key)
    }

    /// Insert the pair (`key`, `value`) if there is no entry for `key`.
    /// update the value of the entry for `key` to `value` otherwise
    public(friend) fun upsert<K: copy + drop, V: drop>(table_handle: &ObjectID, key: K, value: V) {
        if (!contains<K>(table_handle, copy key)) {
            add(table_handle, copy key, value)
        } else {
            let ref = borrow_mut(table_handle, key);
            *ref = value;
        };
    }

    /// Remove from `table` and return the value which `key` maps to.
    /// Aborts if there is no entry for `key`.
    public(friend) fun remove<K: copy + drop, V>(table_handle: &ObjectID, key: K): V {
        let Box { val } = remove_box<K, V, Box<V>>(table_handle, key);
        val
    }

    /// Returns true if `table` contains an entry for `key`.
    public(friend) fun contains<K: copy + drop>(table_handle: &ObjectID, key: K): bool {
        contains_box<K>(table_handle, key)
    }

    #[test_only]
    /// Testing only: allows to drop a table even if it is not empty.
    public(friend) fun drop_unchecked(table_handle: &ObjectID) {
        drop_unchecked_box(table_handle)
    }
    
    /// Destroy a table. The table must be empty to succeed.
    public(friend) fun destroy_empty(table_handle: &ObjectID) {
        destroy_empty_box(table_handle)
    }

    // ======================================================================================================
    // Internal API
    
    /// Wrapper for values. Required for making values appear as resources in the implementation.
    /// Because the GlobalValue in MoveVM must be a resource.
    struct Box<V> has key, drop, store {
        val: V
    }

    public(friend) fun new_table_handle(ctx: &mut TxContext): ObjectID {
        tx_context::fresh_object_id(ctx)
    }

    native fun add_box<K: copy + drop, V, B>(table_handle: &ObjectID, key: K, val: Box<V>);

    native fun borrow_box<K: copy + drop, V, B>(table_handle: &ObjectID, key: K): &Box<V>;

    native fun borrow_box_mut<K: copy + drop, V, B>(table_handle: &ObjectID, key: K): &mut Box<V>;

    native fun contains_box<K: copy + drop>(table_handle: &ObjectID, key: K): bool;

    native fun remove_box<K: copy + drop, V, B>(table_handle: &ObjectID, key: K): Box<V>;

    native fun destroy_empty_box(table_handle: &ObjectID);

    native fun drop_unchecked_box(table_handle: &ObjectID);
}
