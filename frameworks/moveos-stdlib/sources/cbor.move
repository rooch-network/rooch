// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

module moveos_std::cbor {
    use std::string::String;
    use std::option::{Self, Option};
    use moveos_std::simple_map::{Self, SimpleMap};

    /// Error if the `T` is not a struct
    const ERROR_TYPE_NOT_MATCH: u64 = 1;
    /// Error if the CBOR bytes are invalid
    const ERROR_INVALID_CBOR_BYTES: u64 = 2;

    #[data_struct(T)]
    /// Function to deserialize a type T from CBOR bytes.
    public fun from_cbor<T: drop>(bytes: vector<u8>): T {
        let opt_result = native_from_cbor(bytes);
        assert!(option::is_some(&opt_result), ERROR_INVALID_CBOR_BYTES);
        option::destroy_some(opt_result)
    }

    #[data_struct(T)]
    /// Function to deserialize a type T from CBOR bytes.
    /// If the CBOR bytes are invalid, it will return None.
    public fun from_cbor_option<T: drop>(bytes: vector<u8>): Option<T> {
        native_from_cbor(bytes)
    }

    /// Parse a cbor object bytes to a SimpleMap
    /// If the cbor bytes is invalid, it will return an empty SimpleMap
    /// If the field type is primitive type, it will be parsed to bytes, array or object will be parsed to cbor bytes
    public fun to_map(bytes: vector<u8>): SimpleMap<String,vector<u8>>{
        let opt_result = native_from_cbor<SimpleMap<String,vector<u8>>>(bytes);
        if(option::is_none(&opt_result)){
            option::destroy_none(opt_result);
            return simple_map::create()
        };

        option::destroy_some(opt_result)
    }

    #[data_struct(T)]
    /// Serialize a value of type T to CBOR bytes.
    public fun to_cbor<T: drop>(value: &T): vector<u8> {
        native_to_cbor(value)
    }

    native fun native_from_cbor<T>(bytes: vector<u8>): Option<T>;
    native fun native_to_cbor<T>(value: &T): vector<u8>;

    #[test_only]
    use std::vector;

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
        bool_value: bool,
        null_value: Option<Inner>,
        inner: Inner,
        inner_option: Option<Inner>,
        bytes: vector<u8>, 
        inner_array: vector<Inner>,
        account: address,
    }

    #[test]
    fun test_to_cbor() {
        let test = Test { 
            balance: 170141183460469231731687303715884105728u128,
            ascii_string: std::ascii::string(b"rooch.network"),
            utf8_string: std::string::utf8(b"rooch.network"),
            age: 30u8,
            bool_value: true,
            null_value: option::none(),
            inner: Inner {
                value: 100u64,
            },
            inner_option: option::some(Inner {
                value: 102u64,
            }),
            bytes: vector<u8>[3u8, 2u8, 1u8, 0u8],
            inner_array: std::vector::singleton(Inner {
                value: 101u64,
            }),
            account: @0x42,
        };

        let cbor_bytes = to_cbor(&test);
        std::debug::print(&cbor_bytes);
        assert!(cbor_bytes == x"ab6762616c616e6365c250800000000000000000000000000000006c61736369695f737472696e674d726f6f63682e6e6574776f726b6b757466385f737472696e674d726f6f63682e6e6574776f726b63616765181e6a626f6f6c5f76616c7565f56a6e756c6c5f76616c7565f665696e6e6572a16576616c756518646c696e6e65725f6f7074696f6ea16576616c7565186665627974657384030201006b696e6e65725f617272617981a16576616c75651865676163636f756e7458200000000000000000000000000000000000000000000000000000000000000042", 1);
    }

    #[test]
    fun test_from_cbor() {
        let cbor_bytes = x"ab6762616c616e6365c250800000000000000000000000000000006c61736369695f737472696e674d726f6f63682e6e6574776f726b6b757466385f737472696e674d726f6f63682e6e6574776f726b63616765181e6a626f6f6c5f76616c7565f56a6e756c6c5f76616c7565f665696e6e6572a16576616c756518646c696e6e65725f6f7074696f6ea16576616c7565186665627974657384030201006b696e6e65725f617272617981a16576616c75651865676163636f756e7458200000000000000000000000000000000000000000000000000000000000000042";
        let obj = from_cbor<Test>(cbor_bytes);
        assert!(obj.balance == 170141183460469231731687303715884105728u128, 1);
        assert!(obj.age == 30u8, 2);
        assert!(obj.inner.value == 100u64, 3);

        // check bytes
        assert!(vector::length(&obj.bytes) == 4, 4);
        assert!(vector::borrow(&obj.bytes, 0) == &3u8, 5);
        assert!(vector::borrow(&obj.bytes, 1) == &2u8, 6);
        assert!(vector::borrow(&obj.bytes, 2) == &1u8, 7);
        assert!(vector::borrow(&obj.bytes, 3) == &0u8, 8);

        // check inner_array
        assert!(vector::length(&obj.inner_array) == 1, 9);
        assert!(vector::borrow(&obj.inner_array, 0).value == 101u64, 10);

        // check account
        assert!(obj.account == @0x42, 11);

        // check ascii_string
        assert!(obj.ascii_string == std::ascii::string(b"rooch.network"), 12);

        // check utf8_string
        assert!(obj.utf8_string == std::string::utf8(b"rooch.network"), 13);

        // check bool
        assert!(obj.bool_value, 14);

        // check null
        assert!(option::is_none<Inner>(&obj.null_value), 15);

        // check inner_option
        assert!(option::is_some(&obj.inner_option), 16);
        assert!(option::borrow(&obj.inner_option).value == 102u64, 10);
    }

    /*
    #[test]
    fun test_to_map(){
        let cbor_bytes = x"ab6762616c616e6365c250800000000000000000000000000000006c61736369695f737472696e674d726f6f63682e6e6574776f726b6b757466385f737472696e674d726f6f63682e6e6574776f726b63616765181e6a626f6f6c5f76616c7565f56a6e756c6c5f76616c7565f665696e6e6572a16576616c756518646c696e6e65725f6f7074696f6ea16576616c7565186665627974657384030201006b696e6e65725f617272617981a16576616c75651865676163636f756e7458200000000000000000000000000000000000000000000000000000000000000042";
        let map = to_map(cbor_bytes);
        std::debug::print(&map);

        assert!(simple_map::borrow(&map, &string::utf8(b"balance")) == &b"170141183460469231731687303715884105728", 1);
        assert!(simple_map::borrow(&map, &string::utf8(b"ascii_string")) == &b"rooch.network", 2);
        assert!(simple_map::borrow(&map, &string::utf8(b"utf8_string")) == &b"rooch.network", 2);
        assert!(simple_map::borrow(&map, &string::utf8(b"age")) == &b"30", 4);
        assert!(simple_map::borrow(&map, &string::utf8(b"bool_value")) == &b"true", 5);
        assert!(simple_map::borrow(&map, &string::utf8(b"null_value")) == &b"null", 6);
        assert!(simple_map::borrow(&map, &string::utf8(b"account")) == &b"0x42", 7);
        //assert!(simple_map::borrow(&map, &string::utf8(b"inner")) == x"0x42"), 8);
        //assert!(simple_map::borrow(&map, &string::utf8(b"bytes")) == b"[3,3,2,1]", 9);
        //assert!(simple_map::borrow(&map, &string::utf8(b"inner_array")) == b"[{\"value\":101}]", 10);
        simple_map::drop(map);
    }
    */
}