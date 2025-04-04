// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

module rooch_framework::schnorr {
    /// constant codes
    const SCHNORR_VERIFYING_KEY_LENGTH: u64 = 32;
    const SCHNORR_SIGNATURE_LENGTH: u64 = 64;

    /// Error if the signature is invalid.
    const ErrorInvalidSignature: u64 = 1;

    /// Error if the verifying key is invalid.
    const ErrorInvalidVerifyingKey: u64 = 2;

    /// built-in functions
    public fun verifying_key_length(): u64 {
        SCHNORR_VERIFYING_KEY_LENGTH
    }

    public fun signature_length(): u64 {
        SCHNORR_SIGNATURE_LENGTH
    }

    /// @param signature: A 64-bytes signature that is signed using schnorr over secpk256k1 key pairs.
    /// @param verifying_key: A 32-bytes verifying key that is used to verify messages.
    /// @param msg: The message that the signature is signed against.
    ///
    /// If the signature and message are valid to the verifying key, return true. Else false.
    native public fun verify(
        signature: &vector<u8>,
        verifying_key: &vector<u8>,
        msg: &vector<u8>,
    ): bool;

    #[test]
    #[expected_failure(location=Self, abort_code = ErrorInvalidSignature)]
    public fun test_schnorr_invalid_signature() {
      let msg = x"00010203";
      let vk = x"3e99a541db69bd32040dfe5037fbf5210dafa8151a71e21c5204b05d95ce0a62";
      let invalid_sig = x"";

      verify(&invalid_sig, &vk, &msg);
    }

    #[test]
    public fun test_schnorr_valid_signature() {
      let msg = x"00010203";
      let vk = x"3e99a541db69bd32040dfe5037fbf5210dafa8151a71e21c5204b05d95ce0a62";
      let valid_sig = x"bf4395f2f5a75dcfc82f7f4dd9ff032c450b5caed39bdd7b09df4cfa1b15ecd0c9f1d124916903b5291623bd06f2bc005ad8e92c74ec6d962f2d41f3ea2600e7";

      let result = verify(&valid_sig, &vk, &msg);
      assert!(result, 0)
    }

    #[test]
    #[expected_failure(location=Self, abort_code = ErrorInvalidVerifyingKey)]
    public fun test_schnorr_invalid_verifying_key() {
        let msg = x"00010203";
        let invalid_vk = x"";
        let sig = x"bf4395f2f5a75dcfc82f7f4dd9ff032c450b5caed39bdd7b09df4cfa1b15ecd0c9f1d124916903b5291623bd06f2bc005ad8e92c74ec6d962f2d41f3ea2600e7";

        verify(&sig, &invalid_vk, &msg);
    }

    #[test]
    public fun test_schnorr_valid_verifying_key() {
        let msg = x"00010203";
        let valid_vk = x"3e99a541db69bd32040dfe5037fbf5210dafa8151a71e21c5204b05d95ce0a62";
        let sig = x"bf4395f2f5a75dcfc82f7f4dd9ff032c450b5caed39bdd7b09df4cfa1b15ecd0c9f1d124916903b5291623bd06f2bc005ad8e92c74ec6d962f2d41f3ea2600e7";

        let result = verify(&sig, &valid_vk, &msg);
        assert!(result, 0)
    }
}
