module rooch_framework::ecdsa_k1 {

   /// Error if the public key cannot be recovered from the signature.
   const EFailToRecoverPubKey: u64 = 0;

   /// Error if the signature is invalid.
   const EInvalidSignature: u64 = 1;

   /// Error if the public key is invalid.
   const EInvalidPubKey: u64 = 2;

   /// Hash function name that are valid for ecrecover and verify.
   const KECCAK256: u8 = 0;
   const SHA256: u8 = 1;

   /// @param signature: A 65-bytes signature in form (r, s, v) that is signed using
   /// The accepted v values are {0, 1, 2, 3}.
   /// @param msg: The message that the signature is signed against, this is raw message without hashing.
   /// @param hash: The hash function used to hash the message when signing.
   ///
   /// If the signature is valid, return the corresponding recovered Secpk256k1 public
   /// key, otherwise throw error. This is similar to ecrecover in Ethereum, can only be
   /// applied to Ecdsa signatures.
   public native fun ecrecover(signature: &vector<u8>, msg: &vector<u8>, hash: u8): vector<u8>;

   /// @param pubkey: A 33-bytes compressed public key, a prefix either 0x02 or 0x03 and a 256-bit integer.
   ///
   /// If the compressed public key is valid, return the 65-bytes uncompressed public key,
   /// otherwise throw error.
   public native fun decompress_pubkey(pubkey: &vector<u8>): vector<u8>;

   /// @param signature: A 65-bytes signature in form (r, s, v) that is signed using
   /// Ecdsa. This is a recoverable signature with a recovery id.
   /// @param msg: The message that the signature is signed against.
   /// @param hash: The hash function used to hash the message when signing.
   ///
   /// If the signature is valid to the pubkey and hashed message, return true. Else false.
   public native fun verify_recoverable(
      signature: &vector<u8>,
      msg: &vector<u8>,
      hash: u8
   ): bool;

   /// @param signature: A 64-bytes signature in form (r, s, v) that is signed using
   /// Ecdsa. This is an non-recoverable signature without recovery id.
   /// @param public_key: A 33-bytes public key that is used to sign messages.
   /// @param msg: The message that the signature is signed against.
   /// @param hash: The hash function used to hash the message when signing.
   ///
   /// If the signature is valid to the pubkey and hashed message, return true. Else false.
   public native fun verify_nonrecoverable(
      signature: &vector<u8>,
      public_key: &vector<u8>,
      msg: &vector<u8>,
      hash: u8
   ): bool;

   #[test]
   fun test_ecrecover_pubkey() {
      // test case generated against https://github.com/MystenLabs/fastcrypto/blob/f9e64dc028040f863a53a6a88072bda71abd9946/fastcrypto/src/tests/secp256k1_recoverable_tests.rs
      let msg = b"Hello, world!";

      // recover with keccak256 hash
      let sig = x"7e4237ebfbc36613e166bfc5f6229360a9c1949242da97ca04867e4de57b2df30c8340bcb320328cf46d71bda51fcb519e3ce53b348eec62de852e350edbd88600";
      let pubkey_bytes = x"02337cca2171fdbfcfd657fa59881f46269f1e590b5ffab6023686c7ad2ecc2c1c";
      let pubkey = ecrecover(&sig, &msg, KECCAK256);
      assert!(pubkey == pubkey_bytes, 0);

      // recover with sha256 hash
      let sig = x"e5847245b38548547f613aaea3421ad47f5b95a222366fb9f9b8c57568feb19c7077fc31e7d83e00acc1347d08c3e1ad50a4eeb6ab044f25c861ddc7be5b8f9f01";
      let pubkey_bytes = x"02337cca2171fdbfcfd657fa59881f46269f1e590b5ffab6023686c7ad2ecc2c1c";
      let pubkey = ecrecover(&sig, &msg, SHA256);
      assert!(pubkey == pubkey_bytes, 0);
   }

   #[test]
   #[expected_failure(abort_code = EFailToRecoverPubKey)]
   fun test_ecrecover_pubkey_fail_to_recover() {
      let msg = x"00";
      let sig = x"0000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000";
      ecrecover(&sig, &msg, SHA256);
   }

   #[test]
   #[expected_failure(abort_code = EInvalidSignature)]
   fun test_ecrecover_pubkey_invalid_sig() {
      let msg = b"Hello, world!";
      // incorrect length sig
      let sig = x"7e4237ebfbc36613e166bfc5f6229360a9c1949242da97ca04867e4de57b2df30c8340bcb320328cf46d71bda51fcb519e3ce53b348eec62de852e350edbd886";
      ecrecover(&sig, &msg, SHA256);
   }

   #[test]
   #[expected_failure(abort_code = EInvalidSignature)]
   fun test_verify_fails_with_recoverable_invalid_sig() {
      let msg = b"Hello, world!";
      let sig = x"";
      verify_recoverable(&sig, &msg, KECCAK256);
   }

   #[test]
   fun test_verify_success_with_recoverable() {
      let msg = b"Hello, world!";
      let sig = x"7e4237ebfbc36613e166bfc5f6229360a9c1949242da97ca04867e4de57b2df30c8340bcb320328cf46d71bda51fcb519e3ce53b348eec62de852e350edbd88600";
      let verify = verify_recoverable(&sig, &msg, KECCAK256);
      assert!(verify, 0)
   }

   #[test]
   fun test_verify_success_with_nonrecoverable() {
      let msg = x"00010203";
      let pubkey = x"033e99a541db69bd32040dfe5037fbf5210dafa8151a71e21c5204b05d95ce0a62";
      let sig = x"416a21d50b3c838328d4f03213f8ef0c3776389a972ba1ecd37b56243734eba208ea6aaa6fc076ad7accd71d355f693a6fe54fe69b3c168eace9803827bc9046";
      let verify = verify_nonrecoverable(&sig, &pubkey, &msg, SHA256);
      assert!(verify, 0);
   }

   #[test]
   #[expected_failure(abort_code = EInvalidSignature)]
   fun test_verify_fails_with_nonrecoverable_invalid_sig() {
      let msg = x"00010203";
      let pubkey = x"033e99a541db69bd32040dfe5037fbf5210dafa8151a71e21c5204b05d95ce0a62";
      let sig = x"";
      verify_nonrecoverable(&sig, &pubkey, &msg, SHA256);
   }

   #[test]
   #[expected_failure(abort_code = EInvalidPubKey)]
   fun test_verify_fails_with_nonrecoverable_invalid_pubkey() {
      let msg = x"00010203";
      let pubkey = x"";
      let sig = x"416a21d50b3c838328d4f03213f8ef0c3776389a972ba1ecd37b56243734eba208ea6aaa6fc076ad7accd71d355f693a6fe54fe69b3c168eace9803827bc9046";
      verify_nonrecoverable(&sig, &pubkey, &msg, SHA256);
   }

   #[test]
   fun test_decompress_pubkey() {
      let pubkey = x"033e99a541db69bd32040dfe5037fbf5210dafa8151a71e21c5204b05d95ce0a62";
      assert!(std::vector::length(&pubkey) == 33, 0);
      let pubkey_decompressed = decompress_pubkey(&pubkey);
      assert!(std::vector::length(&pubkey_decompressed) == 65, 0);
   }

   #[test]
   #[expected_failure(abort_code = EInvalidPubKey)]
   fun test_decompress_pubkey_invalid_pubkey() {
      let pubkey = x"013e99a541db69bd32040dfe5037fbf5210dafa8151a71e21c5204b05d95ce0a62";
      decompress_pubkey(&pubkey);
   }

   #[test]
   fun test_ecrecover_eth_address() {
      // recover with keccak256 hash from ecrecover_eth_address function
      let msg = b"Hello, world!";
      let sig = x"e5847245b38548547f613aaea3421ad47f5b95a222366fb9f9b8c57568feb19c7077fc31e7d83e00acc1347d08c3e1ad50a4eeb6ab044f25c861ddc7be5b8f9f01";
      let eth_address = x"4259abf3f34ab0e5a399494cb1e9a7f8465ae4d6";
      let addr = ecrecover_eth_address(sig, msg);
      assert!(addr == eth_address, 0);
   }

   // Helper Move function to recover signature directly to an ETH address.
   fun ecrecover_eth_address(sig: vector<u8>, msg: vector<u8>): vector<u8> {
      use std::vector;
      use rooch_framework::hash;

      // Normalize the last byte of the signature to be 0 or 1.
      let v = vector::borrow_mut(&mut sig, 64);
      if (*v == 27) {
         *v = 0;
      } else if (*v == 28) {
         *v = 1;
      } else if (*v > 35) {
         *v = (*v - 1) % 2;
      };

      let pubkey = ecrecover(&sig, &msg, 0);

      let uncompressed = decompress_pubkey(&pubkey);

      // Take the last 64 bytes of the uncompressed pubkey.
      let uncompressed_64 = vector::empty<u8>();
      let i = 1;
      while (i < 65) {
         let value = vector::borrow(&uncompressed, i);
         vector::push_back(&mut uncompressed_64, *value);
         i = i + 1;
      };

      // Take the last 20 bytes of the hash of the 64-bytes uncompressed pubkey.
      let hashed = hash::keccak256(&uncompressed_64);
      let addr = vector::empty<u8>();
      let i = 12;
      while (i < 32) {
         let value = vector::borrow(&hashed, i);
         vector::push_back(&mut addr, *value);
         i = i + 1;
      };

      addr
   }
}
