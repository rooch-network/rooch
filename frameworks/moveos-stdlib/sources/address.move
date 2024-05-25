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

    /// Convert `a` from a hex-encoded ASCII string
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
}
