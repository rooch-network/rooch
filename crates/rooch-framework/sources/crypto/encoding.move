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
   
   /// @param address_bytes: address bytes on the Bitcoin network 
   /// Encode the address bytes with Base58 algorithm and returns an encoded Bitcoin address
   native public fun base58(address_bytes: &vector<u8>): vector<u8>;

   /// @param address_bytes: address bytes on the Bitcoin network 
   /// @param version_byte: version byte used on Bitcoin network for verification of different types of addresses
   /// Encode the address bytes with Base58Check algorithm and returns an encoded Bitcoin address with checksum
   native public fun base58check(address_bytes: &vector<u8>, version_byte: u8): vector<u8>;

   /// @param public_key: 20 or 32 bytes public keys
   /// @param version: 0 for bech32 encoding and 1 for bech32m encoding. 2-16 are held.
   /// Encode the public key with Bech32 or Bech32m encoding algorithm and returns 42 or 62 length Bitcoin Bech32 address.
   native public fun bech32(public_key: &vector<u8>, version: u8): vector<u8>;

   /// @param public_key: 33 bytes compressed public key
   /// Creates a pay to script hash P2SH address from a script converted from a compressed public key.
   native public fun p2sh(public_key: &vector<u8>): vector<u8>;

   /// @param public_key: 33 bytes compressed public key
   /// Creates a pay to (compressed) public key hash address from a public key.
   native public fun p2pkh(public_key: &vector<u8>): vector<u8>;

   #[test]
   /// This test can be verified at http://lenschulwitz.com/base58.
   fun test_base58_encoding() {
      let address_bytes = x"0062e907b15cbf27d5425399ebf6f0fb50ebb88f18c29b7d93"; 
      let encoded_address_bytes = base58(&address_bytes);
      let expected_encoded_address_bytes = b"1A1zP1eP5QGefi2DMPTfTL5SLmv7DivfNa"; // Satoshi Nakamoto address with Base58 (P2PKH)

      assert!(encoded_address_bytes == expected_encoded_address_bytes, 1000);
   }

   #[test]
   /// base58check encoding adds 2 digits (1 bytes) checksum to the begining of the hex string and 12 digits (6 bytes) checksum to the end of the hex string.
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
   /// This test is verified at https://www.blockchain.com/explorer/addresses/btc/bc1qqvdcf32k0vfxgsyet5ldt246q4jaw8scx3sysx0lnstlt6w4m5rsgej0cd.
   fun test_bech32_encoding_to_p2wsh_address() {
      let public_key = x"031b84c5567b126440995d3ed5aaba0565d71e1834604819ff9c17f5e9d5dd07"; // 32-bytes public key for a Bech32 (P2WSH) address
      let version = 0; // version 0 for a Bech32 (P2WSH) address
      
      let encoded_address = bech32(&public_key, version);
      let expected_encoded_address = b"bc1qqvdcf32k0vfxgsyet5ldt246q4jaw8scx3sysx0lnstlt6w4m5rsgej0cd";

      assert!(encoded_address == expected_encoded_address, 1002);
   }

   #[test]
   /// This test is verified at https://www.blockchain.com/explorer/addresses/btc/bc1qq302rl92lgujmvlk5h6pf63zn6wekcjql6cmum.
   fun test_bech32_encoding_to_p2wpkh_address() {
      let public_key = x"045ea1fcaafa392db3f6a5f414ea229e9d9b6240"; // 20-bytes public key for a Bech32 (P2WPKH) address
      let version = 0; // version 0 for a Bech32 (P2WPKH) address
      
      let encoded_address = bech32(&public_key, version);
      let expected_encoded_address = b"bc1qq302rl92lgujmvlk5h6pf63zn6wekcjql6cmum";

      assert!(encoded_address == expected_encoded_address, 1003);
   }

   #[test]
   /// This test is verified at https://www.blockchain.com/explorer/addresses/btc/bc1pqvdcf32k0vfxgsyet5ldt246q4jaw8scx3sysx0lnstlt6w4m5rszwjxq3.
   fun test_bech32_encoding_to_p2tr_address() {
      let public_key = x"031b84c5567b126440995d3ed5aaba0565d71e1834604819ff9c17f5e9d5dd07"; // 32-bytes public key for a Bech32 (Taproot) address
      let version = 1; // version 1 for a Bech32 (Taproot) address
      
      let encoded_address = bech32(&public_key, version);
      let expected_encoded_address = b"bc1pqvdcf32k0vfxgsyet5ldt246q4jaw8scx3sysx0lnstlt6w4m5rszwjxq3";

      assert!(encoded_address == expected_encoded_address, 1004);
   }

    // Test function for P2SH address generation
    #[test]
    /// This test is verified at https://www.blockchain.com/explorer/addresses/btc/3FrvSxCNmbGbYxvaBY9rhTWKauCXWPpLh9.
    fun test_p2sh_address() {
        let public_key = x"0345c567b17d2e69c8b91b3503f0fb50ebb88f1806e9133f62d3d3501efcedf8d3"; // 33-byte compressed public key
        let p2sh_address = p2sh(&public_key);

        let expected_p2sh_address = b"3FrvSxCNmbGbYxvaBY9rhTWKauCXWPpLh9"; // Sample P2SH address
        
        assert!(p2sh_address == expected_p2sh_address, 1005);
    }

    // Test function for P2PKH address generation
    #[test]
    /// This test is verified at https://www.blockchain.com/explorer/addresses/btc/1Dnv7vKt2JEK7tQnbmYBq3Sfuckz7wFS6e.
    fun test_p2pkh_address() {
        let public_key = x"03a819b6f0eb5f22167fffa53e1628cfbf645db9a4c50b3a226e5d20c9984e63a2"; // 33-byte compressed public key
        let p2pkh_address = p2pkh(&public_key);

        let expected_p2pkh_address = b"1Dnv7vKt2JEK7tQnbmYBq3Sfuckz7wFS6e"; // Sample P2PKH address
        
        assert!(p2pkh_address == expected_p2pkh_address, 1006);
    }
}
