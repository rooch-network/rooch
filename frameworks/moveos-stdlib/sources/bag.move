// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

// Copyright (c) Mysten Labs, Inc.
// SPDX-License-Identifier: Apache-2.0

// Same API as sui::bag https://github.com/MystenLabs/sui/blob/034c7e2300f5b02c0848d0fa29c6da88dbc45653/crates/sui-framework/packages/sui-framework/sources/bag.move

/// A bag is a heterogeneous map-like collection. The collection is similar to `moveos_std::table` in that
/// its keys and values are not stored within the `Bag` value, but instead are stored using Sui's
/// object system. The `Bag` struct acts only as a handle into the object system to retrieve those
/// keys and values.
/// Note that this means that `Bag` values with exactly the same key-value mapping will not be
/// equal, with `==`, at runtime. For example
/// ```
/// let bag1 = bag::new();
/// let bag2 = bag::new();
/// bag::add(&mut bag1, 0, false);
/// bag::add(&mut bag1, 1, true);
/// bag::add(&mut bag2, 0, false);
/// bag::add(&mut bag2, 1, true);
/// // bag1 does not equal bag2, despite having the same entries
/// assert!(&bag1 != &bag2, 0);
/// ```
module moveos_std::bag {
    use moveos_std::object::{Self, Object};

    /// If add a non-dropable value to a dropable bag, will abort with this code
    const ErrorBagIsDropable: u64 = 1;
    /// If drop a non-dropable bag, will abort with this code
    const ErrorBagIsNotDropable: u64 = 2;

    struct BagInner has key{
        dropable: bool,
    }

    struct Bag has store {
        handle: Object<BagInner>,
    }

    /// Creates a new, empty bag
    public fun new(): Bag {
        let obj = object::new(BagInner{ dropable: false });
        Bag {
            handle: obj,
        }
    }

    /// Creates a new, empty bag that can be dropped, so all its values should be dropped
    public fun new_dropable(): Bag {
        let obj = object::new(BagInner{ dropable: true });
        Bag {
            handle: obj,
        }
    }

    /// Adds a key-value pair to the bag `bag: &mut Bag`
    /// If the bag is dropable, should call `add_dropable` instead
    public fun add<K: copy + drop + store, V: store>(bag: &mut Bag, k: K, v: V) {
        assert!(!object::borrow(&bag.handle).dropable, ErrorBagIsDropable);
        object::add_field(&mut bag.handle, k, v);
    }

    /// Adds a key-value pair to the bag `bag: &mut Bag`, the `V` should be dropable
    public fun add_dropable<K: copy + drop + store, V: store + drop>(bag: &mut Bag, k: K, v: V) {
        object::add_field(&mut bag.handle, k, v);
    }

    /// Immutable borrows the value associated with the key in the bag `bag: &Bag`.
    public fun borrow<K: copy + drop + store, V: store>(bag: &Bag, k: K): &V {
        object::borrow_field(&bag.handle, k)
    }

    /// Mutably borrows the value associated with the key in the bag `bag: &mut Bag`.
    public fun borrow_mut<K: copy + drop + store, V: store>(bag: &mut Bag, k: K): &mut V {
        object::borrow_mut_field(&mut bag.handle, k)
    }

    /// Mutably borrows the key-value pair in the bag `bag: &mut Bag` and returns the value.
    public fun remove<K: copy + drop + store, V: store>(bag: &mut Bag, k: K): V {
        let v = object::remove_field(&mut bag.handle, k);
        v
    }

    /// Returns true iff there is an value associated with the key `k: K` in the bag `bag: &Bag`
    public fun contains<K: copy + drop + store>(bag: &Bag, k: K): bool {
        object::contains_field(&bag.handle, k)
    }

    /// Returns true iff there is an value associated with the key `k: K` in the bag `bag: &Bag` and the value is of type `V`
    public fun contains_with_type<K: copy + drop + store, V: store>(bag: &Bag, k: K): bool {
        object::contains_field_with_type<BagInner, K, V>(&bag.handle, k)
    }

    /// Returns the size of the bag, the number of key-value pairs
    public fun length(bag: &Bag): u64 {
        object::field_size(&bag.handle)
    }

    /// Returns true iff the bag is empty (if `length` returns `0`)
    public fun is_empty(bag: &Bag): bool {
        length(bag) == 0
    }

    /// Destroys an empty bag
    public fun destroy_empty(bag: Bag) {
        let Bag{handle} = bag;
        let BagInner{dropable:_} = object::remove(handle);
    }

    /// Drops a bag, the bag should be dropable
    public fun drop(bag: Bag) {
        let Bag{handle} = bag;
        let BagInner{dropable} = object::remove_unchecked(handle);
        assert!(dropable, ErrorBagIsNotDropable);
    }

    #[test_only]
    /// Testing only: allows to drop a bag even if it is not empty.
    public fun drop_unchecked(bag: Bag) {
        let Bag{handle} = bag;
        let BagInner{dropable:_} = object::remove_unchecked(handle);
    }
}
