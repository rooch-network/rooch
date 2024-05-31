// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

/// TypeTable is a table use struct Type as Key, struct as Value

module moveos_std::type_table {
    use std::string::String;
    use moveos_std::type_info;
    use moveos_std::object::ObjectID;
    use moveos_std::object::{Self, Object};

    struct TablePlaceholder has key {
        _placeholder: bool,
    }

    struct TypeTable has store {
        handle: Object<TablePlaceholder>,
    }

    /// Create a new Table.
    public fun new(): TypeTable {
        let obj = object::new(TablePlaceholder{_placeholder: false});
        TypeTable {
            handle: obj,
        }
    }

    /// Note: We use Type name as key, the key will be serialized by bcs in the native function. 
    public fun key<V>(): String {
        type_info::type_name<V>()
    }

    /// Add a new entry of `V` to the table. Aborts if an entry for
    /// entry of `V` type already exists.
    public fun add<V: key + store>(table: &mut TypeTable, val: V) {
        object::add_field(&mut table.handle, key<V>(), val);
    }

    /// Acquire an immutable reference to the value which type is `V`.
    /// Aborts if there is no entry for `V`.
    public fun borrow<V: key + store>(table: &TypeTable): &V {
        object::borrow_field(&table.handle, key<V>())
    }

    /// Acquire a mutable reference to the value which type is `V`.
    /// Aborts if there is no entry for `V`.
    public fun borrow_mut<V: key + store>(table: &mut TypeTable): &mut V {
        object::borrow_mut_field(&mut table.handle, key<V>())
    }

    /// Remove from `table` and return the value which type is `V`.
    /// Aborts if there is no entry for `V`.
    public fun remove<V: key + store>(table: &mut TypeTable): V {
        object::remove_field(&mut table.handle, key<V>())
    }

    /// Returns true if `table` contains an entry for type `V`.
    public fun contains<V: key + store>(table: &TypeTable): bool {
        object::contains_field(&table.handle, key<V>())
    }

    /// Returns table handle of `table`.
    public fun handle(table: &TypeTable): ObjectID {
        object::id(&table.handle)
    }
  
    #[test_only]
    /// Testing only: allows to drop a table even if it is not empty.
    public fun drop_unchecked(table: TypeTable) {
        let TypeTable{handle} = table;
        let TablePlaceholder{_placeholder:_} = object::remove_unchecked(handle);
    }

    /// Destroy a table. The table must be empty to succeed.
    public fun destroy_empty(table: TypeTable) {
        let TypeTable{handle} = table;
        let TablePlaceholder{_placeholder:_} = object::remove(handle);
    }

    #[test_only]
    struct TestType has key,store {
        val: u64,
    }

    #[test]
    fun test_all() {
        let table = new();

        let t = TestType {
            val: 1,
        };

        assert!(!contains<TestType>(&table), 1);
        add<TestType>(&mut table, t);
        assert!(contains<TestType>(&table), 2);

        assert!(borrow<TestType>(&table).val == 1, 3);
        borrow_mut<TestType>(&mut table).val = 2;
        assert!(borrow<TestType>(&table).val == 2, 4);

        let TestType {val:_} = remove<TestType>(&mut table);
        assert!(!contains<TestType>(&table), 5);

        drop_unchecked(table);
    }

    #[test]
    #[expected_failure]
    fun test_add_key_exist_failure() {
        let table = new();

        let t = TestType {
            val: 1,
        };

        add<TestType>(&mut table, t);
        assert!(contains<TestType>(&table), 1);

        let t = TestType {
            val: 2,
        };
        add<TestType>(&mut table, t);

        drop_unchecked(table);
    }

    #[test]
    #[expected_failure]
    fun test_borrow_key_not_exist_failure() {
        let table = new();
        let _ = borrow<TestType>(&table).val;

        drop_unchecked(table);
    }

    #[test]
    #[expected_failure]
    fun test_borrow_mut_key_not_exist_failure() {
        let table = new();
        let t = borrow_mut<TestType>(&mut table);
        t.val = 1;

        drop_unchecked(table);
    }

    #[test]
    #[expected_failure]
    fun test_remove_key_not_exist_failure() {
        let table = new();
        let TestType { val: _} = remove<TestType>(&mut table);

        drop_unchecked(table);
    }

    #[test]
    #[expected_failure]
    fun test_destroy_non_empty_failure() {
        let table = new();
        let t = TestType {
            val: 1,
        };
        add<TestType>(&mut table, t);

        destroy_empty(table);
    }
}
