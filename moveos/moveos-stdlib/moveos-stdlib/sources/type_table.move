/// TypeTable is a table use struct Type as Key, struct as Value

module moveos_std::type_table {

    use std::ascii::{String};
    use moveos_std::raw_table;
    use moveos_std::tx_context::TxContext;
    use moveos_std::object_id::ObjectID;

    friend moveos_std::account_storage;
    friend moveos_std::events;

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

    #[private_generics(T)]
    /// Add a new entry to the table. Aborts if an entry for this
    /// key already exists. The entry itself is not stored in the
    /// table, and cannot be discovered from it.
    public fun add<V: key>(table: &mut TypeTable, val: V) {
        add_internal<V>(table, val)
    }

    public(friend) fun add_internal<V: key>(table: &mut TypeTable, val: V) {
        raw_table::add<String, V>(&table.handle, key<V>(), val)
    }

    #[private_generics(T)]
    /// Acquire an immutable reference to the value which `key` maps to.
    /// Aborts if there is no entry for `key`.
    public fun borrow<V: key>(table: &TypeTable): &V {
        borrow_internal<V>(table)
    }

    public(friend) fun borrow_internal<V: key>(table: &TypeTable): &V {
        raw_table::borrow<String, V>(&table.handle, key<V>())
    }

    #[private_generics(T)]
    /// Acquire a mutable reference to the value which `key` maps to.
    /// Aborts if there is no entry for `key`.
    public fun borrow_mut<V: key>(table: &mut TypeTable): &mut V {
        borrow_mut_internal<V>(table)
    }

    public(friend) fun borrow_mut_internal<V: key>(table: &mut TypeTable): &mut V {
        raw_table::borrow_mut<String, V>(&table.handle, key<V>())
    }

    #[private_generics(T)]
    /// Remove from `table` and return the value which `key` maps to.
    /// Aborts if there is no entry for `key`.
    public fun remove<V: key>(table: &mut TypeTable): V {
        remove_internal<V>(table)
    }

    public(friend) fun remove_internal<V: key>(table: &mut TypeTable): V {
        raw_table::remove<String, V>(&table.handle, key<V>())
    }

    #[private_generics(T)]
    /// Returns true if `table` contains an entry for `key`.
    public fun contains<V: key>(table: &TypeTable): bool {
        raw_table::contains<String>(&table.handle, key<V>())
    }

    public(friend) fun contains_internal<V: key>(table: &TypeTable): bool {
        raw_table::contains<String>(&table.handle, key<V>())
    }

    #[test_only]
    /// Testing only: allows to drop a table even if it is not empty.
    public fun drop_unchecked(table: TypeTable) {
        let TypeTable{handle} = table;
        raw_table::drop_unchecked(&handle)
    }

    ///TODO should open the destroy function to public?
    public(friend) fun destroy(table: TypeTable) {
        let TypeTable{handle} = table;
        raw_table::destroy(&handle)
    }

}
