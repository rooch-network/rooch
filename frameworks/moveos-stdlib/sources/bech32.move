// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

/// Module which defines bech32 functions.
module moveos_std::bech32 {
   #[test_only]
   use moveos_std::bcs;

   const BIP350:u16 = 350; // bip350 is bech32m

   // Encode failed error
   const E_ENCODE_FAILED: u64 = 1;

   // Decode failed error
   const E_DECODE_FAILED: u64 = 2;

   // Invalid BIP code error
   const E_INVALID_BIP_CODE: u64 = 3;

   // Invalid bitcoin network
   const E_INVALID_NETWORK: u64 = 4;

   // Invalid witness version
   const E_INVALID_WITNESS_VERSION: u64 = 5;

   /// @param hrp: human-readable part in string
   /// @param data: arbitrary data to be encoded.
   /// Encode arbitrary data using string as the human-readable part and append a bech32 checksum.
   native public fun encode(bip: u16, hrp: vector<u8>, data: vector<u8>): vector<u8>;

   /// @param network: network to be selected, i.e. bc, tb, or bcrt
   /// @param witness_version: segwit witness version. 0 for bech32, 1 for bech32m and taproot, and 2-16 is included.
   /// @param data: arbitrary data to be encoded.
   /// Encode arbitrary data to a Bitcoin address using string as the network, number as the witness version
   native public fun segwit_encode(network: vector<u8>, witness_version: u8, data: vector<u8>): vector<u8>;

   /// @param hrp: human-readable part bytes to be used as a decoding input
   /// @param encoded: encoded bytes to be decoded as data
   /// Decode a bech32 encoded string that includes a bech32 checksum.
   native public fun decode(hrp: vector<u8>, encoded: vector<u8>): vector<u8>;

   /// @param hrp: human-readable part bytes to be used as a decoding input
   /// @param witness_ascii_version: segwit witness ASCII version to be used as a decoding input
   /// @param encoded: encoded bytes to be decoded as data
   /// Decode an encoded Bitcoin address
   native public fun segwit_decode(hrp: vector<u8>, witness_ascii_version: u8, encoded: vector<u8>): vector<u8>;

   public fun bech32m_to_bip() : u16 {
      BIP350
   }

   // Test succeeded with https://slowli.github.io/bech32-buffer/ on Data
   #[test]
   fun test_encode_bech32() {
      let bip = 173; // bip173 is bech32
      let hrp = b"bc"; // hrp "bc" for bitcoin mainnet
      let data = x"0279be667ef9dcbbac55a06295ce870b07029bfcdb2dce28d959f2815b16f81798"; // 32-bytes data with 02 prefix for a Bech32 checksum: https://en.bitcoin.it/wiki/Bech32

      let encoded = encode(bip, hrp, data);
      let expected_encoded = b"bc1qfumuen7l8wthtz45p3ftn58pvrs9xlumvkuu2xet8egzkcklqtesjm9aq0"; // https://mempool.space/address/bc1qfumuen7l8wthtz45p3ftn58pvrs9xlumvkuu2xet8egzkcklqtesjm9aq0

      assert!(encoded == expected_encoded, E_ENCODE_FAILED);
   }

   // Test succeeded with https://slowli.github.io/bech32-buffer/ on Data
   #[test]
   fun test_encode_bech32m() {
      let bip = 350; // bip350 is bech32m
      let hrp = b"rooch"; // hrp "rooch" for custom human-readable part
      let data = x"b19ced819df50b20648cfcabb75c4f216c77991f36d6"; // 22-bytes data for a Bech32m checksum

      let encoded = encode(bip, hrp, data);
      let expected_encoded = b"rooch1kxwwmqva759jqeyvlj4mwhz0y9k80xglxmtqxdfdl5";

      assert!(encoded == expected_encoded, E_ENCODE_FAILED);
   }

   #[test]
   fun test_encode_no_checksum() {
      let bip = 0; // bip0 is no checksum
      let hrp = b"rooch"; // hrp "rooch" for custom human-readable part
      let data = x"45348f7e62331ace696702f2910e32efceb601a341644d4fac07"; // 26-bytes data for no checksum

      let encoded = encode(bip, hrp, data);
      let expected_encoded = b"rooch1g56g7lnzxvdvu6t8qtefzr3jal8tvqdrg9jy6navqu";

      assert!(encoded == expected_encoded, E_ENCODE_FAILED);
   }

   #[test]
   #[expected_failure(abort_code = E_INVALID_BIP_CODE, location = Self)]
   fun test_encode_failed_bip() {
      let bip = 1; // bip1 is invalid
      let hrp = b"rooch";
      let data = x"e91c57466fba2b572c57bafbb9a2de49d3bb09e12aaf031d";

      encode(bip, hrp, data);
   }

   #[test]
   fun test_segwit_encode() {
      let network = b"tb"; // tb is testnet network
      let witness_version = 1; // 1 is taproot address version
      let data = x"41876e7b954162eea060743e8489df36454a50eddd60046c85ac4e67a88ef5fc3320321cd79bee"; // 39-bytes data for encoding to a bitcoin address

      let encoded = segwit_encode(network, witness_version, data);
      let expected_encoded = b"tb1pgxrku7u4g93wagrqwslgfzwlxez5558dm4sqgmy9438x02yw7h7rxgpjrntehmsj7d5je";

      assert!(encoded == expected_encoded, E_ENCODE_FAILED);
   }

   #[test]
   fun test_decode_bech32() {
      let hrp = b"bc"; // hrp "bc" for bitcoin mainnet
      let encoded = b"bc1qfumuen7l8wthtz45p3ftn58pvrs9xlumvkuu2xet8egzkcklqtesjm9aq0"; // https://mempool.space/address/bc1qfumuen7l8wthtz45p3ftn58pvrs9xlumvkuu2xet8egzkcklqtesjm9aq0
      let decoded = decode(hrp, encoded);
      let expected_decoded = x"0279be667ef9dcbbac55a06295ce870b07029bfcdb2dce28d959f2815b16f81798"; // 32-bytes data with 02 prefix for a Bech32 checksum (P2WSH): https://en.bitcoin.it/wiki/Bech32
      assert!(decoded == expected_decoded, E_DECODE_FAILED);
   }

   // Test succeeded with https://slowli.github.io/bech32-buffer/ on Data
   #[test]
   fun test_decode_bech32m() {
      let hrp = b"rooch"; // hrp "rooch" for custom human-readable part
      let encoded = b"rooch1kxwwmqva759jqeyvlj4mwhz0y9k80xglxmtqxdfdl5";
      let decoded = decode(hrp, encoded);
      let expected_decoded = x"b19ced819df50b20648cfcabb75c4f216c77991f36d6";
      assert!(decoded == expected_decoded, E_DECODE_FAILED);
   }

   // TODO: since no valid non-checksum decoding method was present for no checksummed string, it will result in failure when decoding strings with no checksum
   #[test]
   #[expected_failure(abort_code = E_DECODE_FAILED, location = Self)]
   fun test_decode_no_checksum() {
      let hrp = b"rooch"; // hrp "rooch" for custom human-readable part
      let encoded = b"rooch1g56g7lnzxvdvu6t8qtefzr3jal8tvqdrg9jy6navqu";
      decode(hrp, encoded);
   }

   #[test]
   fun test_segwit_decode() {
      let hrp = b"tb"; // tb is testnet network
      let witness_ascii_version = 112; // 112 is p's ASCII value
      let encoded = b"tb1pgxrku7u4g93wagrqwslgfzwlxez5558dm4sqgmy9438x02yw7h7rxgpjrntehmsj7d5je";

      let decoded = segwit_decode(hrp, witness_ascii_version, encoded);
      let expected_decoded = x"41876e7b954162eea060743e8489df36454a50eddd60046c85ac4e67a88ef5fc3320321cd79bee";

      assert!(decoded == expected_decoded, E_DECODE_FAILED);
   }

   #[test]
   fun test_rooch_address(){
      let rooch_addr_str = b"rooch10lnft7hhq37vl0y97lwvkmzqt48fk76y0z88rfcu8zg6qm8qegfqx0rq2h";
      let data = decode(b"rooch", rooch_addr_str);
      assert!(moveos_std::bcs::to_address(data) == @0x7fe695faf7047ccfbc85f7dccb6c405d4e9b7b44788e71a71c3891a06ce0ca12, 1000);
   }

   #[test]
   fun test_address_bech32() {
      let bip = 350; // bip350 is bech32m
      let hrp = b"rooch"; // hrp "rooch" for custom human-readable part
      let addr = @0x42;
      let data = bcs::to_bytes(&addr);
      let encoded = encode(bip, hrp, data);
      let expected_encoded = b"rooch1qqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqppq6exstd";
      assert!(encoded == expected_encoded, 1001);

      let decode_data = decode(b"rooch", encoded);
      let address_from = moveos_std::bcs::to_address(decode_data);
      assert!(addr == address_from, 1002);
   }
}
