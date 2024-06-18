// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

module moveos_std::string_utils {
    use std::vector;
    use std::option::{Self,Option};
    use std::string::{Self,String};

    const ErrorInvalidStringNumber: u64 = 1;

    public fun parse_u8_option(s: &String):Option<u8>{
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

    public fun parse_u8(s: &String): u8 {
        let result = parse_u8_option(s);
        assert!(option::is_some(&result), ErrorInvalidStringNumber);
        option::destroy_some(result)
    }

    public fun parse_u64_option(s: &String):Option<u64>{
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

    public fun parse_u64(s: &String): u64 {
        let result = parse_u64_option(s);
        assert!(option::is_some(&result), ErrorInvalidStringNumber);
        option::destroy_some(result)
    }

    public fun parse_u128_option(s: &String):Option<u128>{
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

    public fun parse_u128(s: &String): u128 {
        let result = parse_u128_option(s);
        assert!(option::is_some(&result), ErrorInvalidStringNumber);
        option::destroy_some(result)
    }

    public fun parse_u256_option(s: &String):Option<u256>{
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

    public fun parse_u256(s: &String): u256 {
        let result = parse_u256_option(s);
        assert!(option::is_some(&result), ErrorInvalidStringNumber);
        option::destroy_some(result)
    }

    //TODO introduce math module and replace this function
    fun pow10(n: u64): u256 {
        let i = 0;
        let result = 1u256;
        while (i < n) {
            result = result * 10;
            i = i + 1;
        };
        result
    }

    public fun parse_decimal_option(s: &String, decimal: u64): Option<u256> {
        let bytes = string::bytes(s);
        let i = 0;
        let result = 0u256;
        let decimal = (decimal as u256);
        let decimal_place = false;
        let decimal_count = 0;
        let remaining_count = decimal;
        while (i < vector::length(bytes)) {
            let c = *vector::borrow(bytes, i);
            if ((c >= 48) && (c <= 57)) {
                result = result * 10 + ((c - 48) as u256);
                if (decimal_place) {
                    decimal_count = decimal_count + 1;
                    if (decimal_count > decimal) {
                        return option::none()
                    };
                    remaining_count = remaining_count - 1;
                };
            } else if (c == 46) {
                decimal_place = true;
            } else {
                return option::none()
            };
            i = i + 1;
        };
        option::some(result * pow10((remaining_count as u64)))
    }

    public fun parse_decimal(s: &String, decimal: u64): u256 {
        let result = parse_decimal_option(s, decimal);
        assert!(option::is_some(&result), ErrorInvalidStringNumber);
        option::destroy_some(result)
    }

    public fun to_lower_case(s: &String): String {
        let bytes = string::bytes(s);
        let result = vector<u8>[];
        vector::for_each_ref(bytes, |c| {
            if (*c >= 65 && *c <= 90) {
                vector::push_back(&mut result, *c + 32);
            } else {
                vector::push_back(&mut result, *c);
            };
        });
        string::utf8(result)
    }

    public fun to_upper_case(s: &String): String {
        let bytes = string::bytes(s);
        let result = vector<u8>[];
        vector::for_each_ref(bytes, |c| {
            if (*c >= 97 && *c <= 122) {
                vector::push_back(&mut result, *c - 32);
            } else {
                vector::push_back(&mut result, *c);
            };
        });
        string::utf8(result)
    }

    public fun to_string_u256(n: u256): String {
        let result = vector::empty();
        if(n == 0) {
            vector::push_back(&mut result, 48);
        } else {
            while(n > 0) {
                vector::push_back(&mut result, ((n % 10) as u8) + 48);
                n = n / 10;
            };
        };
        vector::reverse(&mut result);
        string::utf8(result)
    }

    public fun to_string_u64(n: u64): String {
        to_string_u256((n as u256))
    }

     public fun to_string_u32(n: u32): String {
        to_string_u256((n as u256))
    }
    
    #[test]
    fun test_to_lower_case() {
        let s = string::utf8(b"ABc");
        let result = to_lower_case(&s);
        assert!(result == string::utf8(b"abc"), 1);
    }

    #[test]
    fun test_to_upper_case() {
        let s = string::utf8(b"Abc");
        let result = to_upper_case(&s);
        assert!(result == string::utf8(b"ABC"), 1);
    }

    #[test]
    fun test_parse_u8_option(){
        let s = string::utf8(b"123");
        let result = parse_u8_option(&s);
        assert!(result == option::some(123u8), 1);

        let s = string::utf8(b"abc");
        let result = parse_u8_option(&s);
        assert!(option::is_none(&result), 2);
    }

    #[test]
    fun test_parse_u8(){
        let s = string::utf8(b"123");
        assert!(parse_u8(&s) == 123u8, 1);
    }

    #[test]
    #[expected_failure(abort_code=1, location=moveos_std::string_utils)]
    fun test_parse_u8_failed(){
        let s = string::utf8(b"abc");
        parse_u8(&s);
    }

    #[test]
    #[expected_failure]
    fun test_parse_u8_overflow(){
        let s = string::utf8(b"256");
        parse_u8(&s);
    }

    #[test]
    fun test_u64_max(){
        let s = string::utf8(b"18446744073709551615");
        assert!(parse_u64(&s) == 18446744073709551615u64, 1);
    }

    #[test]
    fun test_u128_max(){
        let s = string::utf8(b"340282366920938463463374607431768211455");
        assert!(parse_u128(&s) == 340282366920938463463374607431768211455u128, 1);
    }

    #[test]
    fun test_u256_max(){
        let s = string::utf8(b"115792089237316195423570985008687907853269984665640564039457584007913129639935");
        assert!(parse_u256(&s) == 115792089237316195423570985008687907853269984665640564039457584007913129639935u256, 1);
    }

    #[test]
    fun test_parse_decimal_0() {
        let s = string::utf8(b"1");
        let result = parse_decimal(&s, 0);
        //std::debug::print(&result);
        assert!(result == 1u256, 1);
    }

    #[test]
    fun test_parse_decimal_10() {
        let s = string::utf8(b"1.1");
        let result = parse_decimal(&s, 10);
        //std::debug::print(&result);
        assert!(result == 11000000000u256, 1);
        let s = string::utf8(b"1");
        assert!(parse_decimal(&s, 10) == 10000000000u256, 2);
        let s = string::utf8(b"0.0000000001");
        assert!(parse_decimal(&s, 10) == 1u256, 3);
    }

    #[test]
    fun test_parse_decimal_18() {
        let s = string::utf8(b"1.1");
        assert!(parse_decimal(&s, 18) == 1100000000000000000u256, 1);

        let s = string::utf8(b"1");
        assert!(parse_decimal(&s, 18) == 1000000000000000000u256, 2);

        let s = string::utf8(b"0.000000000000000001");
        assert!(parse_decimal(&s, 18) == 1u256, 3);
    }

    #[test]
    #[expected_failure(abort_code=ErrorInvalidStringNumber, location=moveos_std::string_utils)]
    fun test_parse_decimal_failed_invalid_char(){
        let s = string::utf8(b"1a.1");
        let _result = parse_decimal(&s, 18);
    }

    #[test]
    #[expected_failure(abort_code=ErrorInvalidStringNumber, location=moveos_std::string_utils)]
    fun test_parse_decimal_failed_float_overflow(){
        let s = string::utf8(b"1.01");
        let _result = parse_decimal(&s, 1);
    }

    #[test]
    fun test_to_string(){
        let n = 123u256;
        let s = to_string_u256(n);
        assert!(s == string::utf8(b"123"), 1);
    }

    #[test]
    fun test_to_string_max(){
        let n = 115792089237316195423570985008687907853269984665640564039457584007913129639935u256;
        let s = to_string_u256(n);
        assert!(s == string::utf8(b"115792089237316195423570985008687907853269984665640564039457584007913129639935"), 1);
    }
}