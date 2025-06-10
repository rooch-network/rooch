// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

module rooch_framework::ecdsa_r1 {
    /// Compressed public key length for P-256
    const ECDSA_R1_COMPRESSED_PUBKEY_LENGTH: u64 = 33;
    /// Signature length (r, s)
    const ECDSA_R1_SIGNATURE_LENGTH: u64 = 64;

    // Error codes
    const ErrorInvalidSignature: u64 = 1;
    const ErrorInvalidPubKey: u64 = 2;

    /// Verifies an ECDSA signature over the secp256r1 (P-256) curve.
    /// The message is hashed with SHA256 before verification.
    native public fun verify(
        signature: &vector<u8>,
        public_key: &vector<u8>,
        msg: &vector<u8>
    ): bool;

    public fun public_key_length(): u64 {
        ECDSA_R1_COMPRESSED_PUBKEY_LENGTH
    }

    public fun signature_length(): u64 {
        ECDSA_R1_SIGNATURE_LENGTH
    }

    #[test]
    fun test_verify_success() {
        // Test case with valid signature
        let msg = x"00010203";
        let pubkey = x"03a0434d9e47f3c86235477c7b1ae6ae5d3442d49b1943c2b752a68e2a47e247c7";
        let sig = x"2a298dacae57395a15d0795ddbfd1dcb564da82b0f269bc70a74f8220429ba1d1e51a22cce81171efdc496b0a2d19d2c1dec3b7f98f0215bf6f2535b4ab8976c";
        let result = verify(&sig, &pubkey, &msg);
        assert!(result, 0);
    }

    #[test]
    #[expected_failure(location=Self, abort_code = ErrorInvalidSignature)]
    fun test_verify_fails_invalid_sig() {
        // Test case with invalid signature
        let msg = x"00010203";
        let pubkey = x"03a0434d9e47f3c86235477c7b1ae6ae5d3442d49b1943c2b752a68e2a47e247c7";
        let sig = x"00000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000";
        verify(&sig, &pubkey, &msg);
    }

    #[test]
    #[expected_failure(location=Self, abort_code = ErrorInvalidPubKey)]
    fun test_verify_fails_invalid_pubkey() {
        // Test case with invalid public key
        let msg = x"00010203";
        let pubkey = x"000000000000000000000000000000000000000000000000000000000000000000";
        let sig = x"2a298dacae57395a15d0795ddbfd1dcb564da82b0f269bc70a74f8220429ba1d1e51a22cce81171efdc496b0a2d19d2c1dec3b7f98f0215bf6f2535b4ab8976c";
        verify(&sig, &pubkey, &msg);
    }

    #[test]
    fun test_verify_with_different_message() {
        // Test case with different message
        let msg = x"deadbeef";
        let pubkey = x"03a0434d9e47f3c86235477c7b1ae6ae5d3442d49b1943c2b752a68e2a47e247c7";
        let sig = x"2a298dacae57395a15d0795ddbfd1dcb564da82b0f269bc70a74f8220429ba1d1e51a22cce81171efdc496b0a2d19d2c1dec3b7f98f0215bf6f2535b4ab8976c";
        let result = verify(&sig, &pubkey, &msg);
        assert!(!result, 0);
    }

    #[test]
    fun test_public_key_length() {
        assert!(public_key_length() == ECDSA_R1_COMPRESSED_PUBKEY_LENGTH, 0);
    }

    #[test]
    fun test_signature_length() {
        assert!(signature_length() == ECDSA_R1_SIGNATURE_LENGTH, 0);
    }
} 