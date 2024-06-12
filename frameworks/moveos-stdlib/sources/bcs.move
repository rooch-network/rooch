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
    
    /// The request Move type is not match with input Move type.
    const ErrorTypeNotMatch: u64 = 1;
    const ErrorInvalidBytes: u64 = 2;
    
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
