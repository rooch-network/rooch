// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

/// Part source from https://github.com/aptos-labs/aptos-core/blob/main/aptos-move/framework/aptos-stdlib/sources/from_bcs.move

/// This module provides a number of functions to convert _primitive_ types from their representation in `std::bcs`
/// to values. This is the opposite of `bcs::to_bytes`. 
/// Note we provie a generic public `from_bytes` function and protected it with `#[data_struct(T)]`.
module moveos_std::bcs{

    use std::option::{Self, Option};
    use std::vector;

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
        let bytes = bcs.bytes;
        assert!(vector::length(&bytes) >= 32, ErrorInvalidLength);
        let i = 0;
        let addr_bytes = vector::empty<u8>();
        while (i < 32) {
            let byte = vector::pop_back(&mut bytes);
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
        let bytes = bcs.bytes;
        assert!(vector::length(&bytes) >= 1, ErrorOutOfRange);
        vector::pop_back(&mut bytes)
    }

    /// Read `u16` value from bcs-serialized bytes.
    public fun peel_u16(bcs: &mut BCS): u16 {
        let bytes = bcs.bytes;
        assert!(vector::length(&bytes) >= 2, ErrorOutOfRange);
        let value = 0;
        let i = 0;
        let bits = 16u8;
        while (i < bits) {
            let byte = (vector::pop_back(&mut bytes) as u16);
            value = value + (byte << (i as u8));
            i = i + 8;
        };

        value
    }

    /// Read `u32` value from bcs-serialized bytes.
    public fun peel_u32(bcs: &mut BCS): u32 {
        let bytes = bcs.bytes;
        assert!(vector::length(&bytes) >= 4, ErrorOutOfRange);
        let value = 0;
        let i = 0;
        let bits = 32u8;
        while (i < bits) {
            let byte = (vector::pop_back(&mut bytes) as u32);
            value = value + (byte << (i as u8));
            i = i + 8;
        };

        value
    }

    /// Read `u64` value from bcs-serialized bytes.
    public fun peel_u64(bcs: &mut BCS): u64 {
        let bytes = bcs.bytes;
        assert!(vector::length(&bytes) >= 8, ErrorOutOfRange);
        let value = 0;
        let i = 0;
        let bits = 64u8;
        while (i < bits) {
            let byte = (vector::pop_back(&mut bytes) as u64);
            value = value + (byte << (i as u8));
            i = i + 8;
        };

        value
    }

    /// Read `u128` value from bcs-serialized bytes.
    public fun peel_u128(bcs: &mut BCS): u128 {
        let bytes = bcs.bytes;
        assert!(vector::length(&bytes) >= 16, ErrorOutOfRange);
        let value = 0;
        let i = 0;
        let bits = 128u8;
        while (i < bits) {
            let byte = (vector::pop_back(&mut bytes) as u128);
            value = value + (byte << (i as u8));
            i = i + 8;
        };

        value
    }

    /// Read `u256` value from bcs-serialized bytes.
    public fun peel_u256(bcs: &mut BCS): u256 {
        let bytes = bcs.bytes;
        assert!(vector::length(&bytes) >= 32, ErrorOutOfRange);
        let value = 0;
        let i = 0;
        let bits = 256u16;
        while (i < bits) {
            let byte = (vector::pop_back(&mut bytes) as u256);
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
        let bytes = bcs.bytes;
        let total = 0u64;
        let shift = 0;
        let len = 0;
        loop {
            assert!(len <= 4, ErrorLengthOutOfRange);
            let byte = (vector::pop_back(&mut bytes) as u64);
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
    fun test_peel_vec_address_success() {
        let bytes = x"7fe695faf7047ccfbc85f7dccb6c405d4e9b7b44788e71a71c3891a06ce0ca12";
        let bcs = new(bytes);
        let vec_address = peel_vec_address(&mut bcs);
        let expected_vec_address = vector::empty<address>();
        let expected_address = @0x7fe695faf7047ccfbc85f7dccb6c405d4e9b7b44788e71a71c3891a06ce0ca12;
        let i = 0;
        while (i < vector::length(&vec_address)) {
            vector::push_back(&mut expected_vec_address, expected_address);
            i = i + 1;
        };
        assert!(vec_address == expected_vec_address, ErrorLengthOutOfRange);
    }

    #[test]
    fun test_peel_vec_bool_success() {
        let bytes = x"01";
        let bcs = new(bytes);
        let vec_bool = peel_vec_bool(&mut bcs);
        let expected_vec_bool = vector::empty<bool>();
        let expected_bool = true;
        vector::push_back(&mut expected_vec_bool, expected_bool);
        assert!(vec_bool == expected_vec_bool, ErrorLengthOutOfRange);
    }

    #[test]
    fun test_peel_vec_u8_success() {
        let bytes = x"11";
        let bcs = new(bytes);
        let u8 = peel_vec_u8(&mut bcs);
        let expected_vec_u8 = x"1111111111111111111111111111111111";
        assert!(u8 == expected_vec_u8, ErrorLengthOutOfRange);
    }

    #[test]
    fun test_peel_vec_vec_u8_success() {
        let bytes = x"11";
        let bcs = new(bytes);
        let vec_vec_u8 = peel_vec_vec_u8(&mut bcs);
        let expected_vec_vec_u8 = vector::empty<vector<u8>>();
        let expected_vec_u8 = x"1111111111111111111111111111111111";
        let i = 0;
        while (i < vector::length(&vec_vec_u8)) {
            vector::push_back(&mut expected_vec_vec_u8, expected_vec_u8);
            i = i + 1;
        };
        assert!(vec_vec_u8 == expected_vec_vec_u8, ErrorLengthOutOfRange);
    }

    #[test]
    fun test_peel_vec_u16_success() {
        let bytes = x"1111";
        let bcs = new(bytes);
        let vec_u16 = peel_vec_u16(&mut bcs);
        let expected_vec_u16 = vector::empty<u16>();
        let expected_u16 = 4369u16;
        let i = 0;
        while (i < vector::length(&vec_u16)) {
            vector::push_back(&mut expected_vec_u16, expected_u16);
            i = i + 1;
        };
        assert!(vec_u16 == expected_vec_u16, ErrorLengthOutOfRange);
    }

    #[test]
    fun test_peel_vec_u32_success() {
        let bytes = x"11111111";
        let bcs = new(bytes);
        let vec_u32 = peel_vec_u32(&mut bcs);
        let expected_vec_u32 = vector::empty<u32>();
        let expected_u32 = 286331153u32;
        let i = 0;
        while (i < vector::length(&vec_u32)) {
            vector::push_back(&mut expected_vec_u32, expected_u32);
            i = i + 1;
        };
        assert!(vec_u32 == expected_vec_u32, ErrorLengthOutOfRange);
    }

    #[test]
    fun test_peel_vec_u64_success() {
        let bytes = x"1111111111111111";
        let bcs = new(bytes);
        let vec_u64 = peel_vec_u64(&mut bcs);
        let expected_vec_u64 = vector::empty<u64>();
        let expected_u64 = 1229782938247303441u64;
        let i = 0;
        while (i < vector::length(&vec_u64)) {
            vector::push_back(&mut expected_vec_u64, expected_u64);
            i = i + 1;
        };
        assert!(vec_u64 == expected_vec_u64, ErrorLengthOutOfRange);
    }

    #[test]
    fun test_peel_vec_u128_success() {
        let bytes = x"11111111111111111111111111111111";
        let bcs = new(bytes);
        let vec_u128 = peel_vec_u128(&mut bcs);
        let expected_vec_u128 = vector::empty<u128>();
        let expected_u128 = 22685491128062564230891640495451214097u128;
        let i = 0;
        while (i < vector::length(&vec_u128)) {
            vector::push_back(&mut expected_vec_u128, expected_u128);
            i = i + 1;
        };
        assert!(vec_u128 == expected_vec_u128, ErrorLengthOutOfRange);
    }

    #[test]
    fun test_peel_vec_u256_success() {
        let bytes = x"1111111111111111111111111111111111111111111111111111111111111111";
        let bcs = new(bytes);
        let vec_u256 = peel_vec_u256(&mut bcs);
        let expected_vec_u256 = vector::empty<u256>();
        let expected_u256 = 7719472615821079694904732333912527190217998977709370935963838933860875309329u256;
        let i = 0;
        while (i < vector::length(&vec_u256)) {
            vector::push_back(&mut expected_vec_u256, expected_u256);
            i = i + 1;
        };
        assert!(vec_u256 == expected_vec_u256, ErrorLengthOutOfRange);
    }

    #[test]
    fun test_peel_option_address_success() {
        let bytes = x"017fe695faf7047ccfbc85f7dccb6c405d4e9b7b44788e71a71c3891a06ce0ca12";
        let bcs = new(bytes);
        let option_address = peel_option_address(&mut bcs);
        let expected_option_addr = option::some(@0x17fe695faf7047ccfbc85f7dccb6c405d4e9b7b44788e71a71c3891a06ce0ca);
        assert!(option_address == expected_option_addr, ErrorInvalidLength);
    }

    #[test]
    fun test_peel_option_bool_success() {
        let bytes = x"01";
        let bcs = new(bytes);
        let option_bool = peel_option_bool(&mut bcs);
        let expected_option_bool = option::some(true);
        assert!(option_bool == expected_option_bool, ErrorInvalidBool);
    }

    #[test]
    fun test_peel_option_u8_success() {
        let bytes = x"0101";
        let bcs = new(bytes);
        let option_u8 = peel_option_u8(&mut bcs);
        std::debug::print(&option_u8);
        let expected_option_u8 = option::some(1);
        assert!(option_u8 == expected_option_u8, ErrorOutOfRange);
    }

    #[test]
    fun test_peel_option_u16_success() {
        let bytes = x"010011";
        let bcs = new(bytes);
        let option_u16 = peel_option_u16(&mut bcs);
        std::debug::print(&option_u16);
        let expected_option_u16 = option::some(1);
        assert!(option_u16 == expected_option_u16, ErrorOutOfRange);
    }

    #[test]
    fun test_peel_option_u32_success() {
        let bytes = x"0100001111";
        let bcs = new(bytes);
        let option_u32 = peel_option_u32(&mut bcs);
        std::debug::print(&option_u32);
        let expected_option_u32 = option::some(285212673);
        assert!(option_u32 == expected_option_u32, ErrorOutOfRange);
    }

    #[test]
    fun test_peel_option_u64_success() {
        let bytes = x"010000000011111111";
        let bcs = new(bytes);
        let option_u64 = peel_option_u64(&mut bcs);
        std::debug::print(&option_u64);
        let expected_option_u64 = option::some(1229782864946528257);
        assert!(option_u64 == expected_option_u64, ErrorOutOfRange);
    }

    #[test]
    fun test_peel_option_u128_success() {
        let bytes = x"0100000000111111110000000011111111";
        let bcs = new(bytes);
        let option_u128 = peel_option_u128(&mut bcs);
        std::debug::print(&option_u128);
        let expected_option_u128 = option::some(22685489775901924302271377683643367425);
        assert!(option_u128 == expected_option_u128, ErrorOutOfRange);
    }

    #[test]
    fun test_peel_option_u256_success() {
        let bytes = x"010000000011111111000000001111111100000000111111110000000011111111";
        let bcs = new(bytes);
        let option_u256 = peel_option_u256(&mut bcs);
        std::debug::print(&option_u256);
        let expected_option_u256 = option::some(7719472155704656682663016097251860136573850893801914284240011010441236971521);
        assert!(option_u256 == expected_option_u256, ErrorOutOfRange);
    }
}
