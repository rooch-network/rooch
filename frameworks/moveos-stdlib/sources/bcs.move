// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

/// Part source from https://github.com/aptos-labs/aptos-core/blob/main/aptos-move/framework/aptos-stdlib/sources/from_bcs.move

/// This module provides a number of functions to convert _primitive_ types from their representation in `std::bcs`
/// to values. This is the opposite of `bcs::to_bytes`. 
/// Note we provie a generic public `from_bytes` function and protected it with `#[data_struct(T)]`.
module moveos_std::bcs{

    use std::option::{Self, Option};

    friend moveos_std::any;
    friend moveos_std::copyable_any;
    friend moveos_std::address;
    
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

    /// Read `address` value from the bcs-serialized bytes.
    public fun peel_address(v: vector<u8>): address {
        assert!(vector::length(&v) >= address::length(), ErrorInvalidLength);
        let i = 0;
        let addr_bytes = vector::empty<u8>();
        while (i < address::length()) {
            let byte = vector::pop_back(&mut v);
            vector::push_back(&mut addr_bytes, byte);
            i = i + 1;
        };
        to_address(addr_bytes)
    }

    /// Read a `bool` value from bcs-serialized bytes.
    public fun peel_bool(v: vector<u8>): bool {
        let value = peel_u8(v);
        if (value == 0) {
            return false
        } else if (value == 1) {
            return true
        } else {
            abort ErrorInvalidBool
        }
    }

    /// Read `u8` value from bcs-serialized bytes.
    public fun peel_u8(v: vector<u8>): u8 {
        assert!(vector::length(&v) >= 1, ErrorOutOfRange);
        vector::pop_back(&mut v)
    }

    /// Read `u16` value from bcs-serialized bytes.
    public fun peel_u16(v: vector<u8>): u16 {
        assert!(vector::length(&v) >= 2, ErrorOutOfRange);
        let value = 0;
        let i = 0;
        let bits = 16u8;
        while (i < bits) {
            let byte = vector::pop_back(&mut v);
            value = value + (byte << (i as u8));
            i = i + 8;
        };

        value
    }

    /// Read `u32` value from bcs-serialized bytes.
    public fun peel_u32(v: vector<u8>): u32 {
        assert!(vector::length(&v) >= 4, ErrorOutOfRange);
        let value = 0;
        let i = 0;
        let bits = 32u8;
        while (i < bits) {
            let byte = vector::pop_back(&mut v);
            value = value + (byte << (i as u8));
            i = i + 8;
        };

        value
    }

    /// Read `u64` value from bcs-serialized bytes.
    public fun peel_u64(v: vector<u8>): u64 {
        assert!(vector::length(&v) >= 8, ErrorOutOfRange);
        let value = 0;
        let i = 0;
        let bits = 64u8;
        while (i < bits) {
            let byte = vector::pop_back(&mut v);
            value = value + (byte << (i as u8));
            i = i + 8;
        };

        value
    }

    /// Read `u128` value from bcs-serialized bytes.
    public fun peel_u128(v: vector<u8>): u128 {
        assert!(vector::length(&v) >= 16, ErrorOutOfRange);
        let value = 0;
        let i = 0;
        let bits = 128u8;
        while (i < bits) {
            let byte = vector::pop_back(&mut v);
            value = value + (byte << (i as u8));
            i = i + 8;
        };

        value
    }

    /// Read `u256` value from bcs-serialized bytes.
    public fun peel_u256(v: vector<u8>): u256 {
        assert!(vector::length(&v) >= 32, ErrorOutOfRange);
        let value = 0;
        let i = 0;
        let bits = 256u16;
        while (i < bits) {
            let byte = vector::pop_back(&mut v);
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
    public fun peel_vec_length(v: vector<u8>): u64 {
        let total = 0u64;
        let shift = 0;
        let len = 0;
        loop {
            assert!(len <= 4, ErrorLengthOutOfRange);
            let byte = vector::pop_back(&mut v) as u64;
            len = len + 1;
            total = total | ((byte & 0x7f) << shift);
            if ((byte & 0x80) == 0) {
                break;
            }
            shift = shift + 7;
        };
        total
    }

    /// Peel a vector of `address` from serialized bytes.
    public fun peel_vec_address(v: vector<u8>): vector<address> {
        let len = peel_vec_length(v);
        let i = 0;
        let addresses = vector::empty<address>();
        while (i < len) {
            let address = peel_address(v);
            vector::push_back(&mut addresses, address);
            i = i + 1;
        };
        addresses
    }

    /// Peel a vector of `address` from serialized bytes.
    public fun peel_vec_bool(v: vector<u8>): vector<bool> {
        let len = peel_vec_length(v);
        let i = 0;
        let bools = vector::empty<bool>();
        while (i < len) {
            let bool = peel_bool(v);
            vector::push_back(&mut bools, bool);
            i = i + 1;
        };
        bools
    }

    /// Peel a vector of `u8` (eg string) from serialized bytes.
    public fun peel_vec_u8(v: vector<u8>): vector<u8> {
        let len = peel_vec_length(v);
        let i = 0;
        let u8s = vector::empty<u8>();
        while (i < len) {
            let u8 = peel_u8(v);
            vector::push_back(&mut u8s, u8);
            i = i + 1;
        };
        u8s
    }

    /// Peel a `vector<vector<u8>>` (eg vec of string) from serialized bytes.
    public fun peel_vec_vec_u8(v: vector<u8>): vector<vector<u8>> {
        let len = peel_vec_length(v);
        let i = 0;
        let vec_u8s = vector::empty<vector<u8>>();
        while (i < len) {
            let vec_u8 = peel_vec_u8(v);
            vector::push_back(&mut vec_u8s, vec_u8);
            i = i + 1;
        };
        vec_u8s
    }

    /// Peel a vector of `u16` from serialized bytes.
    public fun peel_vec_u16(v: vector<u8>): vector<u16> {
        let len = peel_vec_length(v);
        let i = 0;
        let u16s = vector::empty<u16>();
        while (i < len) {
            let u16 = peel_u16(v);
            vector::push_back(&mut u16s, u16);
            i = i + 1;
        };
        u16s
    }

    /// Peel a vector of `u32` from serialized bytes.
    public fun peel_vec_u32(v: vector<u8>): vector<u32> {
        let len = peel_vec_length(v);
        let i = 0;
        let u32s = vector::empty<u32>();
        while (i < len) {
            let u32 = peel_u32(v);
            vector::push_back(&mut u32s, u32);
            i = i + 1;
        };
        u32s
    }

    /// Peel a vector of `u64` from serialized bytes.
    public fun peel_vec_u64(v: vector<u8>): vector<u64> {
        let len = peel_vec_length(v);
        let i = 0;
        let u64s = vector::empty<u64>();
        while (i < len) {
            let u64 = peel_u64(v);
            vector::push_back(&mut u64s, u64);
            i = i + 1;
        };
        u64s
    }

    /// Peel a vector of `u128` from serialized bytes.
    public fun peel_vec_u128(v: vector<u8>): vector<u128> {
        let len = peel_vec_length(v);
        let i = 0;
        let u128s = vector::empty<u128>();
        while (i < len) {
            let u128 = peel_u128(v);
            vector::push_back(&mut u128s, u128);
            i = i + 1;
        };
        u128s
    }

    /// Peel a vector of `u256` from serialized bytes.
    public fun peel_vec_u256(v: vector<u8>): vector<u256> {
        let len = peel_vec_length(v);
        let i = 0;
        let u256s = vector::empty<u256>();
        while (i < len) {
            let u256 = peel_u256(v);
            vector::push_back(&mut u256s, u256);
            i = i + 1;
        };
        u256s
    }

    // === Option<T> ===

    /// Peel `Option<address>` from serialized bytes.
    public fun peel_option_address(v: vector<u8>): Option<address> {
        if (peel_bool(v)) {
            option::some(peel_address(v))
        } else {
            option::none()
        }
    }

    /// Peel `Option<bool>` from serialized bytes.
    public fun peel_option_bool(v: vector<u8>): Option<bool> {
        if (peel_bool(v)) {
            option::some(peel_bool(v))
        } else {
            option::none()
        }
    }

    /// Peel `Option<u8>` from serialized bytes.
    public fun peel_option_u8(v: vector<u8>): Option<u8> {
        if (peel_bool(v)) {
            option::some(peel_u8(v))
        } else {
            option::none()
        }
    }

    /// Peel `Option<u16>` from serialized bytes.
    public fun peel_option_u16(v: vector<u8>): Option<u16> {
        if (peel_bool(v)) {
            option::some(peel_u16(v))
        } else {
            option::none()
        }
    }

    /// Peel `Option<u32>` from serialized bytes.
    public fun peel_option_u32(v: vector<u8>): Option<u32> {
        if (peel_bool(v)) {
            option::some(peel_u32(v))
        } else {
            option::none()
        }
    }

    /// Peel `Option<u64>` from serialized bytes.
    public fun peel_option_u64(v: vector<u8>): Option<u64> {
        if (peel_bool(v)) {
            option::some(peel_u64(v))
        } else {
            option::none()
        }
    }

    /// Peel `Option<u128>` from serialized bytes.
    public fun peel_option_u128(v: vector<u8>): Option<u128> {
        if (peel_bool(v)) {
            option::some(peel_u128(v))
        } else {
            option::none()
        }
    }

    /// Peel `Option<u256>` from serialized bytes.
    public fun peel_option_u256(v: vector<u8>): Option<u256> {
        if (peel_bool(v)) {
            option::some(peel_u256(v))
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

    // TODO: add test cases for this module.
}
