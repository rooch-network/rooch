// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

module bitcoin_move::bitcoin_hash{

    use std::vector;
    use std::hash;
    use std::option::{Self, Option};
    use std::string;
    use moveos_std::address;
    use moveos_std::hex;

    const ErrorInvalidHex: u64 = 1;

    /// Convert an ascii hex string bytes to Bitcoin Hash
    /// Because Bitcoin Hash hex is reversed, we need to reverse the bytes
    /// Abort if the input is not a valid hex
    public fun from_ascii_bytes(bytes: &vector<u8>): address {
        let hash_opt = from_ascii_bytes_option(bytes);
        assert!(option::is_some(&hash_opt), ErrorInvalidHex);
        option::destroy_some(hash_opt)
    }

    /// Convert an ascii hex string bytes to Bitcoin Hash
    /// Because Bitcoin Hash hex is reversed, we need to reverse the bytes
    /// Return None if the input is not a valid hex
    public fun from_ascii_bytes_option(bytes: &vector<u8>): Option<address> {
        let hex_opt = hex::decode_option(bytes);
        if(option::is_none(&hex_opt)){
            return option::none()
        };
        let decoded_bytes = option::destroy_some(hex_opt);
        vector::reverse(&mut decoded_bytes);
        address::from_bytes_option(decoded_bytes)
    }

    /// Convert Bitcoin Hash to hex string
    /// Because Bitcoin Hash hex is reversed, we need to reverse the bytes
    public fun to_string(hash: address): string::String {
        let bytes = address::to_bytes(&hash);
        vector::reverse(&mut bytes);
        string::utf8(hex::encode(bytes))
    }

    /// Bitcoin hash is double sha256 of the input
    public fun sha256d(input: vector<u8>): address {
        let hash1 = hash::sha2_256(input);
        let hash2 = hash::sha2_256(hash1);
        address::from_bytes(hash2)
    }

    #[test]
    fun test_from_ascii_bytes() {
        let input = b"00000000e47349de5a0193abc5a2fe0be81cb1d1987e45ab85f3289d54cddc4d";
        let hash = from_ascii_bytes(&input);
        //std::debug::print(&hash);
        let expected = @0x4ddccd549d28f385ab457e98d1b11ce80bfea2c5ab93015ade4973e400000000;
        assert!(hash == expected, 1);
        let hash_str = to_string(hash);
        //std::debug::print(&hash_str);
        assert!(string::into_bytes(hash_str) == input, 2);
    }
}