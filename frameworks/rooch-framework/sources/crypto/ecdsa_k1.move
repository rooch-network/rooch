// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

// TODO: rename ecdsa_k1 to secp256k1 due to signature types change
module rooch_framework::ecdsa_k1 {

    /// constant codes
    const ECDSA_K1_COMPRESSED_PUBKEY_LENGTH: u64 = 33;
    const ECDSA_K1_SIG_LENGTH: u64 = 64;
    const ECDSA_K1_UNCOMPRESSED_PUBKEY_LENGTH: u64 = 65;

    /// Hash function name that are valid for ecrecover and verify.
    const KECCAK256: u8 = 0;
    const SHA256: u8 = 1;

    /// Signature types that are valid for verify.
    const ECDSA: u8 = 0;
    const SCHNORR: u8 = 1;

    /// Error if the public key cannot be recovered from the signature.
    const ErrorFailToRecoverPubKey: u64 = 1;

    /// Error if the signature is invalid.
    const ErrorInvalidSignature: u64 = 2;

    /// Error if the public key is invalid.
    const ErrorInvalidPubKey: u64 = 3;

    /// Invalid hash function.
    const ErrorInvalidHashType: u64 = 4;

    /// Error if the x only public key is invalid.
    const ErrorInvalidXOnlyPubKey: u64 = 5;

    /// Error if the message is invalid.
    const ErrorInvalidMessage: u64 = 6;

    /// Error if the schnorr signature is invalid.
    const ErrorInvalidSchnorrSignature: u64 = 7;

    /// built-in functions
    public fun public_key_length(): u64 {
        ECDSA_K1_COMPRESSED_PUBKEY_LENGTH
    }

    public fun uncompressed_public_key_length(): u64 {
        ECDSA_K1_UNCOMPRESSED_PUBKEY_LENGTH
    }

    public fun keccak256(): u8 {
        KECCAK256
    }

    public fun sha256(): u8 {
        SHA256
    }

    public fun ecdsa(): u8 {
        ECDSA
    }

    public fun schnorr(): u8 {
        SCHNORR
    }

    /// @param signature: A 65-bytes signature in form (r, s, v) that is signed using
    /// The accepted v values are {0, 1, 2, 3}.
    /// @param msg: The message that the signature is signed against, this is raw message without hashing.
    /// @param hash: The hash function used to hash the message when signing.
    ///
    /// If the signature is valid, return the corresponding recovered Secpk256k1 public
    /// key, otherwise throw error. This is similar to ecrecover in Ethereum, can only be
    /// applied to Ecdsa signatures.
    native public fun ecrecover(signature: &vector<u8>, msg: &vector<u8>, hash: u8): vector<u8>;

    /// @param pubkey: A 33-bytes compressed public key, a prefix either 0x02 or 0x03 and a 256-bit integer.
    ///
    /// If the compressed public key is valid, return the 65-bytes uncompressed public key,
    /// otherwise throw error.
    native public fun decompress_pubkey(pubkey: &vector<u8>): vector<u8>;

    /// @param signature: A 64-bytes signature in form (r, s) that is signed using
    /// Ecdsa. This is an non-recoverable signature without recovery id.
    /// @param public_key: A 33-bytes public key that is used to sign messages.
    /// @param msg: The message that the signature is signed against.
    /// @param hash: The hash function used to hash the message when signing.
    /// TODO: @param sigtype: The signature type used to distinguish which signature to be used when verifying.
    ///
    /// If the signature is valid to the pubkey and hashed message, return true. Else false.
    native public fun verify(
        signature: &vector<u8>,
        public_key: &vector<u8>,
        msg: &vector<u8>,
        hash: u8,
    ): bool;

    #[test]
    fun test_verify_ecdsa_success() {
        let msg = x"00010203";
        let pubkey = x"033e99a541db69bd32040dfe5037fbf5210dafa8151a71e21c5204b05d95ce0a62";
        let sig = x"416a21d50b3c838328d4f03213f8ef0c3776389a972ba1ecd37b56243734eba208ea6aaa6fc076ad7accd71d355f693a6fe54fe69b3c168eace9803827bc9046";
        let result = verify(&sig, &pubkey, &msg, SHA256);
        assert!(result, 0);
    }

    #[test]
    fun test_verify_schnorr_success() {
        let msg = x"f08285dc969c9cdfa65a5a29dc592371acb80534ae301965f38b0583817ea33f";
        let pubkey = x"cddcc4a1d4a94d627e7808f904d0477cf16ae9d4fafa1eb883ab7a498bdda777";
        let sig = x"6c2565ceabff153609aa9ccdeb13421a1181a54d0ca4fe10cd074b0c2da44c641c98992701c9a4d3e24391db3e358eff190510be46e73d0e517d5e5b13bb06fd";
        let result = verify(&sig, &pubkey, &msg, SHA256);
        assert!(result, 1)
    }

    #[test]
    #[expected_failure(location=Self, abort_code = ErrorInvalidSignature)]
    fun test_verify_ecdsa_fails_invalid_sig() {
        let msg = x"00010203";
        let pubkey = x"033e99a541db69bd32040dfe5037fbf5210dafa8151a71e21c5204b05d95ce0a62";
        let sig = x"";
        verify(&sig, &pubkey, &msg, SHA256);
    }

    #[test]
    #[expected_failure(location=Self, abort_code = ErrorInvalidPubKey)]
    fun test_verify_ecdsa_fails_invalid_pubkey() {
        let msg = x"00010203";
        let pubkey = x"133e99a541db69bd32040dfe5037fbf5210dafa8151a71e21c5204b05d95ce0a62";
        let sig = x"416a21d50b3c838328d4f03213f8ef0c3776389a972ba1ecd37b56243734eba208ea6aaa6fc076ad7accd71d355f693a6fe54fe69b3c168eace9803827bc9046";
        verify(&sig, &pubkey, &msg, SHA256);
    }

    #[test]
    #[expected_failure(location=Self, abort_code = ErrorInvalidSchnorrSignature)]
    fun test_verify_schnorr_fails_invalid_sig() {
        let msg = x"f08285dc969c9cdfa65a5a29dc592371acb80534ae301965f38b0583817ea33f";
        let pubkey = x"cddcc4a1d4a94d627e7808f904d0477cf16ae9d4fafa1eb883ab7a498bdda777";
        let sig = x"0c2565ceabff153609aa9ccdeb13421a1181a54d0ca4fe10cd074b0c2da44c641c98992701c9a4d3e24391db3e358eff190510be46e73d0e517d5e5b13bb06fd12";
        verify(&sig, &pubkey, &msg, SHA256);
    }

    #[test]
    #[expected_failure(location=Self, abort_code = ErrorInvalidMessage)]
    fun test_verify_schnorr_fails_invalid_message() {
        let msg = x"00010203";
        let pubkey = x"cddcc4a1d4a94d627e7808f904d0477cf16ae9d4fafa1eb883ab7a498bdda777";
        let sig = x"6c2565ceabff153609aa9ccdeb13421a1181a54d0ca4fe10cd074b0c2da44c641c98992701c9a4d3e24391db3e358eff190510be46e73d0e517d5e5b13bb06fd";
        verify(&sig, &pubkey, &msg, SHA256);
    }

    #[test]
    #[expected_failure(location=Self, abort_code = ErrorInvalidXOnlyPubKey)]
    fun test_verify_schnorr_fails_invalid_x_only_pubkey() {
        let msg = x"f08285dc969c9cdfa65a5a29dc592371acb80534ae301965f38b0583817ea33f";
        let pubkey = x"5e99a541db69bd32040dfe5037fbf5210dafa8151a71e21c5204b05d95ce0a62";
        let sig = x"6c2565ceabff153609aa9ccdeb13421a1181a54d0ca4fe10cd074b0c2da44c641c98992701c9a4d3e24391db3e358eff190510be46e73d0e517d5e5b13bb06fd";
        verify(&sig, &pubkey, &msg, SHA256);
    }

    #[test]
    fun test_ecrecover_pubkey() {
        // test case generated against https://github.com/MystenLabs/fastcrypto/blob/f9e64dc028040f863a53a6a88072bda71abd9946/fastcrypto/src/tests/secp256k1_recoverable_tests.rs
        let msg = b"Hello, world!";

        // recover with keccak256 hash
        let sig = x"7e4237ebfbc36613e166bfc5f6229360a9c1949242da97ca04867e4de57b2df30c8340bcb320328cf46d71bda51fcb519e3ce53b348eec62de852e350edbd88600";
        let pubkey_bytes = x"02337cca2171fdbfcfd657fa59881f46269f1e590b5ffab6023686c7ad2ecc2c1c";
        let pubkey = ecrecover(&sig, &msg, KECCAK256);
        assert!(pubkey == pubkey_bytes, 0);
    }

    #[test]
    #[expected_failure(location=Self, abort_code = ErrorFailToRecoverPubKey)]
    fun test_ecrecover_pubkey_fail_to_recover() {
        let msg = x"00";
        let sig = x"0000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000";
        ecrecover(&sig, &msg, KECCAK256);
    }

    #[test]
    #[expected_failure(location=Self, abort_code = ErrorInvalidSignature)]
    fun test_ecrecover_pubkey_invalid_sig() {
        let msg = b"Hello, world!";
        // incorrect length sig
        let sig = x"7e4237ebfbc36613e166bfc5f6229360a9c1949242da97ca04867e4de57b2df30c8340bcb320328cf46d71bda51fcb519e3ce53b348eec62de852e350edbd886";
        ecrecover(&sig, &msg, KECCAK256);
    }

    #[test]
    fun test_decompress_pubkey() {
        let pubkey = x"033e99a541db69bd32040dfe5037fbf5210dafa8151a71e21c5204b05d95ce0a62";
        assert!(std::vector::length(&pubkey) == 33, 0);
        let pubkey_decompressed = decompress_pubkey(&pubkey);
        assert!(std::vector::length(&pubkey_decompressed) == 65, 0);
    }

    #[test]
    #[expected_failure(location=Self, abort_code = ErrorInvalidPubKey)]
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
        use moveos_std::hash;

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
