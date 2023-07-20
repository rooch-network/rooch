module rooch_framework::schnorr {
    /// @param signature: A 65-bytes signature that is signed using Schnorr over Secpk256k1 key pairs.
    /// @param msg: The message that the signature is signed against.
    /// @param hash: The hash function used to hash the message when signing.
    ///
    /// If the signature is valid to the pubkey and hashed message, return true. Else false.
    public native fun verify(
        signature: &vector<u8>,
        msg: &vector<u8>,
        hash: u8
    ): bool;
   
    // TODO add tests following ecdsa_k1

    #[test]
    public fun test_schnorr_valid_sig() {
    }

    #[test]
    public fun test_schnorr_invalid_sig() {
    }

    #[test]
    public fun test_schnorr_valid_message() {
    }

    #[test]
    public fun test_schnorr_invalid_message() {
    }
}