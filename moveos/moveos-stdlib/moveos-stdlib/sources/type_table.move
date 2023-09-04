/// TypeTable is a table use struct Type as Key, struct as Value

module moveos_std::type_table {

    use std::ascii::{String};
    use moveos_std::raw_table;
    use moveos_std::tx_context::TxContext;
    use moveos_std::object_id::ObjectID;

    friend moveos_std::account_storage;

    struct TypeTable has store {
        handle: ObjectID,
    }

    /// Create a new Table.
    public fun new(ctx: &mut TxContext): TypeTable {
        TypeTable {
            handle: raw_table::new_table_handle(ctx),
        }
    }

    /// Create a new Table with a given handle.
    public(friend) fun new_with_id(handle: ObjectID): TypeTable{
        TypeTable {
            handle,
        }
    }

    /// Note: We use Type name as key, the key will be serialized by bcs in the native function. 
    fun key<V>(): String {
        let type_name = std::type_name::get<V>();
        let name_string = std::type_name::into_string(type_name);
        //std::debug::print(&name_string);
        //std::debug::print(&std::bcs::to_bytes(&name_string));
        name_string
    }

    /// Add a new entry of `V` to the table. Aborts if an entry for
    /// entry of `V` type already exists.
    public fun add<V: key>(table: &mut TypeTable, val: V) {
        raw_table::add<String, V>(&table.handle, key<V>(), val)
    }

    /// Acquire an immutable reference to the value which type is `V`.
    /// Aborts if there is no entry for `V`.
    public fun borrow<V: key>(table: &TypeTable): &V {
        raw_table::borrow<String, V>(&table.handle, key<V>())
    }

    /// Acquire a mutable reference to the value which type is `V`.
    /// Aborts if there is no entry for `V`.
    public fun borrow_mut<V: key>(table: &mut TypeTable): &mut V {
        raw_table::borrow_mut<String, V>(&table.handle, key<V>())
    }

    /// Remove from `table` and return the value which type is `V`.
    /// Aborts if there is no entry for `V`.
    public fun remove<V: key>(table: &mut TypeTable): V {
        raw_table::remove<String, V>(&table.handle, key<V>())
    }

    /// Returns true if `table` contains an entry for type `V`.
    public fun contains<V: key>(table: &TypeTable): bool {
        raw_table::contains<String>(&table.handle, key<V>())
    }

    /// Returns table handle of `table`.
    public fun handle<V: key>(table: &TypeTable): &ObjectID {
        &table.handle
    }
  
    #[test_only]
    /// Testing only: allows to drop a table even if it is not empty.
    public fun drop_unchecked(table: TypeTable) {
        let TypeTable{handle} = table;
        raw_table::drop_unchecked(&handle)
    }

    /// Destroy a table. The table must be empty to succeed.
    public fun destroy_empty(table: TypeTable) {
        let TypeTable{handle} = table;
        raw_table::destroy_empty(&handle)
    }

    #[test_only]
    struct TestType has key {
        val: u64,
    }

    #[test(account = @0x1)]
    fun test_all(account: signer) {
        let sender = std::signer::address_of(&account);
        let tx_context = moveos_std::tx_context::new_test_context(sender);
        let table = new(&mut tx_context);

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

    #[test(account = @0x1)]
    #[expected_failure]
    fun test_add_key_exist_failure(account: signer) {
        let sender = std::signer::address_of(&account);
        let tx_context = moveos_std::tx_context::new_test_context(sender);
        let table = new(&mut tx_context);

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

    #[test(account = @0x1)]
    #[expected_failure]
    fun test_borrow_key_not_exist_failure(account: signer) {
        let sender = std::signer::address_of(&account);
        let tx_context = moveos_std::tx_context::new_test_context(sender);
        let table = new(&mut tx_context);
        let _ = borrow<TestType>(&table).val;

        drop_unchecked(table);
    }

    #[test(account = @0x1)]
    #[expected_failure]
    fun test_borrow_mut_key_not_exist_failure(account: signer) {
        let sender = std::signer::address_of(&account);
        let tx_context = moveos_std::tx_context::new_test_context(sender);
        let table = new(&mut tx_context);
        let t = borrow_mut<TestType>(&mut table);
        t.val = 1;

        drop_unchecked(table);
    }

    #[test(account = @0x1)]
    #[expected_failure]
    fun test_remove_key_not_exist_failure(account: signer) {
        let sender = std::signer::address_of(&account);
        let tx_context = moveos_std::tx_context::new_test_context(sender);
        let table = new(&mut tx_context);
        let TestType { val: _} = remove<TestType>(&mut table);

        drop_unchecked(table);
    }

    #[test(account = @0x1)]
    #[expected_failure]
    fun test_destroy_non_empty_failure(account: signer) {
        let sender = std::signer::address_of(&account);
        let tx_context = moveos_std::tx_context::new_test_context(sender);
        let table = new(&mut tx_context);
        let t = TestType {
            val: 1,
        };
        add<TestType>(&mut table, t);

        destroy_empty(table);
    }
}
