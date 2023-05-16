/// Source from https://github.com/aptos-labs/aptos-core/blob/main/aptos-move/framework/aptos-stdlib/sources/from_bcs.move

/// This module provides a number of functions to convert _primitive_ types from their representation in `std::bcs`
/// to values. This is the opposite of `bcs::to_bytes`. Note that it is not safe to define a generic public `from_bytes`
/// function because this can violate implicit struct invariants, therefore only primitive types are offerred. If
/// a general conversion back-and-force is needed, consider the `moveos_std::Any` type which preserves invariants.
module moveos_std::bcd{
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

    #[private_generics(MoveValue)]
    /// Function to deserialize a type T.
    /// Note the `private_generics` ensure only the `MoveValue`'s owner module can call this function
    public native fun from_bytes<MoveValue>(bytes: vector<u8>): MoveValue;
    
    friend moveos_std::any;
    friend moveos_std::copyable_any;

    // TODO: add test cases for this module.
}