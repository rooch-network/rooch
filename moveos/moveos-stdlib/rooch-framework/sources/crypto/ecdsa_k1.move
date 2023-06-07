module rooch_framework::ecdsa_k1 {

   /// Error if the public key cannot be recovered from the signature.
   const EFailToRecoverPubKey: u64 = 0;

   /// Error if the signature is invalid.
   const EInvalidSignature: u64 = 1;

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
   /// applied to Secp256k1 signatures.
   public native fun ecrecover(signature: &vector<u8>, msg: &vector<u8>, hash: u8): vector<u8>;

   /// @param pubkey: A 33-bytes compressed public key, a prefix either 0x02 or 0x03 and a 256-bit integer.
   ///
   /// If the compressed public key is valid, return the 65-bytes uncompressed public key,
   /// otherwise throw error.
   public native fun decompress_pubkey(pubkey: &vector<u8>): vector<u8>;

   /// @param signature: A 64-bytes signature in form (r, s) that is signed using
   /// Secp256k1. This is an non-recoverable signature without recovery id.
   /// @param public_key: The public key to verify the signature against
   /// @param msg: The message that the signature is signed against, this is raw message without hashing.
   /// @param hash: The hash function used to hash the message when signing.
   ///
   /// If the signature is valid to the pubkey and hashed message, return true. Else false.
   public native fun verify(
      signature: &vector<u8>,
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
      let pubkey = ecrecover(&sig, &msg, 0);
      assert!(pubkey == pubkey_bytes, 0);

      // recover with sha256 hash
      let sig = x"e5847245b38548547f613aaea3421ad47f5b95a222366fb9f9b8c57568feb19c7077fc31e7d83e00acc1347d08c3e1ad50a4eeb6ab044f25c861ddc7be5b8f9f01";
      let pubkey_bytes = x"02337cca2171fdbfcfd657fa59881f46269f1e590b5ffab6023686c7ad2ecc2c1c";
      let pubkey = ecrecover(&sig, &msg, 1);
      assert!(pubkey == pubkey_bytes, 0);
   }

   // TODO: test

   #[test]
   #[expected_failure(abort_code = EFailToRecoverPubKey)]
   fun test_ecrecover_pubkey_fail_to_recover() {
      let msg = x"00";
      let sig = x"0000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000";
      ecrecover(&sig, &msg, 1);
   }

   #[test]
   #[expected_failure(abort_code = EInvalidSignature)]
   fun test_ecrecover_pubkey_invalid_sig() {
      let msg = b"Hello, world!";
      // incorrect length sig
      let sig = x"7e4237ebfbc36613e166bfc5f6229360a9c1949242da97ca04867e4de57b2df30c8340bcb320328cf46d71bda51fcb519e3ce53b348eec62de852e350edbd886";
      ecrecover(&sig, &msg, 1);
   }

   #[test]
   fun test_verify_fails_with_recoverable_sig() {

   }

   #[test]
   fun test_verify_success_with_nonrecoverable_sig() {

   }

   #[test]
   fun test_secp256k1_invalid() {

   }

   #[test]
   fun test_ecrecover_eth_address() {

   }

   // Helper Move function to recover signature directly to an ETH address.
   fun ecrecover_eth_address(sig: vector<u8>, msg: vector<u8>): vector<u8> {
      use std::vector;
      use rooch_framework::rooch_hash;

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
      let hashed = rooch_hash::keccak256(&uncompressed_64);
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
