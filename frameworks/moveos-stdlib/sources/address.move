// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

// Copyright (c) Mysten Labs, Inc.
// SPDX-License-Identifier: Apache-2.0

// Source https://github.com/MystenLabs/sui/blob/598f106ef5fbdfbe1b644236f0caf46c94f4d1b7/crates/sui-framework/sources/address.move

module moveos_std::address {
    use std::vector;
    use std::ascii;
    use std::option::{Self, Option};
    use moveos_std::bcs;
    use moveos_std::hex;

    /// The length of an address, in bytes
    const LENGTH: u64 = 32;

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
    /// Aborts with `ErrorAddressParseError` if the length of `bytes` is invalid length
    public fun from_bytes(bytes: vector<u8>): address{
        bcs::to_address(bytes)
    }


    /// Convert `a` into BCS-encoded bytes.
    public fun to_bytes(a: address): vector<u8> {
        bcs::to_bytes(&a)
    }

    /// Convert `a` to a hex-encoded ASCII string
    public fun to_ascii_string(a: address): ascii::String {
        ascii::string(hex::encode(to_bytes(a)))
    }

    /// Convert `a` from a little endian encoding hex ASCII string
    public fun from_ascii_string(a: ascii::String): Option<address> {
        let opt_bytes = hex::decode_option(ascii::into_bytes(a));
        if (option::is_none(&opt_bytes)) {
            return option::none()
        };

        let bytes = option::destroy_some(opt_bytes);

        vector::reverse(&mut bytes); // Convert little endian encoding to big endian
        bcs::from_bytes_option<address>(bytes)
    }

    /// Convert `a` to a hex-encoded ASCII string
    //TODO add `from_ascii` to string module
    // public fun to_string(a: address): string::String {
    //     string::from_ascii(to_ascii_string(a))
    // }

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
    fun test_from_ascii_string_valid() {
        let valid_hex_str = ascii::string(b"000102030405060708090a0b0c0d0e0f101112131415161718191a1b1c1d1e1f");
        let expected_address = @0x1f1e1d1c1b1a191817161514131211100f0e0d0c0b0a09080706050403020100;
        let result = from_ascii_string(valid_hex_str);
        assert!(option::is_some(&result), 1);
        assert!(option::destroy_some(result) == expected_address, 2);
    }

    #[test]
    fun test_from_ascii_string_invalid_characters() {
        let invalid_hex_str = ascii::string(b"0123456789ABCDEFG");
        let result = from_ascii_string(invalid_hex_str);
        assert!(option::is_none(&result), 1);
    }

    #[test]
    fun test_from_ascii_string_empty_string() {
        let empty_hex_str = ascii::string(b"");
        let result = from_ascii_string(empty_hex_str);
        assert!(option::is_none(&result), 1);
    }

    #[test]
    fun test_from_ascii_string_too_short() {
        let short_hex_str = ascii::string(b"0123456789ab");
        let result = from_ascii_string(short_hex_str);
        assert!(option::is_none(&result), 1);
    }

    #[test]
    fun test_from_ascii_string_too_long() {
        let long_hex_str = ascii::string(b"000102030405060708090a0b0c0d0e0f101112131415161718191a1b1c1d1e1fff");
        let result = from_ascii_string(long_hex_str);
        assert!(option::is_none(&result), 1);
    }

    #[test]
    fun test_from_ascii_string_with_non_data_characters() {
        let hex_str_with_spaces = ascii::string(b"01 23 45 67 89 AB CD EF");
        let result = from_ascii_string(hex_str_with_spaces);
        assert!(option::is_none(&result), 1);
    }

    #[test]
    fun test_from_ascii_string_mixed_case() {
        let valid_hex_str = ascii::string(b"000102030405060708090a0b0c0d0e0f101112131415161718191a1b1c1D1E1F");
        let expected_address = @0x1f1e1d1c1b1a191817161514131211100f0e0D0c0b0a09080706050403020100;
        let result = from_ascii_string(valid_hex_str);
        assert!(option::is_some(&result), 1);
        assert!(option::destroy_some(result) == expected_address, 2);
    }
}
