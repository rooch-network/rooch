// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

/// Source from https://github.com/aptos-labs/aptos-core/blob/main/aptos-move/framework/aptos-stdlib/sources/from_bcs.move

/// This module provides a number of functions to convert _primitive_ types from their representation in `std::bcs`
/// to values. This is the opposite of `bcs::to_bytes`. Note that it is not safe to define a generic public `from_bytes`
/// function because this can violate implicit struct invariants, therefore only primitive types are offerred. If
/// a general conversion back-and-force is needed, consider the `moveos_std::Any` type which preserves invariants.
module rooch_framework::bcs {

    /// The request Move type is not match with input Move type.
    const ErrorTypeNotMatch: u64 = 1;

    public fun to_bytes<MoveValue>(v: &MoveValue): vector<u8> {
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

    //TODO https://github.com/rooch-network/rooch/issues/145
    //Relying on private_generics alone cannot guarantee type safety. In order to achieve type safety for from_bytes, several conditions must be met:
    //1. The caller of from_bytes is the module that defines the `T`. This is ensured by private_generics.
    //2. The fields contained in `T` are either primitive types or are defined by the module that calls from_bytes.
    //We need to find a solution to this problem. If we cannot solve it, then we cannot set from_bytes to public.
    // #[private_generics(MoveValue)]
    /// Function to deserialize a type T.
    /// Note the `private_generics` ensure only the `MoveValue`'s owner module can call this function
    native public(friend) fun from_bytes<MoveValue>(bytes: vector<u8>): MoveValue;

    friend rooch_framework::ethereum_light_client;
}
