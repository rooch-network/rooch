// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

/// Module which defines base58 functions.
module moveos_std::base58 {

   // Decode failed error
   const E_DECODE_FAILED: u64 = 1;
   
   /// @param address_bytes: address bytes for base58 format
   /// Encode the address bytes with Base58 algorithm and returns an encoded base58 bytes
   native public fun encoding(address_bytes: &vector<u8>): vector<u8>;

   /// @param address_bytes: address bytes on the base58 checksum format
   /// @param version_byte: version byte used for verification of different types of checksum addresses
   /// Encode the address bytes with Base58Check algorithm and returns an encoded base58 bytes with checksum
   native public fun checksum_encoding(address_bytes: &vector<u8>, version_byte: u8): vector<u8>;

   /// @param encoded_address_bytes: encoded base58 address bytes
   /// Decode the base58 address bytes with Base58 algorithm and returns a raw base58 address bytes
   native public fun decoding(encoded_address_bytes: &vector<u8>): vector<u8>;

   /// @param encoded_address_bytes: encoded base58 address bytes
   /// @param version_byte: version byte used for verification of different types of base58 addresses
   /// Decode the base58 address bytes with Base58Check algorithm and returns a raw base58 address bytes without checksum
   native public fun checksum_decoding(encoded_address_bytes: &vector<u8>, version_byte: u8): vector<u8>;

   #[test]
   /// This test can be verified at http://lenschulwitz.com/base58.
   fun test_encoding() {
      let address_bytes = x"0062e907b15cbf27d5425399ebf6f0fb50ebb88f18c29b7d93"; 
      let encoded_address_bytes = encoding(&address_bytes);
      let expected_encoded_address_bytes = b"1A1zP1eP5QGefi2DMPTfTL5SLmv7DivfNa"; // Satoshi Nakamoto address with Base58 (P2PKH)

      assert!(encoded_address_bytes == expected_encoded_address_bytes, 1000);
   }

   #[test]
   /// checksum encoding adds 2 digits (1 bytes) checksum to the beginning of the hex string and 12 digits (6 bytes) checksum to the end of the hex string.
   fun test_checksum_encoding() {
      use std::vector;
      let address_bytes = x"0062e907b15cbf27d5425399ebf6f0fb50ebb88f18";
      let encoded_address_bytes = checksum_encoding(&address_bytes, 0); // Use script version 0 for verifying Base58 (P2PKH) address

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
   /// This test can be verified at http://lenschulwitz.com/base58.
   fun test_decoding() {
      let address_bytes = b"1A1zP1eP5QGefi2DMPTfTL5SLmv7DivfNa"; // Satoshi Nakamoto address with Base58 (P2PKH)
      let decoded_address_bytes = decoding(&address_bytes);
      let expected_decoded_address_bytes = x"0062e907b15cbf27d5425399ebf6f0fb50ebb88f18c29b7d93";

      assert!(decoded_address_bytes == expected_decoded_address_bytes, E_DECODE_FAILED);
   }

   #[test]
   /// checksum decoding removes the last 8 digits (4 bytes) checksum.
   fun test_checksum_decoding() {
      let address_bytes = b"1A1zP1eP5QGefi2DMPTfTL5SLmv7DivfNa"; // Satoshi Nakamoto address with Base58 (P2PKH)
      let decoded_address_bytes = checksum_decoding(&address_bytes, 0); // Base58 (P2PKH) is verified for script version 0
      let expected_decoded_address_bytes = x"0062e907b15cbf27d5425399ebf6f0fb50ebb88f18";

      assert!(decoded_address_bytes == expected_decoded_address_bytes, E_DECODE_FAILED);
   }
}
