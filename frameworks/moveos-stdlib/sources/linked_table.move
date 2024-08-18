// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

/// Similar to `moveos_std::table` but the values are linked together, allowing for ordered insertion and
/// removal
module moveos_std::linked_table {
    use std::option::{Self, Option};
    use moveos_std::object::Object;
    use moveos_std::object;



    // Attempted to destroy a non-empty table
    const ErrorTableNotEmpty: u64 = 0;
    // Attempted to remove the front or back of an empty table
    const ErrorTableIsEmpty: u64 = 1;

    struct TablePlaceholder has key {
        _placeholder: bool,
    }

    struct LinkedTable<K: copy + drop + store, phantom V: store> has key, store {
        handle: Object<TablePlaceholder>,
        /// the number of key-value pairs in the table
        size: u64,
        /// the front of the table, i.e. the key of the first entry
        head: Option<K>,
        /// the back of the table, i.e. the key of the last entry
        tail: Option<K>,
    }

    struct Node<K: copy + drop + store, V: store> has store {
        /// the previous key
        prev: Option<K>,
        /// the next key
        next: Option<K>,
        /// the value being stored
        value: V
    }

    /// Creates a new, empty table
    public fun new<K: copy + drop + store, V: store>(): LinkedTable<K, V> {
        LinkedTable {
            handle: object::new(TablePlaceholder{_placeholder: false}),
            size: 0,
            head: option::none(),
            tail: option::none(),
        }

    }

    /// Returns the key for the first element in the table, or None if the table is empty
    public fun front<K: copy + drop + store, V: store>(table: &LinkedTable<K, V>): &Option<K> {
        &table.head
    }

    /// Returns the key for the last element in the table, or None if the table is empty
    public fun back<K: copy + drop + store, V: store>(table: &LinkedTable<K, V>): &Option<K> {
        &table.tail
    }

    /// Inserts a key-value pair at the front of the table, i.e. the newly inserted pair will be
    /// the first element in the table
    /// Aborts with if the table already has an entry with
    /// that key `k: K`.
    public fun push_front<K: copy + drop + store, V: store>(
        table: &mut LinkedTable<K, V>,
        k: K,
        value: V,
    ) {
        table.size = table.size + 1;
        let old_head = option::swap_or_fill(&mut table.head, k);
        if (option::is_none(&table.tail)) option::fill(&mut table.tail, k);
        let prev = option::none();
        let next = if (option::is_some(&old_head)) {
            let old_head_k = option::destroy_some(old_head);
            let node = object::borrow_mut_field<TablePlaceholder, K, Node<K, V>>(&mut table.handle, old_head_k);
            node.prev = option::some(k);
            option::some(old_head_k)
        } else {
            option::none()
        };
        object::add_field(&mut table.handle, k, Node { prev, next, value });
    }

    /// Inserts a key-value pair at the back of the table, i.e. the newly inserted pair will be
    /// the last element in the table
    /// Aborts if the table already has an entry with
    /// that key `k: K`.
    public fun push_back<K: copy + drop + store, V: store>(
        table: &mut LinkedTable<K, V>,
        k: K,
        value: V,
    ) {
        table.size = table.size + 1;
        if (option::is_none(&table.head)) option::fill(&mut table.head, k);
        let old_tail = option::swap_or_fill(&mut table.tail, k);
        let prev = if (option::is_some(&old_tail)) {
            let old_tail_k = option::destroy_some(old_tail);
            object::borrow_mut_field<TablePlaceholder, K, Node<K, V>>(&mut table.handle, old_tail_k).next = option::some(k);
            option::some(old_tail_k)
        } else {
            option::none()
        };
        let next = option::none();
        object::add_field(&mut table.handle, k, Node { prev, next, value });
    }

    /// Immutable borrows the value associated with the key in the table `table: &LinkedTable<K, V>`.
    /// Aborts if the table does not have an entry with
    /// that key `k: K`.
    public fun borrow<K: copy + drop + store, V: store>(table: &LinkedTable<K, V>, k: K): &V {
        &object::borrow_field<TablePlaceholder, K, Node<K, V>>(&table.handle, k).value
    }

    /// Mutably borrows the value associated with the key in the table `table: &mut LinkedTable<K, V>`.
    /// Aborts if the table does not have an entry with
    /// that key `k: K`.
    public fun borrow_mut<K: copy + drop + store, V: store>(
        table: &mut LinkedTable<K, V>,
        k: K,
    ): &mut V {
        &mut object::borrow_mut_field<TablePlaceholder, K, Node<K, V>>(&mut table.handle, k).value
    }

    /// Borrows the key for the previous entry of the specified key `k: K` in the table
    /// `table: &LinkedTable<K, V>`. Returns None if the entry does not have a predecessor.
    /// Aborts if the table does not have an entry with
    /// that key `k: K`
    public fun prev<K: copy + drop + store, V: store>(table: &LinkedTable<K, V>, k: K): &Option<K> {
        &object::borrow_field<TablePlaceholder, K, Node<K, V>>(&table.handle, k).prev
    }

    /// Borrows the key for the next entry of the specified key `k: K` in the table
    /// `table: &LinkedTable<K, V>`. Returns None if the entry does not have a predecessor.
    /// Aborts if the table does not have an entry with
    /// that key `k: K`
    public fun next<K: copy + drop + store, V: store>(table: &LinkedTable<K, V>, k: K): &Option<K> {
        &object::borrow_field<TablePlaceholder, K, Node<K, V>>(&table.handle, k).next
    }

    /// Removes the key-value pair in the table `table: &mut LinkedTable<K, V>` and returns the value.
    /// This splices the element out of the ordering.
    /// Aborts if the table does not have an entry with
    /// that key `k: K`. Note: this is also what happens when the table is empty.
    public fun remove<K: copy + drop + store, V: store>(table: &mut LinkedTable<K, V>, k: K): V {
        let Node<K, V> { prev, next, value } = object::remove_field<TablePlaceholder, K, Node<K, V>>(&mut table.handle, k);
        if (option::is_some(&prev)) {
            object::borrow_mut_field<TablePlaceholder, K, Node<K, V>>(&mut table.handle, *option::borrow(&prev)).next = next
        };
        if (option::is_some(&next)) {
            object::borrow_mut_field<TablePlaceholder, K, Node<K, V>>(&mut table.handle, *option::borrow(&next)).prev = prev
        };
        table.size = table.size - 1;
        if (option::borrow(&table.head) == &k) table.head = next;
        if (option::borrow(&table.tail) == &k) table.tail = prev;
        value
    }

    /// Removes the front of the table `table: &mut LinkedTable<K, V>` and returns the value.
    /// Aborts with `ETableIsEmpty` if the table is empty
    public fun pop_front<K: copy + drop + store, V: store>(table: &mut LinkedTable<K, V>): (K, V) {
        assert!(option::is_some(&table.head), ErrorTableIsEmpty);
        let head = *option::borrow(&table.head);
        (head, remove(table, head))
    }

    /// Removes the back of the table `table: &mut LinkedTable<K, V>` and returns the value.
    /// Aborts with `ETableIsEmpty` if the table is empty
    public fun pop_back<K: copy + drop + store, V: store>(table: &mut LinkedTable<K, V>): (K, V) {
        assert!(option::is_some(&table.tail), ErrorTableIsEmpty);
        let tail = *option::borrow(&table.tail);
        (tail, remove(table, tail))
    }

    /// Returns true iff there is a value associated with the key `k: K` in table
    /// `table: &LinkedTable<K, V>`
    public fun contains<K: copy + drop + store, V: store>(table: &LinkedTable<K, V>, k: K): bool {
        object::contains_field_with_type<TablePlaceholder, K, Node<K, V>>(&table.handle, k)
    }

    /// Returns the size of the table, the number of key-value pairs
    public fun length<K: copy + drop + store, V: store>(table: &LinkedTable<K, V>): u64 {
        table.size
    }

    /// Returns true if the table is empty (if `length` returns `0`)
    public fun is_empty<K: copy + drop + store, V: store>(table: &LinkedTable<K, V>): bool {
        table.size == 0
    }

    /// Destroys an empty table
    /// Aborts with `ETableNotEmpty` if the table still contains values
    public fun destroy_empty<K: copy + drop + store, V: store>(table: LinkedTable<K, V>) {
        let LinkedTable { handle, size, head: _, tail: _ } = table;
        let TablePlaceholder{_placeholder:_} = object::remove_unchecked(handle);

        assert!(size == 0, ErrorTableNotEmpty);
    }

    /// Drop a possibly non-empty table.
    /// Usable only if the value type `V` has the `drop` ability
    public fun drop<K: copy + drop + store, V: drop + store>(table_obj: Object<LinkedTable<K, V>>) {
        let table = object::remove(table_obj);
        let LinkedTable { handle, size: _, head: _, tail: _ } = table;
        let TablePlaceholder{_placeholder:_} = object::remove_unchecked(handle);

    }

    #[test]
    fun test_linked_table(){
        let linked_table = new<u64, u64>();
        push_back(&mut linked_table, 1, 2);
        push_back(&mut linked_table, 3, 4);
        assert!(length(&mut linked_table) == 2 , 0);
        let pre = prev(&linked_table, 3);
        let next = next(&linked_table, 1);
        let front = front(&linked_table);
        let back = back(&linked_table);

        assert!(front == &option::some(1), 1);
        assert!(back == &option::some(3), 2);
        assert!(pre == &option::some(1), 3);
        assert!(next == &option::some(3), 4);
        push_back(&mut linked_table, 5, 6);
        let (key1, value1) = pop_front(&mut linked_table);
        assert!(front(&linked_table) == &option::some(3), 5);
        let (key2, value2) = pop_front(&mut linked_table);
        assert!(key1 == 1 && value1 == 2, 6);
        assert!(key2 == 3 && value2 == 4, 7);
        let value = remove(&mut linked_table, 5);
        assert!(value == 6, 8);
        destroy_empty(linked_table)
    }
}
