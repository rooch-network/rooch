// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

/// Type of large-scale storage tables.
/// source: https://github.com/move-language/move/blob/1b6b7513dcc1a5c866f178ca5c1e74beb2ce181e/language/extensions/move-table-extension/sources/Table.move#L1
///
/// It implements the Table type which supports individual table items to be represented by
/// separate global state items. The number of items and a unique handle are tracked on the table
/// struct itself, while the operations are implemented as native functions. No traversal is provided.

module moveos_std::table {
    use std::option::Option;
    use std::vector;

    use moveos_std::core_addresses;
    use moveos_std::object::ObjectID;
    use moveos_std::object::{Self, Object};

    struct TablePlaceholder has key {
        _placeholder: bool,
    }

    /// Type of tables
    struct Table<phantom K: copy + drop + store, phantom V> has store {
        handle: Object<TablePlaceholder>,
    }

    struct Iterator<K: copy + drop + store, phantom V> has store, drop {
        handle: ObjectID,
        cursor: Option<K>,
        limit: u64,
        keys: vector<address>,
    }

    /// Create a new Table.
    public fun new<K: copy + drop + store, V: store>(): Table<K, V> {
        let obj = object::new(TablePlaceholder{ _placeholder: false });
        Table {
            handle: obj,
        }
    }

    /// Create a new Table with object id.
    public fun new_with_object_id_by_system<K: copy + drop + store, V: store>(system: &signer, id: ObjectID): Table<K, V> {
        core_addresses::assert_system_reserved(system);
        let obj = object::new_with_object_id(id, TablePlaceholder{ _placeholder: false });
        Table {
            handle: obj,
        }
    }

    /// Add a new entry to the table. Aborts if an entry for this
    /// key already exists. The entry itself is not stored in the
    /// table, and cannot be discovered from it.
    public fun add<K: copy + drop + store, V:store>(table: &mut Table<K, V>, key: K, val: V) {
        object::add_field(&mut table.handle, key, val)
    }

    /// Acquire an immutable reference to the value which `key` maps to.
    /// Aborts if there is no entry for `key`.
    public fun borrow<K: copy + drop + store, V:store>(table: &Table<K, V>, key: K): &V {
        object::borrow_field(&table.handle, key)
    }

    /// Acquire an immutable reference to the value which `key` maps to.
    /// Returns specified default value if there is no entry for `key`.
    public fun borrow_with_default<K: copy + drop + store, V:store>(table: &Table<K, V>, key: K, default: &V): &V {
        object::borrow_field_with_default(&table.handle, key, default)
    }

    /// Acquire a mutable reference to the value which `key` maps to.
    /// Aborts if there is no entry for `key`.
    public fun borrow_mut<K: copy + drop + store, V:store>(table: &mut Table<K, V>, key: K): &mut V {
        object::borrow_mut_field(&mut table.handle, key)
    }

    /// Acquire a mutable reference to the value which `key` maps to.
    /// Insert the pair (`key`, `default`) first if there is no entry for `key`.
    public fun borrow_mut_with_default<K: copy + drop + store, V: store + drop>(table: &mut Table<K, V>, key: K, default: V): &mut V {
        object::borrow_mut_field_with_default(&mut table.handle, key, default)
    }

    /// Insert the pair (`key`, `value`) if there is no entry for `key`.
    /// update the value of the entry for `key` to `value` otherwise
    public fun upsert<K: copy + drop + store, V: store + drop>(table: &mut Table<K, V>, key: K, value: V) {
        object::upsert_field(&mut table.handle, key, value)
    }

    /// Remove from `table` and return the value which `key` maps to.
    /// Aborts if there is no entry for `key`.
    public fun remove<K: copy + drop + store, V:store>(table: &mut Table<K, V>, key: K): V {
        object::remove_field(&mut table.handle, key)
    }

    /// Returns true if `table` contains an entry for `key`.
    public fun contains<K: copy + drop + store, V:store>(table: &Table<K, V>, key: K): bool {
        object::contains_field(&table.handle, key)
    }

    /// Returns a vector of keys in the table from the given cursor position, up to the specified limit.
    /// cursor: Optional address to start listing from. If None, starts from the beginning.
    /// limit: Maximum number of keys to return.
    public fun list_field_keys<K: copy + drop + store, V:store>(table: &Table<K, V>, cursor: Option<K>, limit: u64): Iterator<K, V> {
        let keys = object::list_field_keys(&table.handle, cursor, limit);
        Iterator {
            handle: object::id(&table.handle),
            cursor,
            limit,
            keys,
        }
    }

    /// Returns the number of keys in the table.
    public fun field_keys_len<K: copy + drop + store, V:store>(iterator: &Iterator<K, V>): u64 {
        vector::length(&iterator.keys)
    }

    /// Returns a immutable reference to the next key-value pair in the table, starting from the given iterator.
    public fun next<K: copy + drop + store, V:store>(iterator: &mut Iterator<K, V>): (&K, &V) {
        let key = vector::pop_back(&mut iterator.keys);
        object::borrow_field_with_key_internal<K, V>(iterator.handle, key)
    }

    /// Returns a mutable reference to the next key-value pair in the table, starting from the given iterator.
    public fun next_mut<K: copy + drop + store, V:store>(iterator: &mut Iterator<K, V>): (&K, &mut V) {
        let key = vector::pop_back(&mut iterator.keys);
        object::borrow_mut_field_with_key_internal<K, V>(iterator.handle, key)
    }

    /// Destroy a table. Aborts if the table is not empty.
    public fun destroy_empty<K: copy + drop + store, V:store>(table: Table<K, V>) {
        let Table { handle } = table;
        let TablePlaceholder{_placeholder:_} = object::remove(handle);
    }

    /// Returns the size of the table, the number of key-value pairs
    public fun length<K: copy + drop + store, V:store>(table: &Table<K, V>): u64 {
        object::field_size(&table.handle)
    }

    /// Returns true iff the table is empty (if `length` returns `0`)
    public fun is_empty<K: copy + drop + store, V:store>(table: &Table<K, V>): bool {
        object::field_size(&table.handle) == 0
    }

    /// Drop a possibly non-empty table.
    /// Usable only if the value type `V` has the `drop` ability
    public fun drop<K: copy + drop + store, V: drop>(table: Table<K, V>) {
        let Table { handle } = table;
        let TablePlaceholder{_placeholder:_} = object::remove_unchecked(handle);
    }


    #[test_only]
    /// Testing only: allows to drop a table even if it is not empty.
    public fun drop_unchecked<K: copy + drop + store, V:store>(table: Table<K, V>) {
        let Table { handle } = table;
        let TablePlaceholder{_placeholder:_} = object::remove_unchecked(handle);
    }

    /// Returns table handle of `table`.
    public fun handle<K: copy + drop + store, V:store>(table: &Table<K, V>): ObjectID {
        object::id(&table.handle)
    }

    #[test_only]
    struct TableHolder<phantom K: copy + drop + store, phantom V: drop> has key {
        t: Table<K, V>
    }

    #[test]
    fun test_upsert() {
        let t = new<u64, u8>();
        let key: u64 = 111;
        let error_code: u64 = 1;
        assert!(!contains(&t, key), error_code);
        upsert(&mut t, key, 12);
        assert!(*borrow(&t, key) == 12, error_code);
        upsert(&mut t, key, 23);
        assert!(*borrow(&t, key) == 23, error_code);

        drop_unchecked(t);
    }

    #[test]
    fun test_borrow_with_default() {
        let t = new<u64, u8>();
        let key: u64 = 100;
        let error_code: u64 = 1;
        assert!(!contains(&t, key), error_code);
        assert!(*borrow_with_default(&t, key, &12) == 12, error_code);
        add(&mut t, key, 1);
        assert!(*borrow_with_default(&t, key, &12) == 1, error_code);

        drop_unchecked(t);
    }

    #[test]
    fun test_borrow_mut_with_default() {
        let t = new<u64, u8>();
        let key: u64 = 100;
        {
            let value = borrow_mut_with_default(&mut t, key, 0);
            assert!(*value == 0, 1000);
        };
        assert!(contains(&t, key), 1001);
        assert!(*borrow(&t, key) == 0, 1002);
        {
            let value = borrow_mut_with_default(&mut t, key, 0);
            *value = *value + 1;
        };
        assert!(*borrow(&t, key) == 1, 1003);
        drop_unchecked(t);
    }

    #[test]
    fun test_all() {
        let t = new<u64, u8>();
        let key: u64 = 100;
        let error_code: u64 = 1;
        assert!(!contains(&t, key), error_code);
        add(&mut t, key, 12);
        let val = borrow_mut(&mut t, key);
        *val = 23;
        assert!(*borrow(&t, key) == 23, error_code);
        remove(&mut t, key);
        assert!(!contains(&t, key), error_code);
        drop_unchecked(t);
    }

    #[test]
    #[expected_failure]
    fun test_add_key_exist_failure() {
        let t = new<u64, u8>();
        let key: u64 = 100;
        add(&mut t, key, 1);
        add(&mut t, key, 2);

        drop_unchecked(t);
    }

    #[test]
    #[expected_failure]
    fun test_borrow_key_not_exist_failure() {
        let t = new<u64, u8>();
        let key: u64 = 100;
        let _ = borrow(&mut t, key);

        drop_unchecked(t);
    }

    #[test]
    #[expected_failure]
    fun test_borrow_mut_key_not_exist_failure() {
        let t = new<u64, u8>();
        let key: u64 = 100;
        let _ = borrow_mut(&mut t, key);

        drop_unchecked(t);
    }

    #[test]
    #[expected_failure]
    fun test_remove_key_not_exist_failure() {
        let t = new<u64, u8>();
        let key: u64 = 100;
        remove(&mut t, key);

        drop_unchecked(t);
    }

    #[test]
    fun test_nested_table() {
        let t1 = new<u64, Table<u8, u32>>();
        let t2 = new<u8, u32>();

        let t1_key = 2u64;
        let t2_key = 1u8;
        add(&mut t2, t2_key, 32u32);
        add(&mut t1, t1_key, t2);

        let borrowed_t2 = borrow(&t1, t1_key);
        let value = borrow(borrowed_t2, t2_key);
        assert!(*value == 32u32, 1);


        let borrowed_mut_t2 = borrow_mut(&mut t1, t1_key);
        remove(borrowed_mut_t2, t2_key);

        drop_unchecked(t1);
    }

    #[test]
    #[expected_failure]
    fun test_destroy_nonempty_table() {
        let t = new<u64, u8>();
        let key: u64 = 100;
        add(&mut t, key, 1);

        destroy_empty(t);
    }

    #[test]
    fun test_add_and_remove_u64_key(){
        let t = new<u64, u8>();
        let key: u64 = 100;
        let error_code: u64 = 1;
        add(&mut t, key, 1);
        assert!(contains(&t, key), error_code);
        remove(&mut t, key);
        assert!(!contains(&t, key), error_code);
        destroy_empty(t);
    }

    #[test]
    fun test_add_and_remove_string_key(){
        let t = new<std::string::String, vector<u8>>();
        let key = std::string::utf8(b"test1");
        let error_code: u64 = 1;
        add(&mut t, key, b"value");
        assert!(contains(&t, key), error_code);
        remove(&mut t, key);
        assert!(!contains(&t, key), error_code);
        destroy_empty(t);
    }

}
