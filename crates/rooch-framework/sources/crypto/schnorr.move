module rooch_framework::schnorr {
    use std::vector;

    /// constant codes
    const VALID_SCHNORR_TO_SCHEME_NOSTR_LENGTH: u64 = 1;
    const VALID_SCHNORR_PUBKEY_LENGTH: u64 = 32;
    const VALID_SCHNORR_SIG_LENGTH: u64 = 64;

    /// Hash function name that are valid for verify.
    const KECCAK256: u8 = 0;
    const SHA256: u8 = 1;

    /// Error if the signature is invalid.
    const ErrorInvalidSignature: u64 = 0;

    /// Error if the public key is invalid.
    const ErrorInvalidPubKey: u64 = 1;

    /// built-in functions
    public fun scheme_length(): u64 {
        VALID_SCHNORR_TO_SCHEME_NOSTR_LENGTH
    }

    public fun public_key_length(): u64 {
        VALID_SCHNORR_PUBKEY_LENGTH
    }

    public fun signature_length(): u64 {
        VALID_SCHNORR_SIG_LENGTH
    }

    public fun sha256(): u8 {
        SHA256
    }

    public fun get_public_key_from_authenticator_payload(authenticator_payload: &vector<u8>): vector<u8> {
        let public_key = vector::empty<u8>();
        let i = scheme_length() + signature_length();
        let public_key_position = scheme_length() + signature_length() + public_key_length();
        while (i < public_key_position) {
            let value = vector::borrow(authenticator_payload, i);
            vector::push_back(&mut public_key, *value);
            i = i + 1;
        };
        public_key
    }

    public fun get_signature_from_authenticator_payload(authenticator_payload: &vector<u8>): vector<u8> {
        let sign = vector::empty<u8>();
        let i = scheme_length();
        let signature_position = signature_length() + 1;
        while (i < signature_position) {
            let value = vector::borrow(authenticator_payload, i);
            vector::push_back(&mut sign, *value);
            i = i + 1;
        };
        sign
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
    #[expected_failure(abort_code = ErrorInvalidSignature)]
    public fun test_schnorr_invalid_sig() {
      let msg = x"00010203";
      let pk = x"3e99a541db69bd32040dfe5037fbf5210dafa8151a71e21c5204b05d95ce0a62";
      let invalid_sig = x"";

      verify(&invalid_sig, &pk, &msg, SHA256);
    }

    #[test]
    public fun test_schnorr_valid_sig() {
      let msg = x"00010203";
      let pk = x"3e99a541db69bd32040dfe5037fbf5210dafa8151a71e21c5204b05d95ce0a62";
      let valid_sig = x"bf4395f2f5a75dcfc82f7f4dd9ff032c450b5caed39bdd7b09df4cfa1b15ecd0c9f1d124916903b5291623bd06f2bc005ad8e92c74ec6d962f2d41f3ea2600e7";

      let result = verify(&valid_sig, &pk, &msg, SHA256);
      assert!(result, 0)
    }

    #[test]
    #[expected_failure(abort_code = ErrorInvalidPubKey)]
    public fun test_schnorr_invalid_pubkey() {
        let msg = x"00010203";
        let invalid_pk = x"";
        let sig = x"bf4395f2f5a75dcfc82f7f4dd9ff032c450b5caed39bdd7b09df4cfa1b15ecd0c9f1d124916903b5291623bd06f2bc005ad8e92c74ec6d962f2d41f3ea2600e7";

        verify(&sig, &invalid_pk, &msg, SHA256);
    }

    #[test]
    public fun test_schnorr_valid_pubkey() {
        let msg = x"00010203";
        let valid_pk = x"3e99a541db69bd32040dfe5037fbf5210dafa8151a71e21c5204b05d95ce0a62";
        let sig = x"bf4395f2f5a75dcfc82f7f4dd9ff032c450b5caed39bdd7b09df4cfa1b15ecd0c9f1d124916903b5291623bd06f2bc005ad8e92c74ec6d962f2d41f3ea2600e7";

        let result = verify(&sig, &valid_pk, &msg, SHA256);
        assert!(result, 0)
    }
}