// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

module moveos_std::object_dynamic_field {
    use moveos_std::object;
    use moveos_std::object::Object;

    /// Add a dynamic filed to the object. Aborts if an entry for this
    /// key already exists. The entry itself is not stored in the
    /// table, and cannot be discovered from it.
    public fun add_field<T: key, K: copy + drop, V>(obj: &mut Object<T>, key: K, val: V) {
        object::add_field<K,V>(object::id(obj), key, val)
    }

    /// Acquire an immutable reference to the value which `key` maps to.
    /// Aborts if there is no entry for `key`.
    public fun borrow_field<T: key, K: copy + drop, V>(obj: &Object<T>, key: K): &V {
        object::borrow_field<K, V>(object::id(obj), key)
    }

    /// Acquire an immutable reference to the value which `key` maps to.
    /// Returns specified default value if there is no entry for `key`.
    public fun borrow_field_with_default<T: key, K: copy + drop, V>(obj: &Object<T>, key: K, default: &V): &V {
        object::borrow_field_with_default<K, V>(object::id(obj), key, default)
    }

    /// Acquire a mutable reference to the value which `key` maps to.
    /// Aborts if there is no entry for `key`.
    public fun borrow_mut_field<T: key, K: copy + drop, V>(obj: &mut Object<T>, key: K): &mut V {
        object::borrow_mut_field<K, V>(object::id(obj), key)
    }

    /// Acquire a mutable reference to the value which `key` maps to.
    /// Insert the pair (`key`, `default`) first if there is no entry for `key`.
    public fun borrow_mut_field_with_default<T: key, K: copy + drop, V: drop>(obj: &mut Object<T>, key: K, default: V): &mut V {
        object::borrow_mut_field_with_default<K, V>(object::id(obj), key, default)
    }

    /// Insert the pair (`key`, `value`) if there is no entry for `key`.
    /// update the value of the entry for `key` to `value` otherwise
    public fun upsert_field<T: key, K: copy + drop, V: drop>(obj: &mut Object<T>, key: K, value: V) {
        object::upsert_field<K, V>(object::id(obj), key, value)
    }

    /// Remove from `table` and return the value which `key` maps to.
    /// Aborts if there is no entry for `key`.
    public fun remove_field<T: key, K: copy + drop, V>(obj: &mut Object<T>, key: K): V {
        object::remove_field<K, V>(object::id(obj), key)
    }

    /// Returns true if `table` contains an entry for `key`.
    public fun contains_field<T: key, K: copy + drop>(obj: &Object<T>, key: K): bool {
        object::contains_field<K>(object::id(obj), key)
    }

    /// Returns the size of the table, the number of key-value pairs
    public fun field_size<T: key>(obj: &Object<T>): u64 {
        object::table_length(object::id(obj))
    }

}
