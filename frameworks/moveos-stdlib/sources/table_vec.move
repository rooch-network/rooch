// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

// Copyright (c) Mysten Labs, Inc.
// SPDX-License-Identifier: Apache-2.0
// original source: https://github.com/MystenLabs/sui/blob/b09ef4ef9942f1af6ee09b04e9a7694f0aabcc06/crates/sui-framework/packages/sui-framework/sources/table_vec.move

/// A basic scalable vector library implemented using `Table`.
module moveos_std::table_vec {
    use moveos_std::table::{Self, Table};

    struct TableVec<phantom Element: store> has store {
        /// The contents of the table vector.
        contents: Table<u64, Element>,
    }

    const ErrorIndexOutOfBound: u64 = 1;
    const ErrorTableNonEmpty: u64 = 2;

    /// Create an empty TableVec.
    public fun new<Element: store>(): TableVec<Element> {
        TableVec {
            contents: table::new()
        }
    }

    /// Return a TableVec of size one containing element `e`.
    public fun singleton<Element: store>(e: Element): TableVec<Element> {
        let t = new();
        push_back(&mut t, e);
        t
    }

    /// Return the length of the TableVec.
    public fun length<Element: store>(t: &TableVec<Element>): u64 {
        table::length(&t.contents)
    }

    /// Return if the TableVec is empty or not.
    public fun is_empty<Element: store>(t: &TableVec<Element>): bool {
        length(t) == 0
    }

    /// Acquire an immutable reference to the `i`th element of the TableVec `t`.
    /// Aborts if `i` is out of bounds.
    public fun borrow<Element: store>(t: &TableVec<Element>, i: u64): &Element {
        assert!(length(t) > i, ErrorIndexOutOfBound);
        table::borrow(&t.contents, i)
    }

    /// Add element `e` to the end of the TableVec `t`.
    public fun push_back<Element: store>(t: &mut TableVec<Element>, e: Element) {
        let key = length(t);
        table::add(&mut t.contents, key, e);
    }

    /// Return a mutable reference to the `i`th element in the TableVec `t`.
    /// Aborts if `i` is out of bounds.
    public fun borrow_mut<Element: store>(t: &mut TableVec<Element>, i: u64): &mut Element {
        assert!(length(t) > i, ErrorIndexOutOfBound);
        table::borrow_mut(&mut t.contents, i)
    }

    /// Pop an element from the end of TableVec `t`.
    /// Aborts if `t` is empty.
    public fun pop_back<Element: store>(t: &mut TableVec<Element>): Element {
        let length = length(t);
        assert!(length > 0, ErrorIndexOutOfBound);
        table::remove(&mut t.contents, length - 1)
    }

    /// Destroy the TableVec `t`.
    /// Aborts if `t` is not empty.
    public fun destroy_empty<Element: store>(t: TableVec<Element>) {
        assert!(length(&t) == 0, ErrorTableNonEmpty);
        let TableVec { contents } = t;
        table::destroy_empty(contents);
    }

    /// Drop a possibly non-empty TableVec `t`.
    /// Usable only if the value type `Element` has the `drop` ability
    public fun drop<Element: drop + store>(t: TableVec<Element>) {
        let TableVec { contents } = t;
        table::drop(contents)
    }

    /// Swaps the elements at the `i`th and `j`th indices in the TableVec `t`.
    /// Aborts if `i` or `j` is out of bounds.
    public fun swap<Element: store>(t: &mut TableVec<Element>, i: u64, j: u64) {
        assert!(length(t) > i, ErrorIndexOutOfBound);
        assert!(length(t) > j, ErrorIndexOutOfBound);
        if (i == j) { return };
        let element_i = table::remove(&mut t.contents, i);
        let element_j = table::remove(&mut t.contents, j);
        table::add(&mut t.contents, j, element_i);
        table::add(&mut t.contents, i, element_j);
    }

    /// Swap the `i`th element of the TableVec `t` with the last element and then pop the TableVec.
    /// This is O(1), but does not preserve ordering of elements in the TableVec.
    /// Aborts if `i` is out of bounds.
    public fun swap_remove<Element: store>(t: &mut TableVec<Element>, i: u64): Element {
        assert!(length(t) > i, ErrorIndexOutOfBound);
        let last_idx = length(t) - 1;
        swap(t, i, last_idx);
        pop_back(t)
    }

    /// Return if the TableVec `t` contains the element at index `i`.
    public fun contains<Element: store>(t: &TableVec<Element>, i: u64): bool {
        length(t) > i
    }
}
