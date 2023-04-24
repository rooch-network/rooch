/// TypeTable is a table use struct Type as Key, struct as Value

module moveos_std::type_table {

    use std::string::{String};
    use moveos_std::raw_table;
    use moveos_std::tx_context::TxContext;
    use moveos_std::type_info;

    friend moveos_std::account_storage;

    struct TypeTable has store {
        handle: address,
    }

    /// Create a new Table.
    public fun new(ctx: &mut TxContext): TypeTable {
        TypeTable {
            handle: raw_table::new_table_handle(ctx),
        }
    }

    fun key<V>(): String {
        type_info::type_name<V>()
    }

    /// Add a new entry to the table. Aborts if an entry for this
    /// key already exists. The entry itself is not stored in the
    /// table, and cannot be discovered from it.
    public(friend) fun add<V: key>(table: &mut TypeTable, val: V) {
        raw_table::add<String, V>(*&table.handle, key<V>(), val)
    }

    /// Acquire an immutable reference to the value which `key` maps to.
    /// Aborts if there is no entry for `key`.
    public(friend) fun borrow<V: key>(table: &TypeTable): &V {
        raw_table::borrow<String, V>(*&table.handle, key<V>())
    }

    /// Acquire a mutable reference to the value which `key` maps to.
    /// Aborts if there is no entry for `key`.
    public(friend) fun borrow_mut<V: key>(table: &mut TypeTable): &mut V {
        raw_table::borrow_mut<String, V>(*&table.handle, key<V>())
    }

    /// Remove from `table` and return the value which `key` maps to.
    /// Aborts if there is no entry for `key`.
    public(friend) fun remove<V: key>(table: &mut TypeTable): V {
        raw_table::remove<String, V>(*&table.handle, key<V>())
    }

    /// Returns true if `table` contains an entry for `key`.
    public(friend) fun contains<V>(table: &TypeTable): bool {
        raw_table::contains<String>(*&table.handle, key<V>())
    }

    #[test_only]
    /// Testing only: allows to drop a table even if it is not empty.
    public(friend) fun drop_unchecked(table: TypeTable) {
        let TypeTable{handle} = table;
        raw_table::drop_unchecked(handle)
    }

    ///TODO should open the destroy function to public?
    public(friend) fun destroy(table: TypeTable) {
        let TypeTable{handle} = table;
        raw_table::destroy(handle)
    }

}
