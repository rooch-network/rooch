// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

module moveos_std::cbor {
    use std::option::{Self, Option};

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
    struct Test has copy, drop, store {
        value: u64,
        bytes: vector<u8>,
    }


    #[test]
    fun test_from_cbor() {
        let cbor_bytes = x"8218648403020100";
        let obj = from_cbor<Test>(cbor_bytes);
        assert!(obj.value == 100u64, 0);
        assert!(vector::length(&obj.bytes) == 4, 1);
        assert!(vector::borrow(&obj.bytes, 0) == &3u8, 2);
        assert!(vector::borrow(&obj.bytes, 1) == &2u8, 3);
        assert!(vector::borrow(&obj.bytes, 2) == &1u8, 4);
        assert!(vector::borrow(&obj.bytes, 3) == &0u8, 5);
    }


    #[test]
    fun test_to_cbor() {
        let test = Test { value: 100u64, bytes: vector<u8>[3u8, 2u8, 1u8, 0u8] };
        let cbor_bytes = to_cbor(&test);
        assert!(cbor_bytes == x"8218648403020100", 1);
    }
}