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
    /// Aborts with `EAddressParseError` if the length of `s` is not 64,
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
}
