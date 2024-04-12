// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

/// Module which defines bech32 functions.
module moveos_std::bech32 {

   /// @param public_key: 20 or 32 bytes public keys
   /// @param witness_version: 0 for bech32 encoding and 1-16 for bech32m encoding.
   /// Encode the public keys with Bech32 or Bech32m encoding algorithm and returns 42 or 62 length Bech32 or Bech32m addresses.
   native public fun encoding(public_key: &vector<u8>, witness_version: u8): vector<u8>;

   /// @param data: 42 or 62 length Bech32 or Bech32m address bytes
   /// Decode the encoded 42 or 62 length Bech32 or Bech32m address bytes with Bech32 or Bech32m decoding algorithm and returns 20 or 32 bytes of public keys.
   native public fun decoding(data: &vector<u8>): vector<u8>;

   #[test]
   fun test_encoding_with_bech32_32_bytes_pk() {
      let public_key = x"031b84c5567b126440995d3ed5aaba0565d71e1834604819ff9c17f5e9d5dd07"; // 32-bytes public key for a Bech32 address
      let version = 0; // version 0 for a Bech32 address
      
      let encoded_address = bech32(&public_key, version);
      let expected_encoded_address = b"bech321qvdcf32k0vfxgsyet5ldt246q4jaw8scx3sysx0lnstlt6w4m5rsl7wnyw";

      assert!(encoded_address == expected_encoded_address, 1000);
   }

   #[test]
   fun test_encoding_with_bech32_20_bytes_pk() {
      let public_key = x"045ea1fcaafa392db3f6a5f414ea229e9d9b6240"; // 20-bytes public key for a Bech32 address
      let version = 0; // version 0 for a Bech32 address
      
      let encoded_address = bech32(&public_key, version);
      let expected_encoded_address = b"bech321q302rl92lgujmvlk5h6pf63zn6wekcjqxj30hr";

      assert!(encoded_address == expected_encoded_address, 1001);
   }

   #[test]
   fun test_encoding_with_bech32m_32_bytes_pk() {
      let public_key = x"031b84c5567b126440995d3ed5aaba0565d71e1834604819ff9c17f5e9d5dd07"; // 32-bytes public key for a Bech32m address
      let version = 1; // version 1 for a Bech32m address
      
      let encoded_address = bech32(&public_key, version);
      let expected_encoded_address = b"bech32m1qvdcf32k0vfxgsyet5ldt246q4jaw8scx3sysx0lnstlt6w4m5rs6l85rk";

      assert!(encoded_address == expected_encoded_address, 1002);
   }

   #[test]
   fun test_decoding_to_bech32_52_bytes_pk() {
      let encoded_address = b"bech321qvdcf32k0vfxgsyet5ldt246q4jaw8scx3sysx0lnstlt6w4m5rsl7wnyw";
      // TODO handle bech32 and bech32m public key's difference
      let expected_public_key = bech32(&encoded_address);

      let public_key = x"000c0d1809110a160f0c0906081004190b141f0d0b0a151a0015121d0e0710180611100410060f1f13100b1f0b1a0e151b140310"; // 52-bytes raw public key for a Bech32 address
      assert!(public_key == expected_public_key, 1003);
   }

   #[test]
   fun test_decoding_to_bech32_32_bytes_pk() {
      let encoded_address = b"bech321q302rl92lgujmvlk5h6pf63zn6wekcjqxj30hr";
      let expected_public_key = bech32(&encoded_address);

      let public_key = x"00110f0a031f050a1f081c121b0c1f1614171a01091a1102131a0e1916181200"; // 32-bytes raw public key for a Bech32 address
      assert!(public_key == expected_public_key, 1004);
   }

   #[test]
   fun test_decoding_to_bech32m_52_bytes_pk() {
      let encoded_address = b"bech32m1qvdcf32k0vfxgsyet5ldt246q4jaw8scx3sysx0lnstlt6w4m5rs6l85rk";
      // TODO handle bech32 and bech32m public key's difference
      let expected_public_key = bech32(&encoded_address);

      let public_key = x"000c0d1809110a160f0c0906081004190b141f0d0b0a151a0015121d0e0710180611100410060f1f13100b1f0b1a0e151b140310"; // 52-bytes public key for a Bech32m address
      assert!(public_key == expected_public_key, 1005);
   }
}
