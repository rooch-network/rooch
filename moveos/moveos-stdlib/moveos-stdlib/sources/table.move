/// Type of large-scale storage tables.
/// source: https://github.com/move-language/move/blob/1b6b7513dcc1a5c866f178ca5c1e74beb2ce181e/language/extensions/move-table-extension/sources/Table.move#L1
///
/// It implements the Table type which supports individual table items to be represented by
/// separate global state items. The number of items and a unique handle are tracked on the table
/// struct itself, while the operations are implemented as native functions. No traversal is provided.

module moveos_std::table {
    use moveos_std::raw_table;
    use moveos_std::tx_context::TxContext;

    /// Type of tables
    struct Table<phantom K: copy + drop, phantom V> has store {
        handle: address,
    }

    /// Create a new Table.
    public fun new<K: copy + drop, V: store>(ctx: &mut TxContext): Table<K, V> {
        Table {
            handle: raw_table::new_table_handle(ctx),
        }
    }

    /// Add a new entry to the table. Aborts if an entry for this
    /// key already exists. The entry itself is not stored in the
    /// table, and cannot be discovered from it.
    public fun add<K: copy + drop, V>(table: &mut Table<K, V>, key: K, val: V) {
        raw_table::add<K, V>(*&table.handle, key, val)
    }

    /// Acquire an immutable reference to the value which `key` maps to.
    /// Aborts if there is no entry for `key`.
    public fun borrow<K: copy + drop, V>(table: &Table<K, V>, key: K): &V {
        raw_table::borrow<K, V>(*&table.handle, key)
    }

    /// Acquire an immutable reference to the value which `key` maps to.
    /// Returns specified default value if there is no entry for `key`.
    public fun borrow_with_default<K: copy + drop, V>(table: &Table<K, V>, key: K, default: &V): &V {
        raw_table::borrow_with_default<K, V>(*&table.handle, key, default)
    }

    /// Acquire a mutable reference to the value which `key` maps to.
    /// Aborts if there is no entry for `key`.
    public fun borrow_mut<K: copy + drop, V>(table: &mut Table<K, V>, key: K): &mut V {
        raw_table::borrow_mut<K, V>(*&table.handle, key)
    }

    /// Acquire a mutable reference to the value which `key` maps to.
    /// Insert the pair (`key`, `default`) first if there is no entry for `key`.
    public fun borrow_mut_with_default<K: copy + drop, V: drop>(table: &mut Table<K, V>, key: K, default: V): &mut V {
        raw_table::borrow_mut_with_default<K, V>(*&table.handle, key, default)
    }

    /// Insert the pair (`key`, `value`) if there is no entry for `key`.
    /// update the value of the entry for `key` to `value` otherwise
    public fun upsert<K: copy + drop, V: drop>(table: &mut Table<K, V>, key: K, value: V) {
        raw_table::upsert<K, V>(*&table.handle, key, value)
    }

    /// Remove from `table` and return the value which `key` maps to.
    /// Aborts if there is no entry for `key`.
    public fun remove<K: copy + drop, V>(table: &mut Table<K, V>, key: K): V {
        raw_table::remove<K, V>(*&table.handle, key)
    }

    /// Returns true if `table` contains an entry for `key`.
    public fun contains<K: copy + drop, V>(table: &Table<K, V>, key: K): bool {
        raw_table::contains<K, V>(*&table.handle, key)
    }

    #[test_only]
    /// Testing only: allows to drop a table even if it is not empty.
    public fun drop_unchecked<K: copy + drop, V>(table: Table<K, V>) {
        let Table { handle } = table;
        raw_table::drop_unchecked(handle)
    }

    ///TODO should open the destroy function to public?
    public(friend) fun destroy<K: copy + drop, V>(table: Table<K, V>) {
        let Table { handle } = table;
        raw_table::destroy(handle)
    }

    #[test_only]
    struct TableHolder<phantom K: copy + drop, phantom V: drop> has key {
        t: Table<K, V>
    }

    #[test(account = @0x1)]
    fun test_upsert(account: signer) {
        let t = new<u64, u8>();
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
        let t = new<u64, u8>();
        let key: u64 = 100;
        let error_code: u64 = 1;
        assert!(!contains(&t, key), error_code);
        assert!(*borrow_with_default(&t, key, &12) == 12, error_code);
        add(&mut t, key, 1);
        assert!(*borrow_with_default(&t, key, &12) == 1, error_code);

        move_to(&account, TableHolder{ t });
    }
}
