// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

module rooch_framework::schnorr {
    /// constant codes
    const SCHNORR_PUBKEY_LENGTH: u64 = 32;
    const SCHNORR_SIG_LENGTH: u64 = 64;

    /// Hash function name that are valid for verify.
    const SHA256: u8 = 0;

    /// Error if the signature is invalid.
    const ErrorInvalidSignature: u64 = 1;

    /// Error if the public key is invalid.
    const ErrorInvalidPubKey: u64 = 2;

    /// built-in functions
    public fun public_key_length(): u64 {
        SCHNORR_PUBKEY_LENGTH
    }

    public fun signature_length(): u64 {
        SCHNORR_SIG_LENGTH
    }

    public fun sha256(): u8 {
        SHA256
    }

    /// @param signature: A 64-bytes signature that is signed using Schnorr over Secpk256k1 key pairs.
    /// @param public_key: A 32-bytes public key that is used to sign messages.
    /// @param msg: The message that the signature is signed against.
    /// @param hash: The hash function used to hash the message when signing.
    ///
    /// If the signature is valid to the pubkey and hashed message, return true. Else false.
    native public fun verify(
        signature: &vector<u8>,
        public_key: &vector<u8>,
        msg: &vector<u8>,
        hash: u8
    ): bool;

    #[test]
    public fun test_schnorr_invalid_case() {
        let malformed_msg = x"00010203";
        let vk = x"cddcc4a1d4a94d627e7808f904d0477cf16ae9d4fafa1eb883ab7a498bdda777";
        let sig = x"6c2565ceabff153609aa9ccdeb13421a1181a54d0ca4fe10cd074b0c2da44c641c98992701c9a4d3e24391db3e358eff190510be46e73d0e517d5e5b13bb06fd";

        let result = verify(&sig, &vk, &malformed_msg);
        assert!(!result, 0)
    }

    #[test]
    #[expected_failure(location=Self, abort_code = ErrorInvalidVerifyingKey)]
    public fun test_schnorr_invalid_verifying_key() {
        let msg = x"f08285dc969c9cdfa65a5a29dc592371acb80534ae301965f38b0583817ea33f";
        let invalid_vk = x"5e99a541db69bd32040dfe5037fbf5210dafa8151a71e21c5204b05d95ce0a62";
        let sig = x"6c2565ceabff153609aa9ccdeb13421a1181a54d0ca4fe10cd074b0c2da44c641c98992701c9a4d3e24391db3e358eff190510be46e73d0e517d5e5b13bb06fd";

        verify(&sig, &invalid_pk, &msg, SHA256);
    }

    #[test]
    #[expected_failure(location=Self, abort_code = ErrorInvalidSignature)]
    public fun test_schnorr_invalid_signature() {
        let msg = x"f08285dc969c9cdfa65a5a29dc592371acb80534ae301965f38b0583817ea33f";
        let vk = x"cddcc4a1d4a94d627e7808f904d0477cf16ae9d4fafa1eb883ab7a498bdda777";
        let invalid_sig = x"0c2565ceabff153609aa9ccdeb13421a1181a54d0ca4fe10cd074b0c2da44c641c98992701c9a4d3e24391db3e358eff190510be46e73d0e517d5e5b13bb06fd12";

        verify(&invalid_sig, &vk, &msg);
    }

    #[test]
    public fun test_schnorr_valid_case() {
        let msg = x"f08285dc969c9cdfa65a5a29dc592371acb80534ae301965f38b0583817ea33f";
        let vk = x"cddcc4a1d4a94d627e7808f904d0477cf16ae9d4fafa1eb883ab7a498bdda777";
        let sig = x"6c2565ceabff153609aa9ccdeb13421a1181a54d0ca4fe10cd074b0c2da44c641c98992701c9a4d3e24391db3e358eff190510be46e73d0e517d5e5b13bb06fd";

        let result = verify(&sig, &vk, &msg);
        assert!(result, 1)
    }
}
