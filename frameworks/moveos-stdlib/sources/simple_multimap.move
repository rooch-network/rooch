// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

/// A simple map that stores key/value pairs in a vector, and support multi values for one key.
module moveos_std::simple_multimap {
    use std::option;
    use std::vector;

   
    /// Map key is not found
    const ErrorKeyNotFound: u64 = 1;

    struct SimpleMultiMap<Key, Value> has copy, drop, store {
        data: vector<Element<Key, Value>>,
    }

    struct Element<Key, Value> has copy, drop, store {
        key: Key,
        value: vector<Value>,
    }

    /// Create an empty SimpleMultiMap.
    public fun new<Key, Value>(): SimpleMultiMap<Key, Value> {
        SimpleMultiMap {
            data: vector::empty(),
        }
    }

    
    public fun length<Key, Value>(map: &SimpleMultiMap<Key, Value>): u64 {
        vector::length(&map.data)
    }

    public fun is_empty<Key, Value>(map: &SimpleMultiMap<Key, Value>): bool {
        vector::is_empty(&map.data)
    } 

    public fun borrow<Key, Value>(
        map: &SimpleMultiMap<Key, Value>,
        key: &Key,
    ): &vector<Value> {
        let maybe_idx = find(map, key);
        assert!(option::is_some(&maybe_idx), ErrorKeyNotFound);
        let idx = option::extract(&mut maybe_idx);
        let element = vector::borrow(&map.data, idx);
        &element.value
    }

    public fun borrow_mut<Key, Value>(
        map: &mut SimpleMultiMap<Key, Value>,
        key: &Key,
    ): &mut vector<Value> {
        let maybe_idx = find(map, key);
        assert!(option::is_some(&maybe_idx), ErrorKeyNotFound);
        let idx = option::extract(&mut maybe_idx);
        let element = vector::borrow_mut(&mut map.data, idx);
        &mut element.value
    }

    public fun borrow_first<Key, Value>(
        map: &SimpleMultiMap<Key, Value>,
        key: &Key,
    ): &Value {
        let maybe_idx = find(map, key);
        assert!(option::is_some(&maybe_idx), ErrorKeyNotFound);
        let idx = option::extract(&mut maybe_idx);
        let element = vector::borrow(&map.data, idx);
        vector::borrow(&element.value, 0)
    }

    public fun borrow_first_mut<Key, Value>(
        map: &mut SimpleMultiMap<Key, Value>,
        key: &Key,
    ): &mut Value {
        let maybe_idx = find(map, key);
        assert!(option::is_some(&maybe_idx), ErrorKeyNotFound);
        let idx = option::extract(&mut maybe_idx);
        let element = vector::borrow_mut(&mut map.data, idx);
        vector::borrow_mut(&mut element.value, 0)
    }

    public fun borrow_first_with_default<Key, Value>(
        map: &SimpleMultiMap<Key, Value>,
        key: &Key,
        default: &Value,
    ): &Value {
        let maybe_idx = find(map, key);
        if (option::is_none(&maybe_idx)) {
            default
        } else {
            let idx = option::extract(&mut maybe_idx);
            let element = vector::borrow(&map.data, idx);
            if(vector::is_empty(&element.value)){
                default
            } else {
                vector::borrow(&element.value, 0)
            }
        }
    }

    public fun contains_key<Key, Value>(
        map: &SimpleMultiMap<Key, Value>,
        key: &Key,
    ): bool {
        let maybe_idx = find(map, key);
        option::is_some(&maybe_idx)
    }

    public fun destroy_empty<Key, Value>(map: SimpleMultiMap<Key, Value>) {
        let SimpleMultiMap { data } = map;
        vector::destroy_empty(data);
    } 

    public fun add<Key: store + drop, Value: store>(
        map: &mut SimpleMultiMap<Key, Value>,
        key: Key,
        value: Value,
    ) {
        let maybe_idx = find(map, &key);
        if (option::is_some(&maybe_idx)) {
            let idx = option::extract(&mut maybe_idx);
            let element = vector::borrow_mut(&mut map.data, idx);
            vector::push_back(&mut element.value, value);
        } else {
            vector::push_back(&mut map.data, Element { key, value: vector::singleton(value) });
        }
    }


    /// Return all keys in the map. This requires keys to be copyable.
    public fun keys<Key: copy, Value>(map: &SimpleMultiMap<Key, Value>): vector<Key> {
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
    /// This function flatten the vector<vector<Value>> to vector<Value>
    public fun values<Key, Value: copy>(map: &SimpleMultiMap<Key, Value>): vector<Value> {
        let i = 0;
        let values: vector<Value> = vector::empty();
        let len = vector::length(&map.data);
        while (i < len) {
            let e = vector::borrow(&map.data, i);
            let value_len = vector::length(&e.value);
            let j = 0;
            while(j < value_len){
                let v = *vector::borrow(&e.value, j);
                vector::push_back(&mut values, v); 
                j = j + 1;
            };
            i = i + 1;
        };
        values
    }

    /// Transform the map into two vectors with the keys and values respectively
    /// Primarily used to destroy a map
    /// Note: Do not assume the key's order
    public fun to_vec_pair<Key, Value>(
        map: SimpleMultiMap<Key, Value>
    ): (vector<Key>, vector<vector<Value>>) {
        let keys: vector<Key> = vector::empty();
        let values: vector<vector<Value>> = vector::empty();
        let SimpleMultiMap { data } = map;
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

    public fun remove<Key, Value>(
        map: &mut SimpleMultiMap<Key, Value>,
        key: &Key,
    ): (Key, vector<Value>) {
        let maybe_idx = find(map, key);
        assert!(option::is_some(&maybe_idx), ErrorKeyNotFound);
        let placement = option::extract(&mut maybe_idx);
        let Element { key, value } = vector::swap_remove(&mut map.data, placement);
        (key, value)
    }

    fun find<Key, Value>(
        map: &SimpleMultiMap<Key, Value>,
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
        let map = new<u64, u64>();

        assert!(length(&map) == 0, 0);
        assert!(!contains_key(&map, &3), 1);
        add(&mut map, 3, 1);
        add(&mut map, 3, 2);
        assert!(length(&map) == 1, 2);
        assert!(contains_key(&map, &3), 3);
        assert!(borrow_first(&map, &3) == &1, 4);

        assert!(!contains_key(&map, &2), 5);
        add(&mut map, 2, 5);
        assert!(length(&map) == 2, 6);
        assert!(contains_key(&map, &2), 7);
        assert!(borrow_first(&map, &2) == &5, 8);

        remove(&mut map, &2);
        assert!(length(&map) == 1, 9);
        assert!(!contains_key(&map, &2), 10);
        assert!(borrow_first(&map, &3) == &1, 11);

        remove(&mut map, &3);
        assert!(length(&map) == 0, 12);
        assert!(!contains_key(&map, &3), 13);
        
    }

    #[test]
    public fun test_keys() {
        let map = new<u64, u64>();
        assert!(keys(&map) == vector[], 0);
        add(&mut map, 2, 1);
        add(&mut map, 3, 1);

        assert!(keys(&map) == vector[2, 3], 1);
        
    }

    #[test]
    public fun test_values() {
        let map = new<u64, u64>();
        assert!(values(&map) == vector[], 0);
        add(&mut map, 2, 1);
        add(&mut map, 3, 2);
        add(&mut map, 3, 3);
        let values = values(&map);
        assert!(vector::length(&values) == 3, 1);
        assert!(values == vector[1,2,3], 2);
        
    }

    #[test]
    public fun test_to_vec_pair(){
        let map = new<u64, u64>();
        add(&mut map, 2, 1);
        add(&mut map, 3, 2);
        add(&mut map, 3, 3);
        let (keys, values) = to_vec_pair(map);
        //std::debug::print(&keys);
        assert!(keys == vector[3, 2], 0);
        assert!(values == vector[vector[2,3], vector[1]], 1);
    }

    #[test]
    public fun add_twice() {
        let map = new<u64, u64>();
        add(&mut map, 3, 1);
        add(&mut map, 3, 1);

        remove(&mut map, &3);
        destroy_empty(map);
    }

    #[test]
    #[expected_failure]
    public fun remove_twice() {
        let map = new<u64, u64>();
        add(&mut map, 3, 1);
        remove(&mut map, &3);
        remove(&mut map, &3);

        destroy_empty(map);
    }

    #[test]
    public fun test_empty(){
        let map = new<u64, u64>();
        assert!(is_empty(&map), 0);
        assert!(length(&map) == 0, 1);
        assert!(!contains_key(&map, &3), 2);
        assert!(borrow_first_with_default(&map, &3, &0) == &0, 3);
        assert!(keys(&map) == vector[], 4);
        assert!(values(&map) == vector[], 5);
    }
}
