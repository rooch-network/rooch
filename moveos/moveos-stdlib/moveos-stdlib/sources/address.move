//Source https://github.com/MystenLabs/sui/blob/598f106ef5fbdfbe1b644236f0caf46c94f4d1b7/crates/sui-framework/sources/address.move

module moveos_std::address {
    use std::ascii;
    use moveos_std::bcs;
    use moveos_std::hex;

    /// The length of an address, in bytes
    const LENGTH: u64 = 32;

    // The largest integer that can be represented with 32 bytes
    const MAX: u256 = 115792089237316195423570985008687907853269984665640564039457584007913129639935;

    /// Error from `from_bytes` when it is supplied too many or too few bytes.
    const EAddressParseError: u64 = 0;

    /// Error from `from_u256` when
    const EU256TooBigToConvertToAddress: u64 = 1;

    //TODO
    /// Convert `a` into a u256 by interpreting `a` as the bytes of a big-endian integer
    /// (e.g., `to_u256(0x1) == 1`)
    //native public fun to_u256(a: address): u256;

    //TODO
    /// Convert `n` into an address by encoding it as a big-endian integer (e.g., `from_u256(1) = @0x1`)
    /// Aborts if `n` > `MAX_ADDRESS`
    //native public fun from_u256(n: u256): address;

    /// Convert `bytes` into an address.
    /// Aborts with `EAddressParseError` if the length of `bytes` is invalid length
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

    /// Convert `a` to a hex-encoded ASCII string
    //TODO add `from_ascii` to string module
    // public fun to_string(a: address): string::String {
    //     string::from_ascii(to_ascii_string(a))
    // }

    /// Length of a Sui address in bytes
    public fun length(): u64 {
        LENGTH
    }

    /// Largest possible address
    public fun max(): u256 {
        MAX
    }

}
