// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

/// Source from https://github.com/MystenLabs/sui/blob/da72e73a9f4a1b42df863b76af7745a4eeb1d412/crates/sui-framework/packages/sui-framework/sources/crypto/bls12381.move
module moveos_std::bls12381 {

    // Encode failed error
    const E_SIG_FAILED: u64 = 1;

    // Decode failed error
    const E_PUBKEY_FAILED: u64 = 2;

    /// @param signature: A 48-bytes signature that is a point on the G1 subgroup.
    /// @param public_key: A 96-bytes public key that is a point on the G2 subgroup.
    /// @param msg: The message that we test the signature against.
    ///
    /// If the signature is a valid signature of the message and public key according to
    /// BLS_SIG_BLS12381G1_XMD:SHA-256_SSWU_RO_NUL_, return true. Otherwise, return false.
    public native fun bls12381_min_sig_verify(signature: &vector<u8>, public_key: &vector<u8>, msg: &vector<u8>): bool;

    /// @param signature: A 96-bytes signature that is a point on the G2 subgroup.
    /// @param public_key: A 48-bytes public key that is a point on the G1 subgroup.
    /// @param msg: The message that we test the signature against.
    ///
    /// If the signature is a valid signature of the message and public key according to
    /// BLS_SIG_BLS12381G2_XMD:SHA-256_SSWU_RO_NUL_, return true. Otherwise, return false.
    public native fun bls12381_min_pk_verify(signature: &vector<u8>, public_key: &vector<u8>, msg: &vector<u8>): bool;

    #[test]
    fun test_bls12381_min_sig_verify() {
        let msg = b"hello, narwhal";
        let public_key = x"8c66dc2c1ea9e53f0985c17b4e7af19912b6d3c40e0c5920a5a12509b4eb3619f5e07ec56ea77f0b30629ba1cc72d75b139460782a5f0e2f89fb4c42b4b8a5fae3d260102220e63d0754e7e1846deefd3988eade4ed37f1385437d19de1a1618";
        let signature = x"89dff2dc1e9428b9437d50b37f8160eca790110ea2a79b6c88a43a16953466f8e391ff65842b067a1c9441c7c2cebce0";

        let is_verified = bls12381_min_sig_verify(&signature, &public_key, &msg);

        assert!(is_verified == true, 3);
    }

    #[test]
    fun test_verification_failed_bls12381_min_sig_verify() {
        let msg = b"hello, rooch";
        let public_key = x"8c66dc2c1ea9e53f0985c17b4e7af19912b6d3c40e0c5920a5a12509b4eb3619f5e07ec56ea77f0b30629ba1cc72d75b139460782a5f0e2f89fb4c42b4b8a5fae3d260102220e63d0754e7e1846deefd3988eade4ed37f1385437d19de1a1618";
        let signature = x"89dff2dc1e9428b9437d50b37f8160eca790110ea2a79b6c88a43a16953466f8e391ff65842b067a1c9441c7c2cebce0";
        
        let is_verified = bls12381_min_sig_verify(&signature, &public_key, &msg);

        assert!(is_verified == false, 4);
    }

    #[test]
    #[expected_failure(abort_code = E_PUBKEY_FAILED, location = Self)]
    fun test_pk_failed_bls12381_min_sig_verify() {
        let msg = b"hello, narwhal";
        let public_key = x"8c66dc2c1ea9e53f0985c17b4e7af19912b6d3c40e0c5920a5a12509b4eb3619f5e07ec56ea77f0b30629ba1cc72d75b139460782a5f0e2f89fb4c42b4b8a5fae3d260102220e63d0754e7e1846deefd3988eade4ed37f1385437d19de1a";
        let signature = x"89dff2dc1e9428b9437d50b37f8160eca790110ea2a79b6c88a43a16953466f8e391ff65842b067a1c9441c7c2cebce0";

        bls12381_min_sig_verify(&signature, &public_key, &msg);
    }

    #[test]
    #[expected_failure(abort_code = E_SIG_FAILED, location = Self)]
    fun test_sig_failed_bls12381_min_sig_verify() {
        let msg = b"hello, narwhal";
        let public_key = x"8c66dc2c1ea9e53f0985c17b4e7af19912b6d3c40e0c5920a5a12509b4eb3619f5e07ec56ea77f0b30629ba1cc72d75b139460782a5f0e2f89fb4c42b4b8a5fae3d260102220e63d0754e7e1846deefd3988eade4ed37f1385437d19de1a1618";
        let signature = x"89dff2dc1e9428b9437d50b37f8160eca790110ea2a79b6c88a43a16953466f8e391ff65842b067a1c9441c7c2ce";

        bls12381_min_sig_verify(&signature, &public_key, &msg);
    }

    #[test]
    fun test_bls12381_min_pk_verify() {
        let msg = b"hello, narwhal";
        let public_key = x"b157f238403a5b980546fd19ca48f79a2613e3e3a91d14ee69908b8816e4c53665370b2fbd0db62cc4aa0e8caeedc9b5";
        let signature = x"8dec0b9a1a629cc96c57144ee8e7dd5c93acb465286f1214df3b8482c3f16e10db4277ead785f5d5bc77b4e51affd2580dead4d0d21cf20fc5e2b4bec2586c2bd6c73fee76c11f214871f77dada4c578034c3b978f1cccb82bdd78fe5ee67de1";

        let is_verified = bls12381_min_pk_verify(&signature, &public_key, &msg);

        assert!(is_verified == true, 5);
    }

    #[test]
    fun test_verification_failed_bls12381_min_pk_verify() {
        let msg = b"hello, rooch";
        let public_key = x"b157f238403a5b980546fd19ca48f79a2613e3e3a91d14ee69908b8816e4c53665370b2fbd0db62cc4aa0e8caeedc9b5";
        let signature = x"8dec0b9a1a629cc96c57144ee8e7dd5c93acb465286f1214df3b8482c3f16e10db4277ead785f5d5bc77b4e51affd2580dead4d0d21cf20fc5e2b4bec2586c2bd6c73fee76c11f214871f77dada4c578034c3b978f1cccb82bdd78fe5ee67de1";

        let is_verified = bls12381_min_pk_verify(&signature, &public_key, &msg);

        assert!(is_verified == false, 6);
    }

    #[test]
    #[expected_failure(abort_code = E_PUBKEY_FAILED, location = Self)]
    fun test_pk_failed_bls12381_min_pk_verify() {
        let msg = b"hello, narwhal";
        let public_key = x"b157f238403a5b980546fd19ca48f79a2613e3e3a91d14ee69908b8816e4c53665370b2fbd0db62cc4aa0e8caeed";
        let signature = x"8dec0b9a1a629cc96c57144ee8e7dd5c93acb465286f1214df3b8482c3f16e10db4277ead785f5d5bc77b4e51affd2580dead4d0d21cf20fc5e2b4bec2586c2bd6c73fee76c11f214871f77dada4c578034c3b978f1cccb82bdd78fe5ee67de1";

        bls12381_min_pk_verify(&signature, &public_key, &msg);
    }

    #[test]
    #[expected_failure(abort_code = E_SIG_FAILED, location = Self)]
    fun test_sig_failed_bls12381_min_pk_verify() {
        let msg = b"hello, narwhal";
        let public_key = x"b157f238403a5b980546fd19ca48f79a2613e3e3a91d14ee69908b8816e4c53665370b2fbd0db62cc4aa0e8caeedc9b5";
        let signature = x"8dec0b9a1a629cc96c57144ee8e7dd5c93acb465286f1214df3b8482c3f16e10db4277ead785f5d5bc77b4e51affd2580dead4d0d21cf20fc5e2b4bec2586c2bd6c73fee76c11f214871f77dada4c578034c3b978f1cccb82bdd78fe5ee6";

        bls12381_min_pk_verify(&signature, &public_key, &msg);
    }
}