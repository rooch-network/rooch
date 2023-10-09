// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

/// Source from https://github.com/aptos-labs/aptos-core/blob/d50af4db34a6929642603c3896a0af17984b3054/aptos-move/framework/aptos-stdlib/sources/simple_map.move
/// Do some refator because we do not support inline and lambda yet.
/// This module provides a solution for unsorted maps, that is it has the properties that
/// 1) Keys point to Values
/// 2) Each Key must be unique
/// 3) A Key can be found within O(N) time
/// 4) The keys are unsorted.
/// 5) Adds and removals take O(N) time
module moveos_std::simple_map {
    use std::error;
    use std::option;
    use std::vector;

    /// Map key already exists
    const ErrorKeyAlreadyExists: u64 = 1;
    /// Map key is not found
    const ErrorKeyNotFound: u64 = 2;

    struct SimpleMap<Key, Value> has copy, drop, store {
        data: vector<Element<Key, Value>>,
    }

    struct Element<Key, Value> has copy, drop, store {
        key: Key,
        value: Value,
    }

    public fun length<Key: store, Value: store>(map: &SimpleMap<Key, Value>): u64 {
        vector::length(&map.data)
    }

    public fun create<Key: store, Value: store>(): SimpleMap<Key, Value> {
        SimpleMap {
            data: vector::empty(),
        }
    }

    public fun borrow<Key: store, Value: store>(
        map: &SimpleMap<Key, Value>,
        key: &Key,
    ): &Value {
        let maybe_idx = find(map, key);
        assert!(option::is_some(&maybe_idx), error::invalid_argument(ErrorKeyNotFound));
        let idx = option::extract(&mut maybe_idx);
        &vector::borrow(&map.data, idx).value
    }

    public fun borrow_mut<Key: store, Value: store>(
        map: &mut SimpleMap<Key, Value>,
        key: &Key,
    ): &mut Value {
        let maybe_idx = find(map, key);
        assert!(option::is_some(&maybe_idx), error::invalid_argument(ErrorKeyNotFound));
        let idx = option::extract(&mut maybe_idx);
        &mut vector::borrow_mut(&mut map.data, idx).value
    }

    public fun contains_key<Key: store, Value: store>(
        map: &SimpleMap<Key, Value>,
        key: &Key,
    ): bool {
        let maybe_idx = find(map, key);
        option::is_some(&maybe_idx)
    }

    public fun destroy_empty<Key: store, Value: store>(map: SimpleMap<Key, Value>) {
        let SimpleMap { data } = map;
        vector::destroy_empty(data);
    }

    public fun add<Key: store, Value: store>(
        map: &mut SimpleMap<Key, Value>,
        key: Key,
        value: Value,
    ) {
        let maybe_idx = find(map, &key);
        assert!(option::is_none(&maybe_idx), error::invalid_argument(ErrorKeyAlreadyExists));

        vector::push_back(&mut map.data, Element { key, value });
    }

    /// Insert key/value pair or update an existing key to a new value
    public fun upsert<Key: store, Value: store>(
        map: &mut SimpleMap<Key, Value>,
        key: Key,
        value: Value
    ): (std::option::Option<Key>, std::option::Option<Value>) {
        let data = &mut map.data;
        let len = vector::length(data);
        let i = 0;
        while (i < len) {
            let element = vector::borrow(data, i);
            if (&element.key == &key) {
                vector::push_back(data, Element { key, value});
                vector::swap(data, i, len);
                let Element { key, value } = vector::pop_back(data);
                return (std::option::some(key), std::option::some(value))
            };
            i = i + 1;
        };
        vector::push_back(&mut map.data, Element { key, value });
        (std::option::none(), std::option::none())
    }

    /// Return all keys in the map. This requires keys to be copyable.
    public fun keys<Key: copy, Value>(map: &SimpleMap<Key, Value>): vector<Key> {
        let i = 0;
        let keys: vector<Key> = vector::empty();
        let len = vector::length(&map.data);
        while (i < len) {
            let e = vector::borrow(&map.data, i);
            vector::push_back(&mut keys, e.key); 
            i = i + 1;
        };
        keys
    }

    /// Return all values in the map. This requires values to be copyable.
    public fun values<Key, Value: copy>(map: &SimpleMap<Key, Value>): vector<Value> {
        let i = 0;
        let values: vector<Value> = vector::empty();
        let len = vector::length(&map.data);
        while (i < len) {
            let e = vector::borrow(&map.data, i);
            vector::push_back(&mut values, e.value); 
            i = i + 1;
        };
        values
    }

    /// Transform the map into two vectors with the keys and values respectively
    /// Primarily used to destroy a map
    public fun to_vec_pair<Key: store, Value: store>(
        map: SimpleMap<Key, Value>
    ): (vector<Key>, vector<Value>) {
        let keys: vector<Key> = vector::empty();
        let values: vector<Value> = vector::empty();
        let SimpleMap { data } = map;
        let i = 0;
        let len = vector::length(&data);
        while (i < len) {
            let e = vector::pop_back(&mut data);
            let Element { key, value } = e; 
            vector::push_back(&mut keys, key); 
            vector::push_back(&mut values, value);
            i = i + 1;
        };
        vector::destroy_empty(data);
        (keys, values)
    } 

    public fun remove<Key: store, Value: store>(
        map: &mut SimpleMap<Key, Value>,
        key: &Key,
    ): (Key, Value) {
        let maybe_idx = find(map, key);
        assert!(option::is_some(&maybe_idx), error::invalid_argument(ErrorKeyNotFound));
        let placement = option::extract(&mut maybe_idx);
        let Element { key, value } = vector::swap_remove(&mut map.data, placement);
        (key, value)
    }

    fun find<Key: store, Value: store>(
        map: &SimpleMap<Key, Value>,
        key: &Key,
    ): option::Option<u64>{
        let len = vector::length(&map.data);
        let i = 0;
        while (i < len) {
            let element = vector::borrow(&map.data, i);
            if (&element.key == key){
                return option::some(i)
            };
            i = i + 1;
        };
        option::none<u64>()
    }

    #[test]
    public fun add_remove_many() {
        let map = create<u64, u64>();

        assert!(length(&map) == 0, 0);
        assert!(!contains_key(&map, &3), 1);
        add(&mut map, 3, 1);
        assert!(length(&map) == 1, 2);
        assert!(contains_key(&map, &3), 3);
        assert!(borrow(&map, &3) == &1, 4);
        *borrow_mut(&mut map, &3) = 2;
        assert!(borrow(&map, &3) == &2, 5);

        assert!(!contains_key(&map, &2), 6);
        add(&mut map, 2, 5);
        assert!(length(&map) == 2, 7);
        assert!(contains_key(&map, &2), 8);
        assert!(borrow(&map, &2) == &5, 9);
        *borrow_mut(&mut map, &2) = 9;
        assert!(borrow(&map, &2) == &9, 10);

        remove(&mut map, &2);
        assert!(length(&map) == 1, 11);
        assert!(!contains_key(&map, &2), 12);
        assert!(borrow(&map, &3) == &2, 13);

        remove(&mut map, &3);
        assert!(length(&map) == 0, 14);
        assert!(!contains_key(&map, &3), 15);

        destroy_empty(map);
    }

    #[test]
    public fun test_keys() {
        let map = create<u64, u64>();
        assert!(keys(&map) == vector[], 0);
        add(&mut map, 2, 1);
        add(&mut map, 3, 1);

        assert!(keys(&map) == vector[2, 3], 0);
    }

    #[test]
    public fun test_values() {
        let map = create<u64, u64>();
        assert!(values(&map) == vector[], 0);
        add(&mut map, 2, 1);
        add(&mut map, 3, 2);

        assert!(values(&map) == vector[1, 2], 0);
    }

    #[test]
    #[expected_failure]
    public fun add_twice() {
        let map = create<u64, u64>();
        add(&mut map, 3, 1);
        add(&mut map, 3, 1);

        remove(&mut map, &3);
        destroy_empty(map);
    }

    #[test]
    #[expected_failure]
    public fun remove_twice() {
        let map = create<u64, u64>();
        add(&mut map, 3, 1);
        remove(&mut map, &3);
        remove(&mut map, &3);

        destroy_empty(map);
    }

    #[test]
    public fun upsert_test() {
        let map = create<u64, u64>();
        // test adding 3 elements using upsert
        upsert<u64, u64>(&mut map, 1, 1 );
        upsert(&mut map, 2, 2 );
        upsert(&mut map, 3, 3 );

        assert!(length(&map) == 3, 0);
        assert!(contains_key(&map, &1), 1);
        assert!(contains_key(&map, &2), 2);
        assert!(contains_key(&map, &3), 3);
        assert!(borrow(&map, &1) == &1, 4);
        assert!(borrow(&map, &2) == &2, 5);
        assert!(borrow(&map, &3) == &3, 6);

        // change mapping 1->1 to 1->4
        upsert(&mut map, 1, 4 );

        assert!(length(&map) == 3, 7);
        assert!(contains_key(&map, &1), 8);
        assert!(borrow(&map, &1) == &4, 9);
    }
}
