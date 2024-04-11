// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

/// Module which defines encoding functions.
module rooch_framework::encoding {

   /// Invalid publich key
   const ErrorInvalidPubkey: u64 = 1;

   /// Excessive script size
   const ErrorExcessiveScriptSize: u64 = 2;

   /// Invalid data
   const ErrorInvalidData: u64 = 3;

   /// Invalid script version
   const ErrorInvalidScriptVersion: u64 = 4;
   
   /// @param address_bytes: address bytes for base58 format
   /// Encode the address bytes with Base58 algorithm and returns an encoded base58 bytes
   native public fun base58(address_bytes: &vector<u8>): vector<u8>;

   /// @param address_bytes: address bytes on the base58 checksum format
   /// @param version_byte: version byte used for verification of different types of checksum addresses
   /// Encode the address bytes with Base58Check algorithm and returns an encoded base58 bytes with checksum
   native public fun base58check(address_bytes: &vector<u8>, version_byte: u8): vector<u8>;

   /// @param public_key: 20 or 32 bytes public keys
   /// @param version: 0 for bech32 encoding and 1 for bech32m encoding. 2-16 are held.
   /// Encode the public keys with Bech32 or Bech32m encoding algorithm and returns 42 or 62 length Bech32 or Bech32m addresses.
   native public fun bech32(public_key: &vector<u8>, version: u8): vector<u8>;

   #[test]
   /// This test can be verified at http://lenschulwitz.com/base58.
   fun test_base58_encoding() {
      let address_bytes = x"0062e907b15cbf27d5425399ebf6f0fb50ebb88f18c29b7d93"; 
      let encoded_address_bytes = base58(&address_bytes);
      let expected_encoded_address_bytes = b"1A1zP1eP5QGefi2DMPTfTL5SLmv7DivfNa"; // Satoshi Nakamoto address with Base58 (P2PKH)

      assert!(encoded_address_bytes == expected_encoded_address_bytes, 1000);
   }

   #[test]
   /// base58check encoding adds 2 digits (1 bytes) checksum to the beginning of the hex string and 12 digits (6 bytes) checksum to the end of the hex string.
   fun test_base58check_encoding() {
      use std::vector;
      let address_bytes = x"0062e907b15cbf27d5425399ebf6f0fb50ebb88f18";
      let encoded_address_bytes = base58check(&address_bytes, 0); // Use script version 0 for verifying Base58 (P2PKH) address

      let truncated_encoded_address_bytes = vector::empty<u8>();
      let i = 1;
      while (i < vector::length(&encoded_address_bytes) - 6) {
         let value = vector::borrow(&encoded_address_bytes, i);
         vector::push_back(&mut truncated_encoded_address_bytes, *value);
         i = i + 1;
      };

      let expected_truncated_encoded_address_bytes = b"1A1zP1eP5QGefi2DMPTfTL5SLmv7"; // last 6 bytes are replaced with "CF73PR" and stripped.

      assert!(truncated_encoded_address_bytes == expected_truncated_encoded_address_bytes, 1001);
   }

   #[test]
   fun test_bech32_encoding_with_32_public_key() {
      let public_key = x"031b84c5567b126440995d3ed5aaba0565d71e1834604819ff9c17f5e9d5dd07"; // 32-bytes public key for a Bech32 address
      let version = 0; // version 0 for a Bech32 address
      
      let encoded_address = bech32(&public_key, version);
      let expected_encoded_address = b"bech321qvdcf32k0vfxgsyet5ldt246q4jaw8scx3sysx0lnstlt6w4m5rsl7wnyw";

      assert!(encoded_address == expected_encoded_address, 1002);
   }

   #[test]
   fun test_bech32_encoding_with_20_public_key() {
      let public_key = x"045ea1fcaafa392db3f6a5f414ea229e9d9b6240"; // 20-bytes public key for a Bech32 address
      let version = 0; // version 0 for a Bech32 address
      
      let encoded_address = bech32(&public_key, version);
      let expected_encoded_address = b"bech321q302rl92lgujmvlk5h6pf63zn6wekcjqxj30hr";

      assert!(encoded_address == expected_encoded_address, 1003);
   }

   #[test]
   fun test_bech32_encoding_with_version_1_32_public_key() {
      let public_key = x"031b84c5567b126440995d3ed5aaba0565d71e1834604819ff9c17f5e9d5dd07"; // 32-bytes public key for a Bech32m address
      let version = 1; // version 1 for a Bech32m address
      
      let encoded_address = bech32(&public_key, version);
      let expected_encoded_address = b"bech32m1qvdcf32k0vfxgsyet5ldt246q4jaw8scx3sysx0lnstlt6w4m5rs6l85rk";

      assert!(encoded_address == expected_encoded_address, 1004);
   }
}
