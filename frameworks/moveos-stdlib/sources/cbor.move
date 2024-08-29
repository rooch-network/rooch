// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

module moveos_std::cbor {
    use std::string::String;
    use std::option::{Self, Option};
    use moveos_std::simple_map::{Self, SimpleMap};

    /// Error if the CBOR bytes are invalid
    const ERROR_INVALID_CBOR_BYTES: u64 = 1;

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
            return simple_map::new()
        };

        option::destroy_some(opt_result)
    }

    /// Serialize a value of type T to CBOR bytes.
    public fun to_cbor<T>(value: &T): vector<u8> {
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
        bool_value: bool,
        age: u8,
        balance: u128,
        sig: u256,
        ascii_string: std::ascii::String,
        utf8_string: std::string::String,
        option_string: Option<std::string::String>,
        inner: Inner,
        null_value: Option<Inner>,
        inner_option: Option<Inner>,
        inner_array: vector<Inner>,
        account: address,
        bytes: vector<u8>,
    }

    #[test]
    fun test_to_cbor() {
        let test = Test { 
            bool_value: true,
            age: 30u8,
            balance: 170141183460469231731687303715884105728u128,
            sig: 1701411834604692317316873037158841057281687303715884105728u256,
            ascii_string: std::ascii::string(b"rooch.network"),
            utf8_string: std::string::utf8(b"rooch.network"),
            null_value: option::none(),
            option_string: option::some(std::string::utf8(b"rooch.network")),
            inner: Inner {
                value: 100u64,
            },
            inner_option: option::some(Inner {
                value: 102u64,
            }),
            inner_array: std::vector::singleton(Inner {
                value: 101u64,
            }),
            account: @0x42,
            bytes: vector<u8>[3u8, 2u8, 1u8, 0u8],
        };

        let cbor_bytes = to_cbor(&test);
        assert!(cbor_bytes == x"ad6a626f6f6c5f76616c7565f563616765181e6762616c616e6365c2508000000000000000000000000000000063736967c258184563918244f400000000000000000000176a81ca357800006c61736369695f737472696e676d726f6f63682e6e6574776f726b6b757466385f737472696e676d726f6f63682e6e6574776f726b6d6f7074696f6e5f737472696e676d726f6f63682e6e6574776f726b65696e6e6572a16576616c756518646a6e756c6c5f76616c7565f66c696e6e65725f6f7074696f6ea16576616c756518666b696e6e65725f617272617981a16576616c75651865676163636f756e74582000000000000000000000000000000000000000000000000000000000000000426562797465734403020100", 1);
    }

    #[test]
    fun test_from_cbor() {
        let cbor_bytes = x"ad6a626f6f6c5f76616c7565f563616765181e6762616c616e6365c2508000000000000000000000000000000063736967c258184563918244f400000000000000000000176a81ca357800006c61736369695f737472696e676d726f6f63682e6e6574776f726b6b757466385f737472696e676d726f6f63682e6e6574776f726b6d6f7074696f6e5f737472696e676d726f6f63682e6e6574776f726b65696e6e6572a16576616c756518646a6e756c6c5f76616c7565f66c696e6e65725f6f7074696f6ea16576616c756518666b696e6e65725f617272617981a16576616c75651865676163636f756e74582000000000000000000000000000000000000000000000000000000000000000426562797465734403020100";
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

        // check inner array
        assert!(vector::length(&obj.inner_array) == 1, 9);
        assert!(vector::borrow(&obj.inner_array, 0).value == 101u64, 10);

        // check account
        assert!(obj.account == @0x42, 11);

        // check ascii string
        assert!(obj.ascii_string == std::ascii::string(b"rooch.network"), 12);

        // check utf8 string
        assert!(obj.utf8_string == std::string::utf8(b"rooch.network"), 13);

        // check bool
        assert!(obj.bool_value, 14);

        // check null
        assert!(option::is_none<Inner>(&obj.null_value), 15);

        // check inner_option
        assert!(option::is_some(&obj.inner_option), 16);
        assert!(option::borrow(&obj.inner_option).value == 102u64, 17);

        // check u256
        assert!(obj.sig == 1701411834604692317316873037158841057281687303715884105728u256, 18);

        // check option string
        assert!(option::is_some(&obj.option_string), 19);
        assert!(option::borrow(&obj.option_string) == &std::string::utf8(b"rooch.network"), 20);
    }

    #[test]
    fun test_to_map(){
        let cbor_bytes = x"ad6a626f6f6c5f76616c7565f563616765181e6762616c616e6365c2508000000000000000000000000000000063736967c258184563918244f400000000000000000000176a81ca357800006c61736369695f737472696e676d726f6f63682e6e6574776f726b6b757466385f737472696e676d726f6f63682e6e6574776f726b6d6f7074696f6e5f737472696e676d726f6f63682e6e6574776f726b65696e6e6572a16576616c756518646a6e756c6c5f76616c7565f66c696e6e65725f6f7074696f6ea16576616c756518666b696e6e65725f617272617981a16576616c75651865676163636f756e74582000000000000000000000000000000000000000000000000000000000000000426562797465734403020100";
        let map = to_map(cbor_bytes);

        // check u128
        let balance_bytes = simple_map::borrow(&map, &std::string::utf8(b"balance"));
        let balance = from_cbor<u128>(*balance_bytes);
        assert!(balance == 170141183460469231731687303715884105728u128, 1);

        // check ascii string
        let ascii_string_bytes = simple_map::borrow(&map, &std::string::utf8(b"ascii_string"));
        let ascii_string = from_cbor<std::ascii::String>(*ascii_string_bytes);
        assert!(ascii_string == std::ascii::string(b"rooch.network"), 2);

        // check utf8 string
        let utf8_string_bytes = simple_map::borrow(&map, &std::string::utf8(b"utf8_string"));
        let utf8_string = from_cbor<std::string::String>(*utf8_string_bytes);
        assert!(utf8_string == std::string::utf8(b"rooch.network"), 3);

        // check u8
        let age_bytes = simple_map::borrow(&map, &std::string::utf8(b"age"));
        let age = from_cbor<u8>(*age_bytes);
        assert!(age == 30u8, 4);

        // check bool
        let bool_value_bytes = simple_map::borrow(&map, &std::string::utf8(b"bool_value"));
        let bool_value = from_cbor<bool>(*bool_value_bytes);
        assert!(bool_value, 5);

        // check null
        let null_value_bytes = simple_map::borrow(&map, &std::string::utf8(b"null_value"));
        let null_value = from_cbor<Option<Inner>>(*null_value_bytes);
        assert!(option::is_none<Inner>(&null_value), 6);

        // check address
        let account_bytes = simple_map::borrow(&map, &std::string::utf8(b"account"));
        let account = from_cbor<address>(*account_bytes);
        assert!(account == @0x42, 7);

        // check inner struct
        let inner_bytes = simple_map::borrow(&map, &std::string::utf8(b"inner"));
        let inner = from_cbor<Inner>(*inner_bytes);
        assert!(inner.value == 100u64, 8);

        // check bytes
        let bytes_cbor = simple_map::borrow(&map, &std::string::utf8(b"bytes"));
        let bytes = from_cbor<vector<u8>>(*bytes_cbor);
        assert!(vector::length(&bytes) == 4, 9);
        assert!(vector::borrow(&bytes, 0) == &3u8, 10);
        assert!(vector::borrow(&bytes, 1) == &2u8, 11);
        assert!(vector::borrow(&bytes, 2) == &1u8, 12);
        assert!(vector::borrow(&bytes, 3) == &0u8, 13);

        // check inner array
        let inner_array_bytes = simple_map::borrow(&map, &std::string::utf8(b"inner_array"));
        let inner_array = from_cbor<vector<Inner>>(*inner_array_bytes);
        assert!(vector::length(&inner_array) == 1, 14);
        assert!(vector::borrow(&inner_array, 0).value == 101u64, 15);

        // check u256
        let sig_bytes = simple_map::borrow(&map, &std::string::utf8(b"sig"));
        let sig = from_cbor<u256>(*sig_bytes);
        assert!(sig == 1701411834604692317316873037158841057281687303715884105728u256, 16);
 
        // check option string
        let option_string_bytes = simple_map::borrow(&map, &std::string::utf8(b"option_string"));
        //std::debug::print(option_string_bytes);
        let option_string = from_cbor<Option<std::string::String>>(*option_string_bytes);
        assert!(option::is_some(&option_string), 17);
        assert!(option::borrow(&option_string) == &std::string::utf8(b"rooch.network"), 18);

    }

    #[test]
    fun test_invalid_cbor_bytes_to_map(){
        let invalid_bytes = x"abcd";
        let map = to_map(invalid_bytes);
        assert!(simple_map::length(&map) == 0, 1);
    }

    #[test]
    fun test_invalid_cbor_bytes_from_cbor(){
        let invalid_bytes = x"abcd";
        let obj = from_cbor_option<Test>(invalid_bytes);
        assert!(option::is_none(&obj), 1);
    }

    #[test]
    fun test_struct_to_map_and_map_to_struct() {
        let test = Test { 
            bool_value: true,
            age: 30u8,
            balance: 170141183460469231731687303715884105728u128,
            sig: 1701411834604692317316873037158841057281687303715884105728u256,
            ascii_string: std::ascii::string(b"rooch.network"),
            utf8_string: std::string::utf8(b"rooch.network"),
            null_value: option::none(),
            option_string: option::some(std::string::utf8(b"rooch.network")),
            inner: Inner {
                value: 100u64,
            },
            inner_option: option::some(Inner {
                value: 102u64,
            }),
            inner_array: std::vector::singleton(Inner {
                value: 101u64,
            }),
            account: @0x42,
            bytes: vector<u8>[3u8, 2u8, 1u8, 0u8],
        };

        let cbor_bytes = to_cbor(&test);

        // cbor to map
        let test_map = to_map(cbor_bytes);
        // map to cbor
        let cbor2_bytes = to_cbor(&test_map);

        let obj = from_cbor<Test>(cbor2_bytes);
        assert!(obj.balance == 170141183460469231731687303715884105728u128, 1);
        assert!(obj.age == 30u8, 2);
        assert!(obj.inner.value == 100u64, 3);

        // check bytes
        assert!(vector::length(&obj.bytes) == 4, 4);
        assert!(vector::borrow(&obj.bytes, 0) == &3u8, 5);
        assert!(vector::borrow(&obj.bytes, 1) == &2u8, 6);
        assert!(vector::borrow(&obj.bytes, 2) == &1u8, 7);
        assert!(vector::borrow(&obj.bytes, 3) == &0u8, 8);

        // check inner array
        assert!(vector::length(&obj.inner_array) == 1, 9);
        assert!(vector::borrow(&obj.inner_array, 0).value == 101u64, 10);

        // check account
        assert!(obj.account == @0x42, 11);

        // check ascii string
        assert!(obj.ascii_string == std::ascii::string(b"rooch.network"), 12);

        // check utf8 string
        assert!(obj.utf8_string == std::string::utf8(b"rooch.network"), 13);

        // check bool
        assert!(obj.bool_value, 14);

        // check null
        assert!(option::is_none<Inner>(&obj.null_value), 15);

        // check inner_option
        assert!(option::is_some(&obj.inner_option), 16);
        assert!(option::borrow(&obj.inner_option).value == 102u64, 17);

        // check u256
        assert!(obj.sig == 1701411834604692317316873037158841057281687303715884105728u256, 18);

        // check option string
        assert!(option::is_some(&obj.option_string), 19);
        assert!(option::borrow(&obj.option_string) == &std::string::utf8(b"rooch.network"), 20);
    }
}
