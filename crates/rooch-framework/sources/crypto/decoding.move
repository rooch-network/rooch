// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

/// Module which defines decoding functions.
module rooch_framework::decoding {
   /// @param encoded_address_bytes: encoded Bitcoin address bytes on the Bitcoin network 
   /// Decode the Bitcoin address bytes with Base58 algorithm and returns a raw address bytes
   native public fun base58(encoded_address_bytes: &vector<u8>): vector<u8>;

   /// @param encoded_address_bytes: encoded Bitcoin address bytes on the Bitcoin network 
   /// @param version_byte: version byte used on Bitcoin network for verification of different types of addresses
   /// Decode the Bitcoin address bytes with Base58Check algorithm and returns a raw address bytes without checksum
   native public fun base58check(encoded_address_bytes: &vector<u8>, version_byte: u8): vector<u8>;

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
}
