// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

module moveos_std::json{
    
    use std::string::String;
    use moveos_std::simple_map::SimpleMap;

    const ErrorTypeNotMatch: u64 = 1;
    const ErrorInvalidJSONString: u64 = 2;

    #[data_struct(T)]
    /// Function to deserialize a type T.
    /// The u128 and u256 types must be json String type instead of Number type
    public fun from_json<T>(json_str: vector<u8>): T {
        native_from_json(json_str)
    }

    /// Parse a json object string to a SimpleMap
    /// If the field type is primitive type, it will be parsed to String, otherwise it will abort.
    public fun to_map(json_str: vector<u8>): SimpleMap<String,String>{
        native_from_json<SimpleMap<String,String>>(json_str)
    }

    native fun native_from_json<T>(json_str: vector<u8>): T;

    #[test_only]
    use std::vector;
    #[test_only]
    use moveos_std::simple_map;
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
        ascii_string: std::ascii::String,
        utf8_string: std::string::String,
        age: u8,
        inner: Inner,
        bytes: vector<u8>, 
        inner_array: vector<Inner>,
        account: address,
    }

    #[test]
    fun test_from_json() {
        let json_str = b"{\"balance\": \"170141183460469231731687303715884105728\",\"ascii_string\":\"rooch.network\",\"utf8_string\":\"rooch.network\",\"age\":30,\"inner\":{\"value\":100},\"bytes\":[3,3,2,1],\"inner_array\":[{\"value\":101}],\"account\":\"0x42\"}";
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
        let json_str = b"{\"balance\": \"170141183460469231731687303715884105728\",\"string\":\"rooch.network\",\"age\":30,\"bool_value\": true, \"null_value\": null, \"account\":\"0x42\"}";
        let map = to_map(json_str);
        assert!(simple_map::borrow(&map, &string::utf8(b"balance")) == &string::utf8(b"170141183460469231731687303715884105728"), 1);
        assert!(simple_map::borrow(&map, &string::utf8(b"string")) == &string::utf8(b"rooch.network"), 2);
        assert!(simple_map::borrow(&map, &string::utf8(b"age")) == &string::utf8(b"30"), 4);
        assert!(simple_map::borrow(&map, &string::utf8(b"bool_value")) == &string::utf8(b"true"), 5);
        assert!(simple_map::borrow(&map, &string::utf8(b"null_value")) == &string::utf8(b"null"), 6);
        assert!(simple_map::borrow(&map, &string::utf8(b"account")) == &string::utf8(b"0x42"), 7);
    }
}