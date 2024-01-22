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
   fun test_bech32_decoding_to_52_public_key() {
      let encoded_address = b"bech321qvdcf32k0vfxgsyet5ldt246q4jaw8scx3sysx0lnstlt6w4m5rsl7wnyw";
      // TODO handle bech32 and bech32m public key's difference
      let expected_public_key = bech32(&encoded_address);

      let public_key = x"000c0d1809110a160f0c0906081004190b141f0d0b0a151a0015121d0e0710180611100410060f1f13100b1f0b1a0e151b140310"; // 52-bytes raw public key for a Bech32 address
      assert!(public_key == expected_public_key, 1002);
   }

   #[test]
   fun test_bech32_decoding_to_32_public_key() {
      let encoded_address = b"bech321q302rl92lgujmvlk5h6pf63zn6wekcjqxj30hr";
      let expected_public_key = bech32(&encoded_address);

      let public_key = x"00110f0a031f050a1f081c121b0c1f1614171a01091a1102131a0e1916181200"; // 32-bytes raw public key for a Bech32 address
      assert!(public_key == expected_public_key, 1003);
   }

   #[test]
   fun test_bech32_decoding_to_52_bech32m_public_key() {
      let encoded_address = b"bech32m1qvdcf32k0vfxgsyet5ldt246q4jaw8scx3sysx0lnstlt6w4m5rs6l85rk";
      // TODO handle bech32 and bech32m public key's difference
      let expected_public_key = bech32(&encoded_address);

      let public_key = x"000c0d1809110a160f0c0906081004190b141f0d0b0a151a0015121d0e0710180611100410060f1f13100b1f0b1a0e151b140310"; // 52-bytes public key for a Bech32m address
      assert!(public_key == expected_public_key, 1004);
   }
}
