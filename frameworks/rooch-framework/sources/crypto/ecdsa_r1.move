// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

module rooch_framework::ecdsa_r1 {
    use std::vector;

    /// Compressed public key length for P-256
    const ECDSA_R1_COMPRESSED_PUBKEY_LENGTH: u64 = 33;
    /// Signature length (r, s)
    const ECDSA_R1_RAW_SIGNATURE_LENGTH: u64 = 64;

    // Error codes
    const ErrorInvalidSignature: u64 = 1;
    const ErrorInvalidPubKey: u64 = 2;
    const ErrorInvalidHashType: u64 = 3;

    // Hash type
    const HASH_TYPE_SHA256: u8 = 1;

    /// Verifies an ECDSA signature over the secp256r1 (P-256) curve.
    /// The message will be hashed with SHA256 before verification.
    public fun verify(
        signature: &vector<u8>,
        public_key: &vector<u8>,
        msg: &vector<u8>
    ): bool {
        assert!(vector::length(signature) == ECDSA_R1_RAW_SIGNATURE_LENGTH, ErrorInvalidSignature);
        assert!(vector::length(public_key) == ECDSA_R1_COMPRESSED_PUBKEY_LENGTH, ErrorInvalidPubKey);
        native_verify(signature, public_key, msg, HASH_TYPE_SHA256)
    }

    native fun native_verify(
        signature: &vector<u8>,
        public_key: &vector<u8>,
        msg: &vector<u8>,
        hash_type: u8
    ): bool;

    public fun public_key_length(): u64 {
        ECDSA_R1_COMPRESSED_PUBKEY_LENGTH
    }

    public fun raw_signature_length(): u64 {
        ECDSA_R1_RAW_SIGNATURE_LENGTH
    }

    #[test]
    fun test_verify_success() {
        // Test case with valid signature
        let msg = b"hello world";
        // This is a valid P-256 public key
        let pubkey = x"0258a618066814098f8ddb3cbde73838b59028d843958031e50be0a5f4b0a9796d";
        // This is a valid P-256 signature for the message
        let sig = x"74133905657c1992d8d6bd72ffa7ccf8d2adf3e4a3ca25f8dc8eec175752cb5a40459f71b549a25cba3cddf4157e946bbff7b18fc82774e9c4c54e362b97ccb5";
        let result = verify(&sig, &pubkey, &msg);
        assert!(result, 0);
    }

    #[test]
    #[expected_failure(location=Self, abort_code = ErrorInvalidSignature)]
    fun test_verify_fails_invalid_sig() {
        // Test case with invalid signature length
        let msg = b"hello world";
        let pubkey = x"0258a618066814098f8ddb3cbde73838b59028d843958031e50be0a5f4b0a9796d";
        let sig = x"0000"; // Invalid length
        verify(&sig, &pubkey, &msg);
    }

    #[test]
    #[expected_failure(location=Self, abort_code = ErrorInvalidPubKey)]
    fun test_verify_fails_invalid_pubkey() {
        // Test case with invalid public key length
        let msg = b"hello world";
        let pubkey = x"0000"; // Invalid length
        let sig = x"74133905657c1992d8d6bd72ffa7ccf8d2adf3e4a3ca25f8dc8eec175752cb5a40459f71b549a25cba3cddf4157e946bbff7b18fc82774e9c4c54e362b97ccb5";
        verify(&sig, &pubkey, &msg);
    }

    #[test]
    fun test_verify_with_different_message() {
        // Test case with different message
        let msg = b"different message";
        let pubkey = x"0258a618066814098f8ddb3cbde73838b59028d843958031e50be0a5f4b0a9796d";
        let sig = x"74133905657c1992d8d6bd72ffa7ccf8d2adf3e4a3ca25f8dc8eec175752cb5a40459f71b549a25cba3cddf4157e946bbff7b18fc82774e9c4c54e362b97ccb5";
        let result = verify(&sig, &pubkey, &msg);
        assert!(!result, 0);
    }

    #[test]
    fun test_verify_with_empty_message() {
        // Test case with empty message
        let msg = b"";
        let pubkey = x"0258a618066814098f8ddb3cbde73838b59028d843958031e50be0a5f4b0a9796d";
        let sig = x"74133905657c1992d8d6bd72ffa7ccf8d2adf3e4a3ca25f8dc8eec175752cb5a40459f71b549a25cba3cddf4157e946bbff7b18fc82774e9c4c54e362b97ccb5";
        let result = verify(&sig, &pubkey, &msg);
        assert!(!result, 0);
    }

    #[test]
    fun test_verify_with_long_message() {
        // Test case with a longer message
        let msg = x"000102030405060708090a0b0c0d0e0f101112131415161718191a1b1c1d1e1f202122232425262728292a2b2c2d2e2f303132333435363738393a3b3c3d3e3f";
        let pubkey = x"0258a618066814098f8ddb3cbde73838b59028d843958031e50be0a5f4b0a9796d";
        let sig = x"74133905657c1992d8d6bd72ffa7ccf8d2adf3e4a3ca25f8dc8eec175752cb5a40459f71b549a25cba3cddf4157e946bbff7b18fc82774e9c4c54e362b97ccb5";
        let result = verify(&sig, &pubkey, &msg);
        assert!(!result, 0);
    }

} 