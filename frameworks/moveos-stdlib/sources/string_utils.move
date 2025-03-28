// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

module moveos_std::string_utils {
    use std::vector;
    use std::u256;
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

    /// Parse a string into a u16, returning an option
    public fun parse_u16_option(s: &String): Option<u16> {
        let bytes: &vector<u8> = string::bytes(s);
        let i = 0;
        let result = 0u16;
        while (i < vector::length(bytes)) {
            let c = *vector::borrow(bytes, i);
            if (c >= 48 && c <= 57) {
                result = result * 10 + ((c - 48) as u16);
            } else {
                return option::none()
            };
            i = i + 1;
        };
        option::some(result)
    }

    /// Parse a string into a u16, aborting if the string is not a valid number
    public fun parse_u16(s: &String): u16 {
        let result = parse_u16_option(s);
        assert!(option::is_some(&result), ErrorInvalidStringNumber);
        option::destroy_some(result)
    }

    /// Parse a string into a u32, returning an option
    public fun parse_u32_option(s: &String): Option<u32> {
        let bytes: &vector<u8> = string::bytes(s);
        let i = 0;
        let result = 0u32;
        while (i < vector::length(bytes)) {
            let c = *vector::borrow(bytes, i);
            if (c >= 48 && c <= 57) {
                result = result * 10 + ((c - 48) as u32);
            } else {
                return option::none()
            };
            i = i + 1;
        };
        option::some(result)
    }

    /// Parse a string into a u32, aborting if the string is not a valid number
    public fun parse_u32(s: &String): u32 {
        let result = parse_u32_option(s);
        assert!(option::is_some(&result), ErrorInvalidStringNumber);
        option::destroy_some(result)
    }

    public fun parse_decimal_option(s: &String, decimal: u8): Option<u256> {
        let bytes = string::bytes(s);
        let i = 0;
        let result = 0u256;
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
        option::some(result * u256::pow(10, remaining_count))
    }

    public fun parse_decimal(s: &String, decimal: u8): u256 {
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

    public fun to_string_u128(n: u128): String {
        to_string_u256((n as u256))
    }

    public fun to_string_u64(n: u64): String {
        to_string_u256((n as u256))
    }

    public fun to_string_u32(n: u32): String {
        to_string_u256((n as u256))
    }

    public fun to_string_u16(n: u16): String {
        to_string_u256((n as u256))
    }

    public fun to_string_u8(n: u8): String {
        to_string_u256((n as u256))
    }

    public fun starts_with(haystack_str: &String, needle: &String): bool {
        if (string::length(needle) > string::length(haystack_str)) {
            return false
        };
        let sub = string::sub_string(haystack_str, 0, string::length(needle));
        sub == *needle
    }

    public fun contains(s: &String, sub: &String): bool {
        if (string::length(sub) == 0) {
            return true
        };
        string::index_of(s, sub) != string::length(s)
    }

    /// Split a string by a delimiter
    public fun split(s: &String, delimiter: &String): vector<String> {
        let result = vector::empty<String>();
        let start = 0;
        let len = string::length(s);
        
        while (start <= len) {
            let pos = if (start == len) {
                len
            } else {
                let sub = string::sub_string(s, start, len);
                let idx = string::index_of(&sub, delimiter);
                if (idx == string::length(&sub)) {
                    len
                } else {
                    start + idx
                }
            };
            
            if (pos >= start) {
                let part = string::sub_string(s, start, pos);
                vector::push_back(&mut result, part);
            };
            
            if (pos == len) break;
            start = pos + string::length(delimiter);
        };
        result
    }

    /// Trim leading and trailing whitespace from a string
    public fun trim(s: &String): String {
        let bytes = string::bytes(s);
        let len = vector::length(bytes);
        let start = find_first_non_space(bytes, 0, len);
        let end = find_last_non_space(bytes, 0, len);
        if (start >= end) {
            return string::utf8(b"")
        };
        let result = vector::slice(bytes, start, end + 1);
        string::utf8(result)
    }

    const SPACE_CHAR :u8 = 32u8;

    fun find_first_non_space(bytes: &vector<u8>, start: u64, end: u64): u64 {
        let i = start;
        while (i < end) {
            if (*vector::borrow(bytes, i) != SPACE_CHAR) {
                return i
            };
            i = i + 1;
        };
        end
    }
    

    fun find_last_non_space(bytes: &vector<u8>, start: u64, end: u64): u64 {
        let i = end;
        while (i > start) {
            if (*vector::borrow(bytes, i - 1) != SPACE_CHAR) {
                return i - 1
            };
            i = i - 1;
        };
        start
    }

    /// Strip a prefix from a string
    public fun strip_prefix(s: String, prefix: &String): String {
        if (string::length(prefix) > string::length(&s)) {
            return s
        };
        if (starts_with(&s, prefix)) {
            string::sub_string(&s, string::length(prefix), string::length(&s))
        } else {
            s
        }
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

    #[test]
    fun test_trim() {
        let s = string::utf8(b"  hello, world  ");
        let trimmed = trim(&s);
        assert!(trimmed == string::utf8(b"hello, world"), 1);

        let s2 = string::utf8(b"  ");
        let trimmed2 = trim(&s2);
        assert!(trimmed2 == string::utf8(b""), 2);

        let s3 = string::utf8(b"");
        let trimmed3 = trim(&s3);
        assert!(trimmed3 == string::utf8(b""), 3);
    }

    #[test]
    fun test_split() {
        let s = string::utf8(b"hello,world,test");
        let parts = split(&s, &string::utf8(b","));
        assert!(vector::length(&parts) == 3, 1);
        assert!(*vector::borrow(&parts, 0) == string::utf8(b"hello"), 2);
        assert!(*vector::borrow(&parts, 1) == string::utf8(b"world"), 3);
        assert!(*vector::borrow(&parts, 2) == string::utf8(b"test"), 4);

        // Test empty parts
        let s2 = string::utf8(b"a,,b");
        let parts2 = split(&s2, &string::utf8(b","));
        assert!(vector::length(&parts2) == 3, 5);
        assert!(*vector::borrow(&parts2, 0) == string::utf8(b"a"), 6);
        assert!(*vector::borrow(&parts2, 1) == string::utf8(b""), 7);
        assert!(*vector::borrow(&parts2, 2) == string::utf8(b"b"), 8);
    }

    #[test]
    fun test_contains() {
        let s = string::utf8(b"hello world");
        assert!(contains(&s, &string::utf8(b"hello")), 1);
        assert!(contains(&s, &string::utf8(b"world")), 2);
        assert!(contains(&s, &string::utf8(b"o w")), 3);
        assert!(contains(&s, &string::utf8(b"")), 4);
        assert!(!contains(&s, &string::utf8(b"world!")), 5);
        assert!(!contains(&s, &string::utf8(b"hello!")), 6);
        
        let empty = string::utf8(b"");
        assert!(contains(&empty, &string::utf8(b"")), 7);
        assert!(!contains(&empty, &string::utf8(b"a")), 8);
    }

    #[test]
    fun test_strip_prefix() {
        let s = string::utf8(b"hello world");
        assert!(strip_prefix(s, &string::utf8(b"hello ")) == string::utf8(b"world"), 1);
        assert!(strip_prefix(string::utf8(b"hello"), &string::utf8(b"he")) == string::utf8(b"llo"), 2);
        assert!(strip_prefix(string::utf8(b"hello"), &string::utf8(b"hello")) == string::utf8(b""), 3);
        // No match cases - should return original string
        assert!(strip_prefix(string::utf8(b"hello"), &string::utf8(b"world")) == string::utf8(b"hello"), 4);
        assert!(strip_prefix(string::utf8(b"hello"), &string::utf8(b"hello!")) == string::utf8(b"hello"), 5);
        
        let empty = string::utf8(b"");
        assert!(strip_prefix(empty, &string::utf8(b"")) == string::utf8(b""), 6);
        assert!(strip_prefix(empty, &string::utf8(b"a")) == string::utf8(b""), 7);
    }

    #[test]
    fun test_find_first_non_space() {
        let bytes = &b"  hello";
        assert!(find_first_non_space(bytes, 0, 7) == 2, 1);
        
        let all_spaces = &b"   ";
        assert!(find_first_non_space(all_spaces, 0, 3) == 3, 2);
        
        let empty = &b"";
        assert!(find_first_non_space(empty, 0, 0) == 0, 3);
        
        let no_spaces = &b"hello";
        assert!(find_first_non_space(no_spaces, 0, 5) == 0, 4);
    }

    #[test]
    fun test_find_last_non_space() {
        let bytes = &b"hello  ";
        assert!(find_last_non_space(bytes, 0, 7) == 4, 1);
        
        let all_spaces = &b"   ";
        assert!(find_last_non_space(all_spaces, 0, 3) == 0, 2);
        
        let empty = &b"";
        assert!(find_last_non_space(empty, 0, 0) == 0, 3);
        
        let no_spaces = &b"hello";
        assert!(find_last_non_space(no_spaces, 0, 5) == 4, 4);
    }

    #[test]
    fun test_utf8_contains() {
        // Test with two valid UTF-8 characters
        let s = string::utf8(x"E38182E38183"); // Two Hiragana characters
        // Test with first character
        assert!(contains(&s, &string::utf8(x"E38182")), 1);
        // Test with second character
        assert!(contains(&s, &string::utf8(x"E38183")), 2);
        // Test with both characters
        assert!(contains(&s, &string::utf8(x"E38182E38183")), 3);
        // Test with non-existing character
        assert!(!contains(&s, &string::utf8(x"E38184")), 4);
        // Test with empty string
        assert!(contains(&s, &string::utf8(b"")), 5);
    }

    #[test]
    fun test_utf8_starts_with() {
        // Test with two valid UTF-8 characters
        let s = string::utf8(x"E38182E38183"); // Two Hiragana characters
        // Test with first character
        assert!(starts_with(&s, &string::utf8(x"E38182")), 1);
        // Test with both characters
        assert!(starts_with(&s, &string::utf8(x"E38182E38183")), 2);
        // Test with second character (should fail)
        assert!(!starts_with(&s, &string::utf8(x"E38183")), 3);
        // Test with non-existing character
        assert!(!starts_with(&s, &string::utf8(x"E38184")), 4);
        // Test with empty string
        assert!(starts_with(&s, &string::utf8(b"")), 5);
    }

    #[test]
    fun test_utf8_split() {
        // Test with valid UTF-8 characters
        let s = string::utf8(x"E381822CE381832CE38184"); // Three Hiragana characters separated by commas
        let parts = split(&s, &string::utf8(b","));
        assert!(vector::length(&parts) == 3, 1);
        assert!(*vector::borrow(&parts, 0) == string::utf8(x"E38182"), 2);
        assert!(*vector::borrow(&parts, 1) == string::utf8(x"E38183"), 3);
        assert!(*vector::borrow(&parts, 2) == string::utf8(x"E38184"), 4);
    }

    #[test]
    fun test_parse_u16_option() {
        let s = string::utf8(b"12345");
        let result = parse_u16_option(&s);
        assert!(result == option::some(12345u16), 1);
        
        let s = string::utf8(b"abc");
        let result = parse_u16_option(&s);
        assert!(option::is_none(&result), 2);
    }

    #[test]
    fun test_parse_u16() {
        let s = string::utf8(b"12345");
        assert!(parse_u16(&s) == 12345u16, 1);
    }

    #[test]
    #[expected_failure(abort_code=ErrorInvalidStringNumber, location=moveos_std::string_utils)]
    fun test_parse_u16_failed() {
        let s = string::utf8(b"abc");
        parse_u16(&s);
    }

    #[test]
    #[expected_failure]
    fun test_parse_u16_overflow() {
        let s = string::utf8(b"65536"); // Max u16 is 65535
        parse_u16(&s);
    }

    #[test]
    fun test_parse_u32_option() {
        let s = string::utf8(b"123456789");
        let result = parse_u32_option(&s);
        assert!(result == option::some(123456789u32), 1);
        
        let s = string::utf8(b"abc");
        let result = parse_u32_option(&s);
        assert!(option::is_none(&result), 2);
    }

    #[test]
    fun test_parse_u32() {
        let s = string::utf8(b"123456789");
        assert!(parse_u32(&s) == 123456789u32, 1);
    }

    #[test]
    #[expected_failure(abort_code=ErrorInvalidStringNumber, location=moveos_std::string_utils)]
    fun test_parse_u32_failed() {
        let s = string::utf8(b"abc");
        parse_u32(&s);
    }

    #[test]
    fun test_u16_max() {
        let s = string::utf8(b"65535");
        assert!(parse_u16(&s) == 65535u16, 1);
    }

    #[test]
    fun test_u32_max() {
        let s = string::utf8(b"4294967295");
        assert!(parse_u32(&s) == 4294967295u32, 1);
    }

    #[test]
    #[expected_failure]
    fun test_parse_u32_overflow() {
        let s = string::utf8(b"4294967296"); // Max u32 is 4294967295
        parse_u32(&s);
    }
}