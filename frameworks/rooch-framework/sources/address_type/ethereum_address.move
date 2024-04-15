// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

module rooch_framework::ethereum_address {
    use std::vector;
    use rooch_framework::ecdsa_k1;
    use moveos_std::hash;

    /// Ethereum addresses are always 20 bytes
    const ETHEREUM_ADDR_LENGTH: u64 = 20;

    // error code
    const ErrorMalformedPublicKey: u64 = 1;
    const ErrorDecompressPublicKey: u64 = 2;
    const ErrorInvaidAddressBytes: u64 = 3;

    #[data_struct]
    struct ETHAddress has store,copy,drop {
        bytes: vector<u8>,
    }

    public fun new(pub_key: vector<u8>): ETHAddress {
        // A pubkey is a 33-bytes compressed public key
        assert!(
            vector::length(&pub_key) == ecdsa_k1::public_key_length(),
            ErrorMalformedPublicKey
        );
        // Decompressing the pubkey to a 65-bytes public key.
        let uncompressed = ecdsa_k1::decompress_pubkey(&pub_key);
        assert!(
            vector::length(&uncompressed) == ecdsa_k1::uncompressed_public_key_length(),
            ErrorDecompressPublicKey
        );
        // Ignore the first byte and take the last 64-bytes of the uncompressed pubkey.
        let uncompressed_64 = vector::empty<u8>();
        let i = 1;
        while (i < 65) {
            let value = vector::borrow(&uncompressed, i);
            vector::push_back(&mut uncompressed_64, *value);
            i = i + 1;
        };
        // Take the last 20 bytes of the hash of the 64-bytes uncompressed pubkey.
        let hashed = hash::keccak256(&uncompressed_64);
        let address_bytes = vector::empty<u8>();
        let i = 12;
        while (i < 32) {
            let value = vector::borrow(&hashed, i);
            vector::push_back(&mut address_bytes, *value);
            i = i + 1;
        };
        // Return the 20 bytes address as the Ethereum address
        ETHAddress {
            bytes: address_bytes,
        }
    }

    public fun from_bytes(bytes: vector<u8>): ETHAddress {
        assert!(
            vector::length(&bytes) == ETHEREUM_ADDR_LENGTH,
            ErrorInvaidAddressBytes
        );
        ETHAddress {
            bytes: bytes,
        }
    }

    public fun as_bytes(addr: &ETHAddress): &vector<u8> {
        &addr.bytes
    }

    public fun into_bytes(addr: ETHAddress): vector<u8> {
        let ETHAddress { bytes } = addr;
        bytes
    }

    #[test]
    fun test_ethereum_address_conversion() {
        // Create a sample compressed Secp256k1 public key (33 bytes)
        let pub_key = x"03a4f75d35449bb6c72757f3e2e765eecb6b15407d9f55678616b7ecf7e7e48211";

        // Create an expected Ethereum address (20 bytes)
        let expected_address = x"6a84eae48a19f1ec014c627d4a1ffc50ce42c516";

        let addr = new(pub_key);

        assert!(addr.bytes == expected_address, 1000);
    }
}
