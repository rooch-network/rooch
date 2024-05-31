// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

module moveos_std::json{
    
    use std::string::String;
    use std::option::{Self, Option};
    use moveos_std::simple_map::{Self, SimpleMap};

    /// Error if the `T` is not a struct
    const ErrorTypeNotMatch: u64 = 1;
    /// Error if the json string is invalid
    const ErrorInvalidJSONString: u64 = 2;

    #[data_struct(T)]
    /// Function to deserialize a type T.
    /// The u128 and u256 types must be json String type instead of Number type
    public fun from_json<T: copy >(json_str: vector<u8>): T {
        let opt_result = native_from_json(json_str);
        assert!(option::is_some(&opt_result), ErrorInvalidJSONString);
        option::destroy_some(opt_result)
    }

    #[data_struct(T)]
    /// Function to deserialize a type T.
    /// If the json string is invalid, it will return None
    public fun from_json_option<T: copy >(json_str: vector<u8>): Option<T> {
        native_from_json(json_str)
    }

    /// Parse a json object string to a SimpleMap
    /// If the json string is invalid, it will return an empty SimpleMap
    /// If the field type is primitive type, it will be parsed to String, array or object will be parsed to json string
    public fun to_map(json_str: vector<u8>): SimpleMap<String,String>{
        let opt_result = native_from_json<SimpleMap<String,String>>(json_str);
        if(option::is_none(&opt_result)){
            option::destroy_none(opt_result);
            return simple_map::create()
        };
        option::destroy_some(opt_result)
    }

    native fun native_from_json<T>(json_str: vector<u8>): Option<T>;

    #[test_only]
    use std::vector;
    #[test_only]
    use std::string;

    #[test_only]
    #[data_struct]
    struct Inner has copy, drop, store {
        value: u64,
    }
    #[test_only]
    #[data_struct]
    struct Test has copy, drop, store {
        balance: u128,
        utf8_string: std::string::String,
        age: u8,
        inner: Inner,
        bytes: vector<u8>, 
        inner_array: vector<Inner>,
        account: address,
    }

    #[test]
    fun test_from_json() {
        let json_str = b"{\"balance\": \"170141183460469231731687303715884105728\",\"utf8_string\":\"rooch.network\",\"age\":30,\"inner\":{\"value\":100},\"bytes\":[3,3,2,1],\"inner_array\":[{\"value\":101}],\"account\":\"0x42\"}";
        let obj = from_json<Test>(json_str);
        
        assert!(obj.balance == 170141183460469231731687303715884105728u128, 1);
        assert!(obj.age == 30u8, 2);
        assert!(obj.inner.value == 100u64, 3);

        // check bytes
        assert!(vector::length(&obj.bytes) == 4, 4);
        assert!(vector::borrow(&obj.bytes, 0) == &3u8, 5);
        assert!(vector::borrow(&obj.bytes, 1) == &3u8, 6);
        assert!(vector::borrow(&obj.bytes, 2) == &2u8, 7);
        assert!(vector::borrow(&obj.bytes, 3) == &1u8, 8);

        // check inner_array
        assert!(vector::length(&obj.inner_array) == 1, 9);
        assert!(vector::borrow(&obj.inner_array, 0).value == 101u64, 10);

        // check account
        assert!(obj.account == @0x42, 11);
    }

    #[test]
    fun test_to_map(){
        let json_str = b"{\"balance\": \"170141183460469231731687303715884105728\",\"string\":\"rooch.network\",\"age\":30,\"bool_value\": true, \"null_value\": null, \"account\":\"0x42\", \"inner\":{\"value\":100},\"bytes\":[3,3,2,1],\"inner_array\":[{\"value\":101}]}";
        let map = to_map(json_str);
        assert!(simple_map::borrow(&map, &string::utf8(b"balance")) == &string::utf8(b"170141183460469231731687303715884105728"), 1);
        assert!(simple_map::borrow(&map, &string::utf8(b"string")) == &string::utf8(b"rooch.network"), 2);
        assert!(simple_map::borrow(&map, &string::utf8(b"age")) == &string::utf8(b"30"), 4);
        assert!(simple_map::borrow(&map, &string::utf8(b"bool_value")) == &string::utf8(b"true"), 5);
        assert!(simple_map::borrow(&map, &string::utf8(b"null_value")) == &string::utf8(b"null"), 6);
        assert!(simple_map::borrow(&map, &string::utf8(b"account")) == &string::utf8(b"0x42"), 7);
        assert!(simple_map::borrow(&map, &string::utf8(b"inner")) == &string::utf8(b"{\"value\":100}"), 8);
        assert!(simple_map::borrow(&map, &string::utf8(b"bytes")) == &string::utf8(b"[3,3,2,1]"), 9);
        assert!(simple_map::borrow(&map, &string::utf8(b"inner_array")) == &string::utf8(b"[{\"value\":101}]"), 10);
        simple_map::drop(map);
    }

    #[test]
    fun test_invalid_json_to_map(){
        let invalid_json = b"abcd";
        let map = to_map(invalid_json);
        assert!(simple_map::length(&map) == 0, 1);
        simple_map::drop(map);
    }

    #[test]
    fun test_invalid_json_from_json(){
        let invalid_json = b"abcd";
        let obj = from_json_option<Test>(invalid_json);
        assert!(option::is_none(&obj), 1);
    }
}