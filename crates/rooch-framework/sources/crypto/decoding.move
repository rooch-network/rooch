// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

/// Module which defines decoding functions.
module rooch_framework::decoding {

   /// Failed to decode an address
   const ErrorDecodeFailed: u64 = 1;
   
   /// @param encoded_address_bytes: encoded base58 address bytes
   /// Decode the base58 address bytes with Base58 algorithm and returns a raw base58 address bytes
   native public fun base58(encoded_address_bytes: &vector<u8>): vector<u8>;

   /// @param encoded_address_bytes: encoded base58 address bytes
   /// @param version_byte: version byte used for verification of different types of base58 addresses
   /// Decode the base58 address bytes with Base58Check algorithm and returns a raw address bytes without checksum
   native public fun base58check(encoded_address_bytes: &vector<u8>, version_byte: u8): vector<u8>;

   /// @param encoded_bech32_address_bytes: 42 or 62 length Bech32 or Bech32m addresses
   /// Decode the encoded 42 or 62 length Bech32 or Bech32m addresses with Bech32 or Bech32m decoding algorithm and returns 20 or 32 bytes of public keys.
   native public fun bech32(encoded_bech32_address_bytes: &vector<u8>): vector<u8>;

   #[test]
   /// This test can be verified at http://lenschulwitz.com/base58.
   fun test_base58_decoding() {
      let address_bytes = b"1A1zP1eP5QGefi2DMPTfTL5SLmv7DivfNa"; // Satoshi Nakamoto address with Base58 (P2PKH)
      let decoded_address_bytes = base58(&address_bytes);
      let expected_decoded_address_bytes = x"0062e907b15cbf27d5425399ebf6f0fb50ebb88f18c29b7d93";

      assert!(decoded_address_bytes == expected_decoded_address_bytes, 1000);
   }

   #[test]
   /// base58check decoding removes the last 8 digits (4 bytes) checksum.
   fun test_base58check_decoding() {
      let address_bytes = b"1A1zP1eP5QGefi2DMPTfTL5SLmv7DivfNa"; // Satoshi Nakamoto address with Base58 (P2PKH)
      let decoded_address_bytes = base58check(&address_bytes, 0); // Base58 (P2PKH) is verified for script version 0
      let expected_decoded_address_bytes = x"0062e907b15cbf27d5425399ebf6f0fb50ebb88f18";

      assert!(decoded_address_bytes == expected_decoded_address_bytes, 1001);
   }

   #[test]
   /// This test is verified at https://www.blockchain.com/explorer/addresses/btc/bc1qqvdcf32k0vfxgsyet5ldt246q4jaw8scx3sysx0lnstlt6w4m5rsgej0cd.
   fun test_bech32_decoding_to_p2wsh_address() {
      let encoded_address = b"bc1qqvdcf32k0vfxgsyet5ldt246q4jaw8scx3sysx0lnstlt6w4m5rsgej0cd";
      let expected_public_key = bech32(&encoded_address);

      let public_key = x"031b84c5567b126440995d3ed5aaba0565d71e1834604819ff9c17f5e9d5dd07"; // 32-bytes public key for a Bech32 (P2WSH) address
      assert!(public_key == expected_public_key, 1002);
   }

   #[test]
   /// This test is verified at https://www.blockchain.com/explorer/addresses/btc/bc1qq302rl92lgujmvlk5h6pf63zn6wekcjql6cmum.
   fun test_bech32_decoding_to_p2wpkh_address() {
      let encoded_address = b"bc1qq302rl92lgujmvlk5h6pf63zn6wekcjql6cmum";
      let expected_public_key = bech32(&encoded_address);

      let public_key = x"045ea1fcaafa392db3f6a5f414ea229e9d9b6240"; // 20-bytes public key for a Bech32 (P2WPKH) address

      assert!(public_key == expected_public_key, 1003);
   }

   #[test]
   /// This test is verified at https://www.blockchain.com/explorer/addresses/btc/bc1pqvdcf32k0vfxgsyet5ldt246q4jaw8scx3sysx0lnstlt6w4m5rszwjxq3.
   fun test_bech32_encoding_to_p2tr_address() {
      let encoded_address = b"bc1pqvdcf32k0vfxgsyet5ldt246q4jaw8scx3sysx0lnstlt6w4m5rszwjxq3";
      let expected_public_key = bech32(&encoded_address);

      let public_key = x"031b84c5567b126440995d3ed5aaba0565d71e1834604819ff9c17f5e9d5dd07"; // 32-bytes public key for a Bech32 (Taproot) address

      assert!(public_key == expected_public_key, 1004);
   }
}
