// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

/// Type of large-scale storage tables.
/// source: https://github.com/move-language/move/blob/1b6b7513dcc1a5c866f178ca5c1e74beb2ce181e/language/extensions/move-table-extension/sources/Table.move#L1
///
/// It implements the Table type which supports individual table items to be represented by
/// separate global state items. The number of items and a unique handle are tracked on the table
/// struct itself, while the operations are implemented as native functions. No traversal is provided.

module moveos_std::table {
    use moveos_std::raw_table;
    use moveos_std::context::{Self, Context};
    use moveos_std::object::ObjectID;

    friend moveos_std::account_storage;

    /// Type of tables
    struct Table<phantom K: copy + drop, phantom V> has store {
        handle: ObjectID,
    }

    /// Create a new Table.
    public fun new<K: copy + drop, V: store>(ctx: &mut Context): Table<K, V> {
        let tx_ctx = context::tx_context_mut(ctx);
        Table {
            handle: raw_table::new_table_handle(tx_ctx),
        }
    }

    /// Create a table with a given handle.
    public(friend) fun new_with_id<K: copy + drop, V: store>(handle: ObjectID): Table<K, V>{
        Table {
            handle,
        }
    }

    /// Add a new entry to the table. Aborts if an entry for this
    /// key already exists. The entry itself is not stored in the
    /// table, and cannot be discovered from it.
    public fun add<K: copy + drop, V>(table: &mut Table<K, V>, key: K, val: V) {
        raw_table::add<K, V>(&table.handle, key, val)
    }

    /// Acquire an immutable reference to the value which `key` maps to.
    /// Aborts if there is no entry for `key`.
    public fun borrow<K: copy + drop, V>(table: &Table<K, V>, key: K): &V {
        raw_table::borrow<K, V>(&table.handle, key)
    }

    /// Acquire an immutable reference to the value which `key` maps to.
    /// Returns specified default value if there is no entry for `key`.
    public fun borrow_with_default<K: copy + drop, V>(table: &Table<K, V>, key: K, default: &V): &V {
        raw_table::borrow_with_default<K, V>(&table.handle, key, default)
    }

    /// Acquire a mutable reference to the value which `key` maps to.
    /// Aborts if there is no entry for `key`.
    public fun borrow_mut<K: copy + drop, V>(table: &mut Table<K, V>, key: K): &mut V {
        raw_table::borrow_mut<K, V>(&table.handle, key)
    }

    /// Acquire a mutable reference to the value which `key` maps to.
    /// Insert the pair (`key`, `default`) first if there is no entry for `key`.
    public fun borrow_mut_with_default<K: copy + drop, V: drop>(table: &mut Table<K, V>, key: K, default: V): &mut V {
        raw_table::borrow_mut_with_default<K, V>(&table.handle, key, default)
    }

    /// Insert the pair (`key`, `value`) if there is no entry for `key`.
    /// update the value of the entry for `key` to `value` otherwise
    public fun upsert<K: copy + drop, V: drop>(table: &mut Table<K, V>, key: K, value: V) {
        raw_table::upsert<K, V>(&table.handle, key, value)
    }

    /// Remove from `table` and return the value which `key` maps to.
    /// Aborts if there is no entry for `key`.
    public fun remove<K: copy + drop, V>(table: &mut Table<K, V>, key: K): V {
        raw_table::remove<K, V>(&table.handle, key)
    }

    /// Returns true if `table` contains an entry for `key`.
    public fun contains<K: copy + drop, V>(table: &Table<K, V>, key: K): bool {
        raw_table::contains<K>(&table.handle, key)
    }

    /// Destroy a table. Aborts if the table is not empty.
    public fun destroy_empty<K: copy + drop, V>(table: Table<K, V>) {
        let Table { handle } = table;
        raw_table::destroy_empty(&handle)
    }

    /// Returns the size of the table, the number of key-value pairs
    public fun length<K: copy + drop, V>(table: &Table<K, V>): u64 {
        raw_table::length(&table.handle)
    }

    /// Returns true iff the table is empty (if `length` returns `0`)
    public fun is_empty<K: copy + drop, V>(table: &Table<K, V>): bool {
        raw_table::length(&table.handle) == 0
    }

    /// Drop a possibly non-empty table.
    /// Usable only if the value type `V` has the `drop` ability
    public fun drop<K: copy + drop, V: drop>(table: Table<K, V>) {
        let Table { handle } = table;
        raw_table::drop_unchecked(&handle)
    }


    #[test_only]
    /// Testing only: allows to drop a table even if it is not empty.
    public fun drop_unchecked<K: copy + drop, V>(table: Table<K, V>) {
        let Table { handle } = table;
        raw_table::drop_unchecked(&handle)
    }


    /// Returns table handle of `table`.
    public fun handle<K: copy + drop, V>(table: &Table<K, V>): &ObjectID {
        &table.handle
    }

    #[test_only]
    struct TableHolder<phantom K: copy + drop, phantom V: drop> has key {
        t: Table<K, V>
    }

    #[test(account = @0x1)]
    fun test_upsert(account: signer) {
        let sender = std::signer::address_of(&account);
        let ctx = moveos_std::context::new_test_context(sender);
        let t = new<u64, u8>(&mut ctx);
        let key: u64 = 111;
        let error_code: u64 = 1;
        assert!(!contains(&t, key), error_code);
        upsert(&mut t, key, 12);
        assert!(*borrow(&t, key) == 12, error_code);
        upsert(&mut t, key, 23);
        assert!(*borrow(&t, key) == 23, error_code);

        drop_unchecked(t);
        moveos_std::context::drop_test_context(ctx);
    }

    #[test(account = @0x1)]
    fun test_borrow_with_default(account: signer) {
        let sender = std::signer::address_of(&account);
        let ctx = moveos_std::context::new_test_context(sender);
        let t = new<u64, u8>(&mut ctx);
        let key: u64 = 100;
        let error_code: u64 = 1;
        assert!(!contains(&t, key), error_code);
        assert!(*borrow_with_default(&t, key, &12) == 12, error_code);
        add(&mut t, key, 1);
        assert!(*borrow_with_default(&t, key, &12) == 1, error_code);

        drop_unchecked(t);
        moveos_std::context::drop_test_context(ctx);
    }

    #[test(account = @0x1)]
    fun test_all(account: signer) {
        let sender = std::signer::address_of(&account);
        let ctx = moveos_std::context::new_test_context(sender);
        let t = new<u64, u8>(&mut ctx);
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
        moveos_std::context::drop_test_context(ctx);
    }

    #[test(account = @0x1)]
    #[expected_failure]
    fun test_add_key_exist_failure(account: signer) {
        let sender = std::signer::address_of(&account);
        let ctx = moveos_std::context::new_test_context(sender);
        let t = new<u64, u8>(&mut ctx);
        let key: u64 = 100;
        add(&mut t, key, 1);
        add(&mut t, key, 2);

        drop_unchecked(t);
        moveos_std::context::drop_test_context(ctx);
    }

    #[test(account = @0x1)]
    #[expected_failure]
    fun test_borrow_key_not_exist_failure(account: signer) {
        let sender = std::signer::address_of(&account);
        let ctx = moveos_std::context::new_test_context(sender);
        let t = new<u64, u8>(&mut ctx);
        let key: u64 = 100;
        let _ = borrow(&mut t, key);

        drop_unchecked(t);
        moveos_std::context::drop_test_context(ctx);
    }

    #[test(account = @0x1)]
    #[expected_failure]
    fun test_borrow_mut_key_not_exist_failure(account: signer) {
        let sender = std::signer::address_of(&account);
        let ctx = moveos_std::context::new_test_context(sender);
        let t = new<u64, u8>(&mut ctx);
        let key: u64 = 100;
        let _ = borrow_mut(&mut t, key);

        drop_unchecked(t);
        moveos_std::context::drop_test_context(ctx);
    }

    #[test(account = @0x1)]
    #[expected_failure]
    fun test_remove_key_not_exist_failure(account: signer) {
        let sender = std::signer::address_of(&account);
        let ctx = moveos_std::context::new_test_context(sender);
        let t = new<u64, u8>(&mut ctx);
        let key: u64 = 100;
        remove(&mut t, key);

        drop_unchecked(t);
        moveos_std::context::drop_test_context(ctx);
    }

    #[test(account = @0x1)]
    fun test_nested_table(account: signer) {
        let sender = std::signer::address_of(&account);
        let ctx = moveos_std::context::new_test_context(sender);
        let t1 = new<u64, Table<u8, u32>>(&mut ctx);
        let t2 = new<u8, u32>(&mut ctx);
        let t2_id = t2.handle;

        let t1_key = 2u64;
        let t2_key = 1u8;
        add(&mut t2, t2_key, 32u32);
        add(&mut t1, t1_key, t2);

        let borrowed_t2 = borrow(&t1, t1_key);
        let value = borrow(borrowed_t2, t2_key);
        assert!(*value == 32u32, 1);

        let t2 = new_with_id<u8, u32>(copy t2_id);
        assert!(contains(&t2, t2_key), 2);
        let Table { handle: _ } = t2;

        let borrowed_mut_t2 = borrow_mut(&mut t1, t1_key);
        remove(borrowed_mut_t2, t2_key);

        let t3 = new_with_id<u8, u32>(t2_id);
        assert!(!contains(&t3, t2_key), 2);
        
        drop_unchecked(t3); // No need to drop t2 as t2 shares same handle with t3
        drop_unchecked(t1);
        moveos_std::context::drop_test_context(ctx);
    }

    #[test(account = @0x1)]
    #[expected_failure]
    fun test_destroy_nonempty_table(account: signer) {
        let sender = std::signer::address_of(&account);
        let ctx = moveos_std::context::new_test_context(sender);
        let t = new<u64, u8>(&mut ctx);
        let key: u64 = 100;
        add(&mut t, key, 1);

        destroy_empty(t);
        moveos_std::context::drop_test_context(ctx);
    }

    #[test(account = @0x1)]
    #[expected_failure]
    fun test_nested_table_destroy(account: signer) {
        let sender = std::signer::address_of(&account);
        let ctx = moveos_std::context::new_test_context(sender);
        let t1 = new<u64, Table<u8, u32>>(&mut ctx);
        let t2 = new<u8, u32>(&mut ctx);
        let t2_id = t2.handle;

        let t1_key = 2u64;
        let t2_key = 1u8;
        add(&mut t2, t2_key, 32u32);
        add(&mut t1, t1_key, t2);

        destroy_empty(t1);

        let t2 = new_with_id<u8, u32>(t2_id);
        assert!(*borrow(&t2, t2_key) == 32u32, 1);

        drop_unchecked(t2);
        moveos_std::context::drop_test_context(ctx);
    }


}
