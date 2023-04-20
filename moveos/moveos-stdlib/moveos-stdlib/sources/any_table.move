/// Type of tables store any type of value.

module moveos_std::any_table {

    /// Type of tables
    struct Table<phantom K: copy + drop> has store {
        handle: address,
    }

    /// Create a new Table.
    public fun new<K: copy + drop>(): Table<K> {
        Table {
            handle: new_table_handle<K>(),
        }
    }

    /// Add a new entry to the table. Aborts if an entry for this
    /// key already exists. The entry itself is not stored in the
    /// table, and cannot be discovered from it.
    public fun add<K: copy + drop, V>(table: &mut Table<K>, key: K, val: V) {
        add_box<K, V, Box<V>>(table, key, Box { val })
    }

    /// Acquire an immutable reference to the value which `key` maps to.
    /// Aborts if there is no entry for `key`.
    public fun borrow<K: copy + drop, V>(table: &Table<K>, key: K): &V {
        &borrow_box<K, V, Box<V>>(table, key).val
    }

    /// Acquire an immutable reference to the value which `key` maps to.
    /// Returns specified default value if there is no entry for `key`.
    public fun borrow_with_default<K: copy + drop, V>(table: &Table<K>, key: K, default: &V): &V {
        if (!contains(table, copy key)) {
            default
        } else {
            borrow(table, copy key)
        }
    }

    /// Acquire a mutable reference to the value which `key` maps to.
    /// Aborts if there is no entry for `key`.
    public fun borrow_mut<K: copy + drop, V>(table: &mut Table<K>, key: K): &mut V {
        &mut borrow_box_mut<K, V, Box<V>>(table, key).val
    }

    /// Acquire a mutable reference to the value which `key` maps to.
    /// Insert the pair (`key`, `default`) first if there is no entry for `key`.
    public fun borrow_mut_with_default<K: copy + drop, V: drop>(table: &mut Table<K>, key: K, default: V): &mut V {
        if (!contains(table, copy key)) {
            add(table, copy key, default)
        };
        borrow_mut(table, key)
    }

    /// Insert the pair (`key`, `value`) if there is no entry for `key`.
    /// update the value of the entry for `key` to `value` otherwise
    public fun upsert<K: copy + drop, V: drop>(table: &mut Table<K>, key: K, value: V) {
        if (!contains(table, copy key)) {
            add(table, copy key, value)
        } else {
            let ref = borrow_mut(table, key);
            *ref = value;
        };
    }

    /// Remove from `table` and return the value which `key` maps to.
    /// Aborts if there is no entry for `key`.
    public fun remove<K: copy + drop, V>(table: &mut Table<K>, key: K): V {
        let Box { val } = remove_box<K, V, Box<V>>(table, key);
        val
    }

    /// Returns true iff `table` contains an entry for `key`.
    public fun contains<K: copy + drop>(table: &Table<K>, key: K): bool {
        contains_box<K>(table, key)
    }

    #[test_only]
    /// Testing only: allows to drop a table even if it is not empty.
    public fun drop_unchecked<K: copy + drop>(table: Table<K>) {
        drop_unchecked_box<K>(table)
    }

    ///TODO should open the destroy function to public?
    public(friend) fun destroy<K: copy + drop>(table: Table<K>) {
        destroy_empty_box<K>(&table);
        drop_unchecked_box<K>(table)
    }

    #[test_only]
    struct TableHolder<phantom K: copy + drop> has key {
        t: Table<K>
    }

    #[test(account = @0x1)]
    fun test_upsert(account: signer) {
        let t = new<u64>();
        let key: u64 = 111;
        let error_code: u64 = 1;
        assert!(!contains(&t, key), error_code);
        upsert(&mut t, key, 12);
        assert!(*borrow(&t, key) == 12, error_code);
        upsert(&mut t, key, 23);
        assert!(*borrow(&t, key) == 23, error_code);

        move_to(&account, TableHolder { t });
    }

    #[test(account = @0x1)]
    fun test_borrow_with_default(account: signer) {
        let t = new<u64>();
        let key: u64 = 100;
        let error_code: u64 = 1;
        assert!(!contains(&t, key), error_code);
        assert!(*borrow_with_default(&t, key, &12) == 12, error_code);
        add(&mut t, key, 1);
        assert!(*borrow_with_default(&t, key, &12) == 1, error_code);

        move_to(&account, TableHolder{ t });
    }

    // ======================================================================================================
    // Internal API

    /// Wrapper for values. Required for making values appear as resources in the implementation.
    struct Box<V> has key, drop, store {
        val: V
        //TODO we should save V's type info here and check it when deserializing
    }

    // Primitives which take as an additional type parameter `Box<V>`, so the implementation
    // can use this to determine serialization layout.
    native fun new_table_handle<K>(): address;

    native fun add_box<K: copy + drop, V, B>(table: &mut Table<K>, key: K, val: Box<V>);

    native fun borrow_box<K: copy + drop, V, B>(table: &Table<K>, key: K): &Box<V>;

    native fun borrow_box_mut<K: copy + drop, V, B>(table: &mut Table<K>, key: K): &mut Box<V>;

    native fun contains_box<K: copy + drop>(table: &Table<K>, key: K): bool;

    native fun remove_box<K: copy + drop, V, B>(table: &mut Table<K>, key: K): Box<V>;

    native fun destroy_empty_box<K: copy + drop>(table: &Table<K>);

    native fun drop_unchecked_box<K: copy + drop>(table: Table<K>);
}
