module rooch_framework::ed25519 {
    use std::vector;

    /// constant codes
    const SCHEME_ED25519: u64 = 0;
    const V_ED25519_SCHEME_LENGTH: u64 = 1;
    const V_ED25519_PUBKEY_LENGTH: u64 = 32;
    const V_ED25519_SIG_LENGTH: u64 = 64;

    /// built-in functions
    public fun scheme(): u64 {
        SCHEME_ED25519
    }

    public fun scheme_length(): u64 {
        V_ED25519_SCHEME_LENGTH
    }

    public fun public_key_length(): u64 {
        V_ED25519_PUBKEY_LENGTH
    }

    public fun signature_length(): u64 {
        V_ED25519_SIG_LENGTH
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

    /// @param signature: 32-byte signature that is a point on the Ed25519 elliptic curve.
    /// @param public_key: 32-byte signature that is a point on the Ed25519 elliptic curve.
    /// @param msg: The message that we test the signature against.
    ///
    /// If the signature is a valid Ed25519 signature of the message and public key, return true.
    /// Otherwise, return false.
    public native fun verify(signature: &vector<u8>, public_key: &vector<u8>, msg: &vector<u8>): bool;

    #[test]
    fun test_ed25519_valid_sig() {
        // Test generated from https://github.com/MystenLabs/fastcrypto/blob/874bb52ccadf9800b3bc21e640449705d7ff9ab0/fastcrypto/src/tests/ed25519_tests.rs
        let msg = x"315f5bdb76d078c43b8ac0064e4a0164612b1fce77c869345bfc94c75894edd3";
        let pk = x"cc62332e34bb2d5cd69f60efbb2a36cb916c7eb458301ea36636c4dbb012bd88";
        let sig = x"cce72947906dbae4c166fc01fd096432784032be43db540909bc901dbc057992b4d655ca4f4355cf0868e1266baacf6919902969f063e74162f8f04bc4056105";

        let verify = verify(&sig, &pk, &msg);
        assert!(verify == true, 0);
    }

    #[test]
    fun test_ed25519_invalid_sig() {
        let msg = x"315f5bdb76d078c43b8ac0064e4a0164612b1fce77c869345bfc94c75894edd3";
        let pk = x"cc62332e34bb2d5cd69f60efbb2a36cb916c7eb458301ea36636c4dbb012bd88";
        let invalid_sig = x"cde72947906dbae4c166fc01fd096432784032be43db540909bc901dbc057992b4d655ca4f4355cf0868e1266baacf6919902969f063e74162f8f04bc4056105";

        let verify = verify(&invalid_sig, &pk, &msg);
        assert!(verify == false, 0);

        let pk = x"cc62332e34bb2d5cd69f60efbb2a36cb916c7eb458301ea36636c4dbb012bd88";
        let sig = x"cce72947906dbae4c166fc01fd096432784032be43db540909bc901dbc057992b4d655ca4f4355cf0868e1266baacf6919902969f063e74162f8f04bc4056105";
        let other_msg = x"415f5bdb76d078c43b8ac0064e4a0164612b1fce77c869345bfc94c75894edd3";

        let verify = verify(&sig, &pk, &other_msg);
        assert!(verify == false, 0);
    }

    #[test]
    fun test_ed25519_invalid_pubkey() {
        let msg = x"315f5bdb76d078c43b8ac0064e4a0164612b1fce77c869345bfc94c75894edd3";
        let pk = x"";
        let sig = x"cce72947906dbae4c166fc01fd096432784032be43db540909bc901dbc057992b4d655ca4f4355cf0868e1266baacf6919902969f063e74162f8f04bc4056105";

        let verify = verify(&sig, &pk, &msg);
        assert!(verify == false, 0)
    }
}