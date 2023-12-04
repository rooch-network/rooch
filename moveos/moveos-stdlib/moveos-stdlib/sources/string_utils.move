// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

module moveos_std::string_utils {
    use std::vector;
    use std::option::{Self,Option};
    use std::string::{Self,String};

    const ErrorInvalidStringNumber: u64 = 1;

    public fun to_u8_option(s: &String):Option<u8>{
        let bytes:&vector<u8> = string::bytes(s);
        let i = 0;
        let result = 0u8;
        while (i < vector::length(bytes)) {
            let c = *vector::borrow(bytes, i);
            if (c >= 48 && c <= 57) {
                result = result * 10 + (c - 48);
            }else{
                return option::none()
            };
            i = i + 1;
        };
        option::some(result)
    }

    public fun to_u8(s: &String): u8 {
        let result = to_u8_option(s);
        assert!(option::is_some(&result), ErrorInvalidStringNumber);
        option::destroy_some(result)
    }

    public fun to_u64_option(s: &String):Option<u64>{
        let bytes:&vector<u8> = string::bytes(s);
        let i = 0;
        let result = 0u64;
        while (i < vector::length(bytes)) {
            let c = *vector::borrow(bytes, i);
            if (c >= 48 && c <= 57) {
                result = result * 10 + ((c - 48) as u64);
            }else{
                return option::none()
            };
            i = i + 1;
        };
        option::some(result)
    }

    public fun to_u64(s: &String): u64 {
        let result = to_u64_option(s);
        assert!(option::is_some(&result), ErrorInvalidStringNumber);
        option::destroy_some(result)
    }

    public fun to_u128_option(s: &String):Option<u128>{
        let bytes:&vector<u8> = string::bytes(s);
        let i = 0;
        let result = 0u128;
        while (i < vector::length(bytes)) {
            let c = *vector::borrow(bytes, i);
            if (c >= 48 && c <= 57) {
                result = result * 10 + ((c - 48) as u128);
            }else{
                return option::none()
            };
            i = i + 1;
        };
        option::some(result)
    }

    public fun to_u128(s: &String): u128 {
        let result = to_u128_option(s);
        assert!(option::is_some(&result), ErrorInvalidStringNumber);
        option::destroy_some(result)
    }

    public fun to_u256_option(s: &String):Option<u256>{
        let bytes:&vector<u8> = string::bytes(s);
        let i = 0;
        let result = 0u256;
        while (i < vector::length(bytes)) {
            let c = *vector::borrow(bytes, i);
            if (c >= 48 && c <= 57) {
                result = result * 10 + ((c - 48) as u256);
            }else{
                return option::none()
            };
            i = i + 1;
        };
        option::some(result)
    }

    public fun to_u256(s: &String): u256 {
        let result = to_u256_option(s);
        assert!(option::is_some(&result), ErrorInvalidStringNumber);
        option::destroy_some(result)
    }

    #[test]
    fun test_to_u8_option(){
        let s = string::utf8(b"123");
        let result = to_u8_option(&s);
        assert!(result == option::some(123u8), 1);

        let s = string::utf8(b"abc");
        let result = to_u8_option(&s);
        assert!(option::is_none(&result), 2);
    }

    #[test]
    fun test_to_u8(){
        let s = string::utf8(b"123");
        assert!(to_u8(&s) == 123u8, 1);
    }

    #[test]
    #[expected_failure(abort_code=1, location=moveos_std::string_utils)]
    fun test_to_u8_failed(){
        let s = string::utf8(b"abc");
        to_u8(&s);
    }

    #[test]
    #[expected_failure]
    fun test_to_u8_overflow(){
        let s = string::utf8(b"256");
        to_u8(&s);
    }

    #[test]
    fun test_u64_max(){
        let s = string::utf8(b"18446744073709551615");
        assert!(to_u64(&s) == 18446744073709551615u64, 1);
    }

    #[test]
    fun test_u128_max(){
        let s = string::utf8(b"340282366920938463463374607431768211455");
        assert!(to_u128(&s) == 340282366920938463463374607431768211455u128, 1);
    }

    #[test]
    fun test_u256_max(){
        let s = string::utf8(b"115792089237316195423570985008687907853269984665640564039457584007913129639935");
        assert!(to_u256(&s) == 115792089237316195423570985008687907853269984665640564039457584007913129639935u256, 1);
    }
}