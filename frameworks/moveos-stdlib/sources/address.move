// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

// Copyright (c) Mysten Labs, Inc.
// SPDX-License-Identifier: Apache-2.0

// Source https://github.com/MystenLabs/sui/blob/598f106ef5fbdfbe1b644236f0caf46c94f4d1b7/crates/sui-framework/sources/address.move

module moveos_std::address {
    use std::vector;
    use std::ascii;
    use std::string;
    use std::option::{Self, Option};
    use moveos_std::bech32;
    use moveos_std::bcs;
    use moveos_std::hex;
    use moveos_std::string_utils;

    /// The length of an address, in bytes
    const LENGTH: u64 = 32;

    /// HRP for Rooch addresses
    const ROOCH_HRP: vector<u8> = b"rooch";

    // The largest integer that can be represented with 32 bytes
    const MAX: u256 = 115792089237316195423570985008687907853269984665640564039457584007913129639935;

    const ZERO: address = @0x0000000000000000000000000000000000000000000000000000000000000000;

    /// Error from `from_bytes` when it is supplied too many or too few bytes.
    const ErrorAddressParseError: u64 = 1;

    /// Error from `from_u256` when
    const ErrorU256TooBigToConvertToAddress: u64 = 2;

    //TODO
    /// Convert `a` into a u256 by interpreting `a` as the bytes of a big-endian integer
    /// (e.g., `to_u256(0x1) == 1`)
    //native public fun to_u256(a: address): u256;

    //TODO
    /// Convert `n` into an address by encoding it as a big-endian integer (e.g., `from_u256(1) = @0x1`)
    /// Aborts if `n` > `MAX_ADDRESS`
    //native public fun from_u256(n: u256): address;

    /// Convert `bytes` into an address.
    /// Aborts with `ErrorAddressParseError` if the length of `bytes` is invalid length or if the bytes are not a valid address
    public fun from_bytes(bytes: vector<u8>): address {
        bcs::from_bytes<address>(bytes)
    }

    /// Convert `bytes` into an address.
    /// Returns `None` if the length of `bytes` is invalid length or if the bytes are not a valid address
    public fun from_bytes_option(bytes: vector<u8>): Option<address> {
        bcs::from_bytes_option<address>(bytes)
    }

    /// Convert `a` into BCS-encoded bytes.
    public fun to_bytes(a: &address): vector<u8> {
        bcs::to_bytes(a)
    }

    /// Convert `a` to a hex-encoded ASCII string
    public fun to_ascii_string(a: &address): ascii::String {
        ascii::string(hex::encode(to_bytes(a)))
    }

    /// Convert `a` to a hex-encoded utf8 string
    public fun to_string(a: &address): string::String {
        string::utf8(hex::encode(to_bytes(a)))
    }

    /// Converts an ASCII string to an address, taking the numerical value for each character. The
    /// string must be Base16 encoded, and thus exactly 64 characters long.
    /// For example, the string "00000000000000000000000000000000000000000000000000000000DEADB33F"
    /// will be converted to the address @0xDEADB33F.
    /// Aborts with `ErrorAddressParseError` if the length of `s` is not 64,
    /// or if an invalid character is encountered.
    public fun from_ascii_bytes(bytes: &vector<u8>): address {
        let address_opt = from_ascii_bytes_option(bytes);
        assert!(option::is_some(&address_opt), ErrorAddressParseError);
        option::destroy_some(address_opt)
    }

    public fun from_ascii_bytes_option(bytes: &vector<u8>): Option<address> {
        if (vector::length(bytes) != 64) {
            return option::none()
        };
        let opt_bytes = hex::decode_option(bytes);
        if (option::is_none(&opt_bytes)) {
            return option::none()
        };
        let hex_bytes = option::destroy_some(opt_bytes);
        option::some(from_bytes(hex_bytes))
    }

    /// Convert `a` from hex ASCII string
    public fun from_ascii_string(a: ascii::String): Option<address> {
        let opt_bytes = hex::decode_option(&ascii::into_bytes(a));
        if (option::is_none(&opt_bytes)) {
            return option::none()
        };

        let bytes = option::destroy_some(opt_bytes);
        bcs::from_bytes_option<address>(bytes)
    }

    /// Convert `a` to a bech32 string
    public fun to_bech32_string(addr: address): string::String {
        let addr_bytes = bcs::to_bytes(&addr);
        let result = bech32::encode(bech32::bech32m_to_bip(), ROOCH_HRP, addr_bytes);
        string::utf8(result)
    }

    /// Convert a bech32 string to `address`
    public fun from_bech32_string(str: &string::String): address {
        let encoded_bytes = string::bytes(str);
        assert!(!vector::is_empty(encoded_bytes), ErrorAddressParseError);

        let decode_data = bech32::decode(ROOCH_HRP, *encoded_bytes);
        moveos_std::bcs::to_address(decode_data)
    }

    /// Convert a string to address, supporting both hex and bech32 formats
    /// For hex format: supports "0x1", "1", "0x0000...0001" (with or without 0x prefix)
    /// For bech32 format: "rooch1..." format
    /// Returns None if the string cannot be parsed as a valid address
    public fun from_string_option(str: &string::String): Option<address> {
        // First try bech32 format (starts with "rooch1")
        if (string_utils::starts_with_bytes(str, &ROOCH_HRP)) {
            let bech32_result = try_parse_bech32(str);
            if (option::is_some(&bech32_result)) {
                return bech32_result
            }
        };

        // Try hex format
        try_parse_hex_address(str)
    }

    /// Convert a string to address, supporting both hex and bech32 formats
    /// Aborts if the string cannot be parsed as a valid address
    public fun from_string(str: &string::String): address {
        let addr_opt = from_string_option(str);
        assert!(option::is_some(&addr_opt), ErrorAddressParseError);
        option::destroy_some(addr_opt)
    }

    /// Try to parse bech32 address format
    fun try_parse_bech32(str: &string::String): Option<address> {
        // For bech32, we need to handle potential aborts from bech32::decode
        // Since Move doesn't have try-catch, we'll do validation first

        // Rooch bech32 addresses are exactly 64 characters long
        // Format: "rooch" (5) + "1" (1) + data+checksum (58) = 64 characters
        if (string::length(str) != 64) {
            return option::none()
        };

        // For a more robust implementation, we should provide a bech32::decode_option function
        // For now, we'll assume if it starts correctly, we can try to decode
        // In practice, bech32::decode might still abort on invalid checksums
        option::some(from_bech32_string(str))
    }

    /// Try to parse hex address format (with or without 0x prefix, supports padding)
    fun try_parse_hex_address(str: &string::String): Option<address> {
        // Remove 0x prefix if present
        let cleaned_str = strip_hex_prefix(*str);
        
        // Get the address bytes after cleaning
        let addr_bytes = string::bytes(&cleaned_str);
        let addr_len = vector::length(addr_bytes);
        
        // Empty string should return None
        if (addr_len == 0) {
            return option::none()
        };
        
        // Check if the hex string is too long (more than 64 characters)
        if (addr_len > 64) {
            return option::none()
        };
        
        // Addresses must be padded to 64 hex characters for proper parsing
        let padded_bytes = vector::empty<u8>();
        let padding_needed = 64 - addr_len;
        let i = 0;
        while (i < padding_needed) {
            vector::push_back(&mut padded_bytes, 48); // '0'
            i = i + 1;
        };
        
        // Copy address bytes to padded_bytes
        i = 0;
        while (i < addr_len) {
            vector::push_back(&mut padded_bytes, *vector::borrow(addr_bytes, i));
            i = i + 1;
        };
        
        let opt_bytes = hex::decode_option(&padded_bytes);
        if (option::is_none(&opt_bytes)) {
            return option::none()
        };

        let bytes = option::destroy_some(opt_bytes);
        bcs::from_bytes_option<address>(bytes)
    }

    /// Strip 0x or 0X prefix from string
    fun strip_hex_prefix(str: string::String): string::String {
        let bytes = string::bytes(&str);
        let len = vector::length(bytes);
        
        if (len >= 2 && 
            *vector::borrow(bytes, 0) == 48 && // '0'
            (*vector::borrow(bytes, 1) == 120 || *vector::borrow(bytes, 1) == 88)) { // 'x' or 'X'
            // Remove first 2 characters
            let result_bytes = vector::empty<u8>();
            let i = 2;
            while (i < len) {
                vector::push_back(&mut result_bytes, *vector::borrow(bytes, i));
                i = i + 1;
            };
            string::utf8(result_bytes)
        } else {
            str
        }
    }

    /// Length of a Rooch address in bytes
    public fun length(): u64 {
        LENGTH
    }

    /// Largest possible address
    public fun max(): u256 {
        MAX
    }

    /// all zeros address
    public fun zero(): address {
        ZERO
    }

    #[test]
    fun test_from_ascii_bytes() {
        let ascii_bytes = b"1f1e1d1c1b1a191817161514131211100f0e0d0c0b0a09080706050403020100";
        let addr = from_ascii_bytes(&ascii_bytes);
        assert!(addr == @0x1f1e1d1c1b1a191817161514131211100f0e0d0c0b0a09080706050403020100, 1);
        let addr_str = to_ascii_string(&addr);
        assert!(ascii::into_bytes(addr_str) == ascii_bytes, 2);
    }

    #[test]
    fun test_to_string() {
        let ascii_bytes = b"1f1e1d1c1b1a191817161514131211100f0e0d0c0b0a09080706050403020100";
        let expected_address = @0x1f1e1d1c1b1a191817161514131211100f0e0d0c0b0a09080706050403020100;
        let result = to_string(&expected_address);
        assert!(string::into_bytes(result) == ascii_bytes, 1);
    }

    #[test]
    fun test_from_ascii_bytes_valid() {
        let valid_hex_bytes = b"1f1e1d1c1b1a191817161514131211100f0e0d0c0b0a09080706050403020100";
        let expected_address = @0x1f1e1d1c1b1a191817161514131211100f0e0d0c0b0a09080706050403020100;
        let result = from_ascii_bytes(&valid_hex_bytes);
        assert!(result == expected_address, 1);
    }

    #[test]
    #[expected_failure(abort_code = ErrorAddressParseError, location = moveos_std::address)]
    fun test_from_ascii_bytes_invalid_characters() {
        let invalid_hex_bytes = b"0123456789ABCDEFG";
        from_ascii_bytes(&invalid_hex_bytes);
    }

    #[test]
    #[expected_failure(abort_code = ErrorAddressParseError, location = moveos_std::address)]
    fun test_from_ascii_bytes_empty_string() {
        let empty_hex_bytes = b"";
        let _result = from_ascii_bytes(&empty_hex_bytes);
    }

    #[test]
    #[expected_failure(abort_code = ErrorAddressParseError, location = moveos_std::address)]
    fun test_from_ascii_bytes_too_short() {
        let short_hex_bytes = b"0123456789ab";
        let _result = from_ascii_bytes(&short_hex_bytes);
    }

    #[test]
    #[expected_failure(abort_code = ErrorAddressParseError, location = moveos_std::address)]
    fun test_from_ascii_bytes_too_long() {
        let long_hex_bytes = b"000102030405060708090a0b0c0d0e0f101112131415161718191a1b1c1d1e1fff";
        let _result = from_ascii_bytes(&long_hex_bytes);
    }

    #[test]
    #[expected_failure(abort_code = ErrorAddressParseError, location = moveos_std::address)]
    fun test_from_ascii_bytes_with_non_data_characters() {
        let hex_bytes_with_spaces = b"01 23 45 67 89 AB CD EF";
        let _result = from_ascii_bytes(&hex_bytes_with_spaces);
    }

    #[test]
    fun test_from_ascii_bytes_mixed_case() {
        let valid_hex_bytes = b"1F1E1d1c1b1a191817161514131211100f0e0D0c0b0a09080706050403020100";
        let expected_address = @0x1f1e1d1c1b1a191817161514131211100f0e0D0c0b0a09080706050403020100;
        let result = from_ascii_bytes(&valid_hex_bytes);
        assert!(result == expected_address, 1);
    }

    #[test]
    fun test_bech32_string() {
        let addr = @0x42;
        let addr_str = to_bech32_string(addr);
        assert!(addr_str == string::utf8(b"rooch1qqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqppq6exstd"), 1001);
        let addr_from = from_bech32_string(&addr_str);
        assert!(addr == addr_from, 1002);

        let addr2 = @0xa7afe75c4f3a7631191905601f4396b25dde044539807de65ed4fc7358dbd98e;
        let addr_str2 = to_bech32_string(addr2);
        assert!(addr_str2 == string::utf8(b"rooch157h7whz08fmrzxgeq4sp7sukkfwaupz98xq8mej76n78xkxmmx8q9ujmg6"), 1003);
        let addr_from2 = from_bech32_string(&addr_str2);
        assert!(addr2 == addr_from2, 1004)
    }

    #[test]
    fun test_ascii_string() {
        let addr = @0x42;
        let addr_str = to_ascii_string(&addr);
        let addr_from_opt = from_ascii_string(addr_str);
        let addr_from = option::extract(&mut addr_from_opt);
        assert!(addr == addr_from, 1001);

        let addr2 = @0xa7afe75c4f3a7631191905601f4396b25dde044539807de65ed4fc7358dbd98e;
        let addr_str2 = to_ascii_string(&addr2);
        let addr_from_opt2 = from_ascii_string(addr_str2);
        let addr_from2 = option::extract(&mut addr_from_opt2);
        assert!(addr2 == addr_from2, 1001);
    }

    #[test]
    fun test_from_string_hex() {
        // Test with 0x prefix
        let addr_str = string::utf8(b"0x1");
        let addr = from_string(&addr_str);
        assert!(addr == @0x1, 3001);
        
        // Test without 0x prefix
        let addr_str2 = string::utf8(b"42");
        let addr2 = from_string(&addr_str2);
        assert!(addr2 == @0x42, 3002);

        // Test with full hex address
        let addr_str3 = string::utf8(b"0x0000000000000000000000000000000000000000000000000000000000000001");
        let addr3 = from_string(&addr_str3);
        assert!(addr3 == @0x1, 3003);

        // Test with uppercase hex
        let addr_str4 = string::utf8(b"0xABCDEF");
        let addr4 = from_string(&addr_str4);
        assert!(addr4 == @0xabcdef, 3004);

        // Test with mixed case
        let addr_str5 = string::utf8(b"0xAbCdEf123");
        let addr5 = from_string(&addr_str5);
        assert!(addr5 == @0xabcdef123, 3005);

        // Test zero address
        let addr_str6 = string::utf8(b"0x0");
        let addr6 = from_string(&addr_str6);
        assert!(addr6 == @0x0, 3006);
    }

    #[test]
    fun test_from_string_bech32() {
        let addr = @0x42;
        let bech32_str = to_bech32_string(addr);
        let addr_from = from_string(&bech32_str);
        assert!(addr == addr_from, 3010);

        let addr2 = @0xa7afe75c4f3a7631191905601f4396b25dde044539807de65ed4fc7358dbd98e;
        let bech32_str2 = to_bech32_string(addr2);
        let addr_from2 = from_string(&bech32_str2);
        assert!(addr2 == addr_from2, 3011);

        // Test zero address in bech32
        let zero_addr = @0x0;
        let zero_bech32 = to_bech32_string(zero_addr);
        let zero_from = from_string(&zero_bech32);
        assert!(zero_addr == zero_from, 3012);
    }

    #[test]
    fun test_from_string_option_valid() {
        // Test valid hex addresses
        let hex_addr = string::utf8(b"0x123");
        let result = from_string_option(&hex_addr);
        assert!(option::is_some(&result), 3020);
        assert!(option::destroy_some(result) == @0x123, 3021);

        // Test valid bech32 addresses
        let addr = @0x42;
        let bech32_str = to_bech32_string(addr);
        let result2 = from_string_option(&bech32_str);
        assert!(option::is_some(&result2), 3022);
        assert!(option::destroy_some(result2) == @0x42, 3023);
    }

    #[test]
    fun test_from_string_option_invalid() {
        // Test invalid hex
        let invalid_hex = string::utf8(b"0xGGG");
        let result = from_string_option(&invalid_hex);
        assert!(option::is_none(&result), 3030);

        // Test invalid hex with non-hex characters
        let invalid_hex2 = string::utf8(b"0x123Z");
        let result2 = from_string_option(&invalid_hex2);
        assert!(option::is_none(&result2), 3031);

        // Test empty string
        let empty_str = string::utf8(b"");
        let result3 = from_string_option(&empty_str);
        assert!(option::is_none(&result3), 3032);

        // Test string that's not hex or bech32
        let invalid_str = string::utf8(b"not_an_address");
        let result4 = from_string_option(&invalid_str);
        assert!(option::is_none(&result4), 3033);

        // Test string with spaces
        let spaced_str = string::utf8(b"0x 123");
        let result5 = from_string_option(&spaced_str);
        assert!(option::is_none(&result5), 3034);
    }

    #[test]
    #[expected_failure(abort_code = ErrorAddressParseError, location = moveos_std::address)]
    fun test_from_string_invalid_hex_abort() {
        let invalid_hex = string::utf8(b"0xGGG");
        from_string(&invalid_hex);
    }

    #[test]
    #[expected_failure(abort_code = ErrorAddressParseError, location = moveos_std::address)]
    fun test_from_string_empty_abort() {
        let empty_str = string::utf8(b"");
        from_string(&empty_str);
    }

    #[test]
    fun test_strip_hex_prefix() {
        let hex_with_prefix = string::utf8(b"0x42");
        let stripped = strip_hex_prefix(hex_with_prefix);
        assert!(stripped == string::utf8(b"42"), 3040);

        let hex_without_prefix = string::utf8(b"42");
        let not_stripped = strip_hex_prefix(hex_without_prefix);
        assert!(not_stripped == string::utf8(b"42"), 3041);

        let uppercase_prefix = string::utf8(b"0X42");
        let stripped_upper = strip_hex_prefix(uppercase_prefix);
        assert!(stripped_upper == string::utf8(b"42"), 3042);

        // Test edge cases
        let short_0x = string::utf8(b"0x");
        let stripped_short = strip_hex_prefix(short_0x);
        assert!(stripped_short == string::utf8(b""), 3043);

        let single_char = string::utf8(b"a");
        let not_stripped_single = strip_hex_prefix(single_char);
        assert!(not_stripped_single == string::utf8(b"a"), 3044);

        let empty_str = string::utf8(b"");
        let empty_result = strip_hex_prefix(empty_str);
        assert!(empty_result == string::utf8(b""), 3045);
    }

    #[test]
    fun test_try_parse_hex_address_edge_cases() {
        // Test maximum length hex (64 characters)
        let max_hex = string::utf8(b"ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff");
        let result = try_parse_hex_address(&max_hex);
        assert!(option::is_some(&result), 3090);

        // Test with 0x prefix and max length
        let max_hex_with_prefix = string::utf8(b"0xffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff");
        let result2 = try_parse_hex_address(&max_hex_with_prefix);
        assert!(option::is_some(&result2), 3091);

        // Test single character
        let single_char = string::utf8(b"1");
        let result3 = try_parse_hex_address(&single_char);
        assert!(option::is_some(&result3), 3092);

        // Test invalid character
        let invalid_char = string::utf8(b"xyz");
        let result4 = try_parse_hex_address(&invalid_char);
        assert!(option::is_none(&result4), 3093);

        // Test empty string
        let empty_str = string::utf8(b"");
        let result5 = try_parse_hex_address(&empty_str);
        assert!(option::is_none(&result5), 3094);

        // Test too long hex string (more than 64 characters)
        let too_long = string::utf8(b"ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff");
        let result6 = try_parse_hex_address(&too_long);
        assert!(option::is_none(&result6), 3095);
    }
}
