module rooch_framework::schnorr {
    use std::vector;

    /// constant codes
    const V_SCHNORR_TO_NOSTR_SCHEME_LENGTH: u64 = 1;
    const V_SCHNORR_PUBKEY_LENGTH: u64 = 32;
    const V_SCHNORR_SIG_LENGTH: u64 = 64;

    /// Hash function name that are valid for verify.
    const KECCAK256: u8 = 0;
    const SHA256: u8 = 1;

    /// Error if the signature is invalid.
    const EInvalidSignature: u64 = 0;

    /// Error if the public key is invalid.
    const EInvalidPubKey: u64 = 1;

    /// built-in functions
    public fun scheme_length(): u64 {
        V_SCHNORR_TO_NOSTR_SCHEME_LENGTH
    }

    public fun public_key_length(): u64 {
        V_SCHNORR_PUBKEY_LENGTH
    }

    public fun signature_length(): u64 {
        V_SCHNORR_SIG_LENGTH
    }

    public fun keccak256(): u8 {
        KECCAK256
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
    public fun test_schnorr_invalid_sig() {
      let msg = x"00010203";
      let pk = x"3e99a541db69bd32040dfe5037fbf5210dafa8151a71e21c5204b05d95ce0a62";
      let invalid_sig = x"235c98d74dd7926570a757550d282b0a4bde6c53772c62348a04085201811f7f99240b073efa9822b224ee906b8d816977106a72ca01ed6835fd04c9b7112400";

      let verify = verify(&invalid_sig, &pk, &msg, SHA256);
      assert!(!verify, EInvalidSignature);

      let pk = x"3e99a541db69bd32040dfe5037fbf5210dafa8151a71e21c5204b05d95ce0a62";
      let sig = x"bf4395f2f5a75dcfc82f7f4dd9ff032c450b5caed39bdd7b09df4cfa1b15ecd0c9f1d124916903b5291623bd06f2bc005ad8e92c74ec6d962f2d41f3ea2600e8";
      let other_msg = x"00010203";

      let verify = verify(&sig, &pk, &other_msg, KECCAK256);
      assert!(!verify, EInvalidSignature);
    }

    #[test]
    public fun test_schnorr_valid_sig() {
      let msg = x"00010203";
      let pk = x"3e99a541db69bd32040dfe5037fbf5210dafa8151a71e21c5204b05d95ce0a62";
      let sig = x"bf4395f2f5a75dcfc82f7f4dd9ff032c450b5caed39bdd7b09df4cfa1b15ecd0c9f1d124916903b5291623bd06f2bc005ad8e92c74ec6d962f2d41f3ea2600e7";

      let verify = verify(&sig, &pk, &msg, SHA256);
      assert!(verify, EInvalidSignature);
    }

    #[test]
    public fun test_schnorr_invalid_pubkey() {
        let msg = x"00010203";
        let pk = x"3e99a541db69bd32040dfe5037fbf5210dafa8151a71e21c5204b05d95ce0a63";
        let sig = x"bf4395f2f5a75dcfc82f7f4dd9ff032c450b5caed39bdd7b09df4cfa1b15ecd0c9f1d124916903b5291623bd06f2bc005ad8e92c74ec6d962f2d41f3ea2600e7";

        let verify = verify(&sig, &pk, &msg, SHA256);
        assert!(!verify, EInvalidPubKey)
    }

    #[test]
    public fun test_schnorr_valid_pubkey() {
        let msg = x"00010203";
        let pk = x"3e99a541db69bd32040dfe5037fbf5210dafa8151a71e21c5204b05d95ce0a62";
        let sig = x"1312289adb61ab33b1132f5ecd1e4d5f791f4618f5e17de2284b286c534a4fb8a7ee8a141f9a98eab92488796007e53cb71a3a3a4d738cf2a818acb48178153a";

        let verify = verify(&sig, &pk, &msg, KECCAK256);
        assert!(verify, EInvalidPubKey)
    }
}