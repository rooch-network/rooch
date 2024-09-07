// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

/// Part source from https://github.com/aptos-labs/aptos-core/blob/main/aptos-move/framework/aptos-stdlib/sources/from_bcs.move

/// This module provides a number of functions to convert _primitive_ types from their representation in `std::bcs`
/// to values. This is the opposite of `bcs::to_bytes`. 
/// Note we provie a generic public `from_bytes` function and protected it with `#[data_struct(T)]`.
module moveos_std::bcs{

    use std::option::{Self, Option};
    use std::vector;
    #[test_only]
    use std::vector::{is_empty, pop_back};

    friend moveos_std::any;
    friend moveos_std::copyable_any;

    /// The request Move type is not match with input Move type.
    const ErrorTypeNotMatch: u64 = 1;
    const ErrorInvalidBytes: u64 = 2;
    const ErrorInvalidLength: u64 = 3;
    const ErrorInvalidBool: u64 = 4;
    const ErrorOutOfRange: u64 = 5;
    const ErrorLengthOutOfRange: u64 = 6;
    
    public fun to_bytes<MoveValue>(v: &MoveValue): vector<u8>{
        std::bcs::to_bytes(v)
    }

    public fun to_bool(v: vector<u8>): bool {
        from_bytes<bool>(v)
    }

    public fun to_u8(v: vector<u8>): u8 {
        from_bytes<u8>(v)
    }

    public fun to_u64(v: vector<u8>): u64 {
        from_bytes<u64>(v)
    }

    public fun to_u128(v: vector<u8>): u128 {
        from_bytes<u128>(v)
    }

    public fun to_address(v: vector<u8>): address {
        from_bytes<address>(v)
    }

    // === BCS struct ===

    /// A helper struct that saves resources on operations. For better
    /// vector performance, it stores reversed bytes of the BCS and
    /// enables use of `vector::pop_back`.
    struct BCS has store, copy, drop {
        bytes: vector<u8>
    }

    /// Creates a new instance of BCS wrapper that holds inversed
    /// bytes for better performance.
    public fun new(bytes: vector<u8>): BCS {
        vector::reverse(&mut bytes);
        BCS { bytes }
    }

    /// Unpack the `BCS` struct returning the leftover bytes.
    /// Useful for passing the data further after partial deserialization.
    public fun into_remainder_bytes(bcs: BCS): vector<u8> {
        let BCS { bytes } = bcs;
        vector::reverse(&mut bytes);
        bytes
    }

    // === Peel functions ===

    /// Read `address` value from the bcs-serialized bytes.
    public fun peel_address(bcs: &mut BCS): address {
        assert!(vector::length(&bcs.bytes) >= 32, ErrorInvalidLength);
        let i = 0;
        let addr_bytes = vector::empty<u8>();
        while (i < 32) {
            let byte = vector::pop_back(&mut bcs.bytes);
            vector::push_back(&mut addr_bytes, byte);
            i = i + 1;
        };
        to_address(addr_bytes)
    }

    /// Read a `bool` value from bcs-serialized bytes.
    public fun peel_bool(bcs: &mut BCS): bool {
        let value = peel_u8(bcs);
        if (value == 0) {
            return false
        } else if (value == 1) {
            return true
        } else {
            abort ErrorInvalidBool
        }
    }

    /// Read `u8` value from bcs-serialized bytes.
    public fun peel_u8(bcs: &mut BCS): u8 {
        assert!(vector::length(&bcs.bytes) >= 1, ErrorOutOfRange);
        vector::pop_back(&mut bcs.bytes)
    }

    /// Read `u16` value from bcs-serialized bytes.
    public fun peel_u16(bcs: &mut BCS): u16 {
        assert!(vector::length(&bcs.bytes) >= 2, ErrorOutOfRange);
        let value = 0;
        let i = 0;
        let bits = 16u8;
        while (i < bits) {
            let byte = (vector::pop_back(&mut bcs.bytes) as u16);
            value = value + (byte << (i as u8));
            i = i + 8;
        };

        value
    }

    /// Read `u32` value from bcs-serialized bytes.
    public fun peel_u32(bcs: &mut BCS): u32 {
        assert!(vector::length(&bcs.bytes) >= 4, ErrorOutOfRange);
        let value = 0;
        let i = 0;
        let bits = 32u8;
        while (i < bits) {
            let byte = (vector::pop_back(&mut bcs.bytes) as u32);
            value = value + (byte << (i as u8));
            i = i + 8;
        };

        value
    }

    /// Read `u64` value from bcs-serialized bytes.
    public fun peel_u64(bcs: &mut BCS): u64 {
        assert!(vector::length(&bcs.bytes) >= 8, ErrorOutOfRange);
        let value = 0;
        let i = 0;
        let bits = 64u8;
        while (i < bits) {
            let byte = (vector::pop_back(&mut bcs.bytes) as u64);
            value = value + (byte << (i as u8));
            i = i + 8;
        };

        value
    }

    /// Read `u128` value from bcs-serialized bytes.
    public fun peel_u128(bcs: &mut BCS): u128 {
        assert!(vector::length(&bcs.bytes) >= 16, ErrorOutOfRange);
        let value = 0;
        let i = 0;
        let bits = 128u8;
        while (i < bits) {
            let byte = (vector::pop_back(&mut bcs.bytes) as u128);
            value = value + (byte << (i as u8));
            i = i + 8;
        };

        value
    }

    /// Read `u256` value from bcs-serialized bytes.
    public fun peel_u256(bcs: &mut BCS): u256 {
        assert!(vector::length(&bcs.bytes) >= 32, ErrorOutOfRange);
        let value = 0;
        let i = 0;
        let bits = 256u16;
        while (i < bits) {
            let byte = (vector::pop_back(&mut bcs.bytes) as u256);
            value = value + (byte << (i as u8));
            i = i + 8;
        };

        value
    }

    // === Vector<T> ===

    /// Read ULEB bytes expecting a vector length. Result should
    /// then be used to perform `peel_*` operation LEN times.
    ///
    /// In BCS `vector` length is implemented with ULEB128;
    /// See more here: https://en.wikipedia.org/wiki/LEB128
    public fun peel_vec_length(bcs: &mut BCS): u64 {
        let total = 0u64;
        let shift = 0;
        let len = 0;
        loop {
            assert!(len <= 4, ErrorLengthOutOfRange);
            let byte = (vector::pop_back(&mut bcs.bytes) as u64);
            len = len + 1;
            total = total | ((byte & 0x7f) << shift);
            if ((byte & 0x80) == 0) {
                break
            };
            shift = shift + 7;
        };
        total
    }

    /// Peel a vector of `address` from serialized bytes.
    public fun peel_vec_address(bcs: &mut BCS): vector<address> {
        let len = peel_vec_length(bcs);
        let i = 0;
        let addresses = vector::empty<address>();
        while (i < len) {
            let address = peel_address(bcs);
            vector::push_back(&mut addresses, address);
            i = i + 1;
        };
        addresses
    }

    /// Peel a vector of `bool` from serialized bytes.
    public fun peel_vec_bool(bcs: &mut BCS): vector<bool> {
        let len = peel_vec_length(bcs);
        let i = 0;
        let bools = vector::empty<bool>();
        while (i < len) {
            let bool = peel_bool(bcs);
            vector::push_back(&mut bools, bool);
            i = i + 1;
        };
        bools
    }

    /// Peel a vector of `u8` (eg string) from serialized bytes.
    public fun peel_vec_u8(bcs: &mut BCS): vector<u8> {
        let len = peel_vec_length(bcs);
        let i = 0;
        let u8s = vector::empty<u8>();
        while (i < len) {
            let u8 = peel_u8(bcs);
            vector::push_back(&mut u8s, u8);
            i = i + 1;
        };
        u8s
    }

    /// Peel a `vector<vector<u8>>` (eg vec of string) from serialized bytes.
    public fun peel_vec_vec_u8(bcs: &mut BCS): vector<vector<u8>> {
        let len = peel_vec_length(bcs);
        let i = 0;
        let vec_u8s = vector::empty<vector<u8>>();
        while (i < len) {
            let vec_u8 = peel_vec_u8(bcs);
            vector::push_back(&mut vec_u8s, vec_u8);
            i = i + 1;
        };
        vec_u8s
    }

    /// Peel a vector of `u16` from serialized bytes.
    public fun peel_vec_u16(bcs: &mut BCS): vector<u16> {
        let len = peel_vec_length(bcs);
        let i = 0;
        let u16s = vector::empty<u16>();
        while (i < len) {
            let u16 = peel_u16(bcs);
            vector::push_back(&mut u16s, u16);
            i = i + 1;
        };
        u16s
    }

    /// Peel a vector of `u32` from serialized bytes.
    public fun peel_vec_u32(bcs: &mut BCS): vector<u32> {
        let len = peel_vec_length(bcs);
        let i = 0;
        let u32s = vector::empty<u32>();
        while (i < len) {
            let u32 = peel_u32(bcs);
            vector::push_back(&mut u32s, u32);
            i = i + 1;
        };
        u32s
    }

    /// Peel a vector of `u64` from serialized bytes.
    public fun peel_vec_u64(bcs: &mut BCS): vector<u64> {
        let len = peel_vec_length(bcs);
        let i = 0;
        let u64s = vector::empty<u64>();
        while (i < len) {
            let u64 = peel_u64(bcs);
            vector::push_back(&mut u64s, u64);
            i = i + 1;
        };
        u64s
    }

    /// Peel a vector of `u128` from serialized bytes.
    public fun peel_vec_u128(bcs: &mut BCS): vector<u128> {
        let len = peel_vec_length(bcs);
        let i = 0;
        let u128s = vector::empty<u128>();
        while (i < len) {
            let u128 = peel_u128(bcs);
            vector::push_back(&mut u128s, u128);
            i = i + 1;
        };
        u128s
    }

    /// Peel a vector of `u256` from serialized bytes.
    public fun peel_vec_u256(bcs: &mut BCS): vector<u256> {
        let len = peel_vec_length(bcs);
        let i = 0;
        let u256s = vector::empty<u256>();
        while (i < len) {
            let u256 = peel_u256(bcs);
            vector::push_back(&mut u256s, u256);
            i = i + 1;
        };
        u256s
    }

    // === Option<T> ===

    /// Peel `Option<address>` from serialized bytes.
    public fun peel_option_address(bcs: &mut BCS): Option<address> {
        if (peel_bool(bcs)) {
            option::some(peel_address(bcs))
        } else {
            option::none()
        }
    }

    /// Peel `Option<bool>` from serialized bytes.
    public fun peel_option_bool(bcs: &mut BCS): Option<bool> {
        if (peel_bool(bcs)) {
            option::some(peel_bool(bcs))
        } else {
            option::none()
        }
    }

    /// Peel `Option<u8>` from serialized bytes.
    public fun peel_option_u8(bcs: &mut BCS): Option<u8> {
        if (peel_bool(bcs)) {
            option::some(peel_u8(bcs))
        } else {
            option::none()
        }
    }

    /// Peel `Option<u16>` from serialized bytes.
    public fun peel_option_u16(bcs: &mut BCS): Option<u16> {
        if (peel_bool(bcs)) {
            option::some(peel_u16(bcs))
        } else {
            option::none()
        }
    }

    /// Peel `Option<u32>` from serialized bytes.
    public fun peel_option_u32(bcs: &mut BCS): Option<u32> {
        if (peel_bool(bcs)) {
            option::some(peel_u32(bcs))
        } else {
            option::none()
        }
    }

    /// Peel `Option<u64>` from serialized bytes.
    public fun peel_option_u64(bcs: &mut BCS): Option<u64> {
        if (peel_bool(bcs)) {
            option::some(peel_u64(bcs))
        } else {
            option::none()
        }
    }

    /// Peel `Option<u128>` from serialized bytes.
    public fun peel_option_u128(bcs: &mut BCS): Option<u128> {
        if (peel_bool(bcs)) {
            option::some(peel_u128(bcs))
        } else {
            option::none()
        }
    }

    /// Peel `Option<u256>` from serialized bytes.
    public fun peel_option_u256(bcs: &mut BCS): Option<u256> {
        if (peel_bool(bcs)) {
            option::some(peel_u256(bcs))
        } else {
            option::none()
        }
    }

    #[data_struct(T)]
    /// Function to deserialize a type T.
    /// Note the `data_struct` ensure the `T` must be a `#[data_struct]` type
    public fun from_bytes<T>(bytes: vector<u8>): T {
        let opt_result = native_from_bytes(bytes);
        assert!(option::is_some(&opt_result), ErrorInvalidBytes);
        option::destroy_some(opt_result)
    }

    #[data_struct(T)]
    /// Function to deserialize a type T.
    /// Note the `data_struct` ensure the `T` must be a `#[data_struct]` type
    /// If the bytes are invalid, it will return None.
    public fun from_bytes_option<T>(bytes: vector<u8>): Option<T> {
        native_from_bytes(bytes)
    }

    native public(friend) fun native_from_bytes<T>(bytes: vector<u8>): Option<T>;
    
    #[test]
    fun test_peel_address_success() {
        let bytes = x"7fe695faf7047ccfbc85f7dccb6c405d4e9b7b44788e71a71c3891a06ce0ca12";
        let bcs = new(bytes);
        let address = peel_address(&mut bcs);
        let expected_addr = @0x7fe695faf7047ccfbc85f7dccb6c405d4e9b7b44788e71a71c3891a06ce0ca12;
        assert!(address == expected_addr, ErrorInvalidLength);
    }

    #[test]
    #[expected_failure(abort_code = ErrorInvalidLength, location = Self)]
    fun test_peel_address_fail_with_length() {
        let bytes = x"7fe695faf7047ccfbc85f7dccb6c405d4e9b7b44788e71a71c3891a06ce0";
        let bcs = new(bytes);
        peel_address(&mut bcs);
    }

    #[test]
    fun test_peel_bool_success() {
        let bytes = x"00";
        let bcs = new(bytes);
        let bool = peel_bool(&mut bcs);
        let expected_bool = false;
        assert!(bool == expected_bool, ErrorInvalidBool);
    }

    #[test]
    #[expected_failure(abort_code = ErrorOutOfRange, location = Self)]
    fun test_peel_bool_fail_with_range() {
        let bytes = x"";
        let bcs = new(bytes);
        peel_bool(&mut bcs);
    }

    #[test]
    #[expected_failure(abort_code = ErrorInvalidBool, location = Self)]
    fun test_peel_bool_fail_with_boolean() {
        let bytes = x"22";
        let bcs = new(bytes);
        peel_bool(&mut bcs);
    }

    #[test]
    fun test_peel_u8_success() {
        let bytes = x"11";
        let bcs = new(bytes);
        let u8 = peel_u8(&mut bcs);
        let expected_u8 = 17;
        assert!(u8 == expected_u8, ErrorOutOfRange);
    }

    #[test]
    #[expected_failure(abort_code = ErrorOutOfRange, location = Self)]
    fun test_peel_u8_fail_with_range() {
        let bytes = x"";
        let bcs = new(bytes);
        peel_u8(&mut bcs);
    }

    #[test]
    fun test_peel_u16_success() {
        let bytes = x"0011";
        let bcs = new(bytes);
        let u16 = peel_u16(&mut bcs);
        let expected_u16 = 4352;
        assert!(u16 == expected_u16, ErrorOutOfRange);
    }

    #[test]
    #[expected_failure(abort_code = ErrorOutOfRange, location = Self)]
    fun test_peel_u16_fail_with_range() {
        let bytes = x"11";
        let bcs = new(bytes);
        peel_u16(&mut bcs);
    }

    #[test]
    fun test_peel_u32_success() {
        let bytes = x"00001111";
        let bcs = new(bytes);
        let u32 = peel_u32(&mut bcs);
        let expected_u32 = 286326784;
        assert!(u32 == expected_u32, ErrorOutOfRange);
    }

    #[test]
    #[expected_failure(abort_code = ErrorOutOfRange, location = Self)]
    fun test_peel_u32_fail_with_range() {
        let bytes = x"000011";
        let bcs = new(bytes);
        peel_u32(&mut bcs);
    }

    #[test]
    fun test_peel_u64_success() {
        let bytes = x"0000000011111111";
        let bcs = new(bytes);
        let u64 = peel_u64(&mut bcs);
        let expected_u64 = 1229782937960972288;
        assert!(u64 == expected_u64, ErrorOutOfRange);
    }

    #[test]
    #[expected_failure(abort_code = ErrorOutOfRange, location = Self)]
    fun test_peel_u64_fail_with_range() {
        let bytes = x"00000000111111";
        let bcs = new(bytes);
        peel_u64(&mut bcs);
    }

    #[test]
    fun test_peel_u128_success() {
        let bytes = x"00000000111111110000000011111111";
        let bcs = new(bytes);
        let u128 = peel_u128(&mut bcs);
        let expected_u128 = 22685491122780686731170467593842589696;
        assert!(u128 == expected_u128, ErrorOutOfRange);
    }

    #[test]
    #[expected_failure(abort_code = ErrorOutOfRange, location = Self)]
    fun test_peel_u128_fail_with_range() {
        let bytes = x"000000001111111100000000111111";
        let bcs = new(bytes);
        peel_u128(&mut bcs);
    }

    #[test]
    fun test_peel_u256_success() {
        let bytes = x"0000000011111111000000001111111100000000111111110000000011111111";
        let bcs = new(bytes);
        let u256 = peel_u256(&mut bcs);
        let expected_u256 = 7719472614023749917513163129863071459539701524256607433418042731035017347072;
        assert!(u256 == expected_u256, ErrorOutOfRange);
    }

    #[test]
    #[expected_failure(abort_code = ErrorOutOfRange, location = Self)]
    fun test_peel_u256_fail_with_range() {
        let bytes = x"00000000111111110000000011111111000000001111111100000000111111";
        let bcs = new(bytes);
        peel_u256(&mut bcs);
    }

    #[test]
    fun test_peel_vec_length_success() {
        let bytes = x"11";
        let bcs = new(bytes);
        let vec_len = peel_vec_length(&mut bcs);
        let expected_vec_len = 17;
        assert!(vec_len == expected_vec_len, ErrorLengthOutOfRange);
    }

    #[test]
    fun test_vec() {
        let bool_cases = vector[vector[], vector[true], vector[false, true, false]];
        let excepet_bool_cases =  bool_cases;
        while (!is_empty(&excepet_bool_cases)) {
            let case = pop_back(&mut excepet_bool_cases);
            let bytes = new(to_bytes(&case));
            assert!(peel_vec_bool(&mut bytes) == case, 0);
            assert!(is_empty(&into_remainder_bytes(bytes)), 1);
        };

        let u8_cases = vector[vector[], vector[1], vector[0, 2, 0xFF]];
        let excepet_u8_cases =  u8_cases;
        while (!is_empty(&excepet_u8_cases)) {
            let case = pop_back(&mut excepet_u8_cases);
            let bytes = new(to_bytes(&case));
            assert!(peel_vec_u8(&mut bytes) == case, 0);
            assert!(is_empty(&into_remainder_bytes(bytes)), 1);
        };

        let u16_cases = vector[vector[], vector[1], vector[0, 2, 0xFFFF]];
        let excepet_u16_cases =  u16_cases;
        while (!is_empty(&excepet_u16_cases)) {
            let case = pop_back(&mut excepet_u16_cases);
            let bytes = new(to_bytes(&case));
            assert!(peel_vec_u16(&mut bytes) == case, 0);
            assert!(is_empty(&into_remainder_bytes(bytes)), 1);
        };

        let u32_cases = vector[vector[], vector[1], vector[0, 2, 0xFFFF_FFFF]];
        let excepet_u32_cases =  u32_cases;
        while (!is_empty(&excepet_u32_cases)) {
            let case = pop_back(&mut excepet_u32_cases);
            let bytes = new(to_bytes(&case));
            assert!(peel_vec_u32(&mut bytes) == case, 0);
            assert!(is_empty(&into_remainder_bytes(bytes)), 1);
        };

        let u64_cases = vector[vector[], vector[1], vector[0, 2, 0xFFFF_FFFF_FFFF_FFFF]];
        let excepet_u64_cases =  u64_cases;
        while (!is_empty(&excepet_u64_cases)) {
            let case = pop_back(&mut excepet_u64_cases);
            let bytes = new(to_bytes(&case));
            assert!(peel_vec_u64(&mut bytes) == case, 0);
            assert!(is_empty(&into_remainder_bytes(bytes)), 1);
        };

        let u128_cases = vector[vector[], vector[1], vector[0, 2, 0xFFFF_FFFF_FFFF_FFFF_FFFF_FFFF_FFFF_FFFF]];
        let excepet_u128_cases =  u128_cases;
        while (!is_empty(&excepet_u128_cases)) {
            let case = pop_back(&mut excepet_u128_cases);
            let bytes = new(to_bytes(&case));
            assert!(peel_vec_u128(&mut bytes) == case, 0);
            assert!(is_empty(&into_remainder_bytes(bytes)), 1);
        };

        let u256_cases = vector[vector[], vector[1], vector[0, 2, 0xFFFF_FFFF_FFFF_FFFF_FFFF_FFFF_FFFF_FFFF_FFFF_FFFF_FFFF_FFFF_FFFF_FFFF_FFFF_FFFF]];
        let excepet_u256_cases =  u256_cases;
        while (!is_empty(&excepet_u256_cases)) {
            let case = pop_back(&mut excepet_u256_cases);
            let bytes = new(to_bytes(&case));
            assert!(peel_vec_u256(&mut bytes) == case, 0);
            assert!(is_empty(&into_remainder_bytes(bytes)), 1);
        };

        let address_cases = vector[vector[], vector[@0x0], vector[@0x1, @0x2, @0x3]];
        let excepet_address_cases =  address_cases;
        while (!is_empty(&excepet_address_cases)) {
            let case = pop_back(&mut excepet_address_cases);
            let bytes = new(to_bytes(&case));
            assert!(peel_vec_address(&mut bytes) == case, 0);
            assert!(is_empty(&into_remainder_bytes(bytes)), 1);
        };

    }
}
