// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

// Evm Precompiled Contracts https://www.evm.codes/precompiled?fork=cancun
module moveos_std::evm {
    use moveos_std::hash;

    #[test_only]
    use std::vector;
    
    const ErrorEcRecoverFailed: u64 = 1;
    const ErrorModexpFailed: u64 = 5;
    const ErrorEcAddFailed: u64 = 6;
    const ErrorEcMulFailed: u64 = 7;
    const ErrorEcPairingFailed: u64 = 8;
    const ErrorBlake2fFailed: u64 = 9;
    const ErrorPointEvaluationFailed: u64 = 10;
    const ErrorInvalidInputSize: u64 = 11;

    /// @param hash: Keccack-256 hash of the transaction.
    /// @param v: Recovery identifier, expected to be either 27 or 28.
    /// @param r: x-value, expected to be in the range ]0; secp256k1n[.
    /// @param s: Expected to be in the range ]0; secp256k1n[.
    ///
    /// @return public_address: The recovered 20-byte address right aligned to 32 bytes.
    /// 
    /// Elliptic curve digital signature algorithm (ECDSA) public key recovery function.
    public native fun ec_recover(hash: vector<u8>, v: vector<u8>, r: vector<u8>, s: vector<u8>): vector<u8>;

    /// @param data: Data to hash with SHA2-256.
    /// 
    /// @return hash: The result hash.
    /// 
    /// Hash function.
    public fun sha2_256(data: vector<u8>): vector<u8> {
        std::hash::sha2_256(data)
    }

    /// @param data: Data to hash with RIPEMD-160.
    /// 
    /// @return hash: The result 20-byte hash right aligned to 32 bytes.
    /// 
    /// Hash function.
    public fun ripemd_160(data: vector<u8>): vector<u8> {
        hash::ripemd160(&data)
    }

    /// @param data: Data to return.
    /// 
    /// @return data: Data from input.
    /// 
    /// Returns the input.
    public fun identity(data: vector<u8>): vector<u8> {
       let data_clone = data; 
       data_clone
    }

    /// @param b_size: Byte size of B.
    /// @param e_size: Byte size of E.
    /// @param m_size: Byte size of M.
    /// @param b: Base as unsigned integer.
    /// @param e: Exponent as unsigned integer, if zero, then B ** E will be one.
    /// @param m: Modulo as unsigned integer, if zero, then returns zero.
    /// 
    /// @return value: Result of the computation, with the same number of bytes as M.
    ///
    /// Arbitrary-precision exponentiation under modulo.
    public native fun modexp(b_size: vector<u8>, e_size: vector<u8>, m_size: vector<u8>, b: vector<u8>, e: vector<u8>, m: vector<u8>): vector<u8>;

    /// @param x1: X coordinate of the first point on the elliptic curve 'alt_bn128'.
    /// @param y1: Y coordinate of the first point on the elliptic curve 'alt_bn128'.
    /// @param x2: X coordinate of the second point on the elliptic curve 'alt_bn128'.
    /// @param y2: Y coordinate of the second point on the elliptic curve 'alt_bn128'.
    /// 
    /// @return x: X coordinate of the result point on the elliptic curve 'alt_bn128'.
    /// @return y: Y coordinate of the result point on the elliptic curve 'alt_bn128'.
    ///
    /// Notes: The point at infinity is encoded with both field x and y at 0.
    /// 
    /// Point addition (ADD) on the elliptic curve 'alt_bn128'.
    public native fun ec_add(x1: vector<u8>, y1: vector<u8>, x2: vector<u8>, y2: vector<u8>): (vector<u8>, vector<u8>);

    /// @param x1: X coordinate of the first point on the elliptic curve 'alt_bn128'.
    /// @param y1: Y coordinate of the first point on the elliptic curve 'alt_bn128'.
    /// @param s: Scalar to use for the multiplication.
    /// 
    /// @return x: X coordinate of the result point on the elliptic curve 'alt_bn128'.
    /// @return y: Y coordinate of the result point on the elliptic curve 'alt_bn128'.
    ///
    /// Notes: The point at infinity is encoded with both field x and y at 0.
    /// 
    /// Scalar multiplication (MUL) on the elliptic curve 'alt_bn128'.
    public native fun ec_mul(x1: vector<u8>, y1: vector<u8>, s: vector<u8>): (vector<u8>, vector<u8>);

    /// @param data: Coordinates of the points. 
    /// The input must always be a multiple of 6 32-byte values. 0 inputs is valid and returns 1.
    /// 
    /// @return success: 1 if the pairing was a success, 0 otherwise.
    ///
    /// Notes: The point at infinity is encoded with both field x and y at 0.
    /// 
    /// Bilinear function on groups on the elliptic curve 'alt_bn128'.
    public native fun ec_pairing(data: vector<u8>): vector<u8>;

    /// @param rounds: Number of rounds (big-endian unsigned integer).
    /// @param h: State vector (8 8-byte little-endian unsigned integer).
    /// @param m: Message block vector (16 8-byte little-endian unsigned integer).
    /// @param t: Offset counters (2 8-byte little-endian integer).
    /// @param f: Final block indicator flag (0 or 1).
    /// 
    /// @return h: State vector (8 8-byte little-endian unsigned integer).
    ///
    /// Compression function F used in the BLAKE2 cryptographic hashing algorithm.
    public native fun blake2f(rounds: vector<u8>, h: vector<u8>, m: vector<u8>, t: vector<u8>, f: vector<u8>): vector<u8>;

    /// @param versioned_hash: Reference to a blob in the execution layer.
    /// @param x: x-coordinate at which the blob is being evaluated.
    /// @param y: y-coordinate at which the blob is being evaluated.
    /// @param commitment: Commitment to the blob being evaluated.
    /// @param proof: Proof associated with the commitment.
    /// 
    /// @return FIELD_ELEMENTS_PER_BLOB: The number of field elements in the blob.
    /// @return : BLS_MODULUS: The modulus used in the BLS signature scheme.
    ///
    /// Verify p(z) = y given commitment that corresponds to the polynomial p(x) and a KZG proof. Also verify that the provided commitment matches the provided versioned_hash.
    public native fun point_evaluation(versioned_hash: vector<u8>, x: vector<u8>, y: vector<u8>, commitment: vector<u8>, proof: vector<u8>): (vector<u8>, vector<u8>);

    #[test]
    fun test_ec_recover() {
        let hash = x"456e9aea5e197a1f1af7a3e85a3212fa4049a3ba34c2289b4c860fc0b0c64ef3";
        let v = x"000000000000000000000000000000000000000000000000000000000000001c";
        let r = x"9242685bf161793cc25603c231bc2f568eb630ea16aa137d2664ac8038825608";
        let s = x"4f8ae3bd7535248d0bd448298cc2e2071e56992d0774dc340c368ae950852ada";

        let public_address = ec_recover(hash, v, r, s);
        assert!(public_address == x"0000000000000000000000007156526fbd7a3c72969b54f64e42c10fbb768c8a", ErrorEcRecoverFailed);
    }

    #[test]
    fun test_modexp() {
        let b_size = x"0000000000000000000000000000000000000000000000000000000000000001";
        let e_size = x"0000000000000000000000000000000000000000000000000000000000000001";
        let m_size = x"0000000000000000000000000000000000000000000000000000000000000001";
        let b = x"08";
        let e = x"09";
        let m = x"0a";

        let value = modexp(b_size, e_size, m_size, b, e, m);
        assert!(value == x"08", ErrorModexpFailed);
    }

    #[test]
    fun test_ec_add() {
        let x1 = x"0000000000000000000000000000000000000000000000000000000000000001";
        let y1 = x"0000000000000000000000000000000000000000000000000000000000000002";
        let x2 = x"0000000000000000000000000000000000000000000000000000000000000001";
        let y2 = x"0000000000000000000000000000000000000000000000000000000000000002";

        let (x3, y3) = ec_add(x1, y1, x2, y2);
        assert!(x3 == x"030644e72e131a029b85045b68181585d97816a916871ca8d3c208c16d87cfd3", ErrorEcAddFailed);
        assert!(y3 == x"15ed738c0e0a7c92e7845f96b2ae9c0a68a6a449e3538fc7ff3ebf7a5a18a2c4", ErrorEcAddFailed);
    }

    #[test]
    fun test_ec_mul() {
        let x1 = x"0000000000000000000000000000000000000000000000000000000000000001";
        let y1 = x"0000000000000000000000000000000000000000000000000000000000000002";
        let s = x"0000000000000000000000000000000000000000000000000000000000000002";

        let (x, y) = ec_mul(x1, y1, s);
        assert!(x == x"030644e72e131a029b85045b68181585d97816a916871ca8d3c208c16d87cfd3", ErrorEcMulFailed);
        assert!(y == x"15ed738c0e0a7c92e7845f96b2ae9c0a68a6a449e3538fc7ff3ebf7a5a18a2c4", ErrorEcMulFailed);
    }

    #[test]
    fun test_ec_pairing() {
        let x1 = x"2cf44499d5d27bb186308b7af7af02ac5bc9eeb6a3d147c186b21fb1b76e18da";
        let y1 = x"2c0f001f52110ccfe69108924926e45f0b0c868df0e7bde1fe16d3242dc715f6";
        let x2 = x"1fb19bb476f6b9e44e2a32234da8212f61cd63919354bc06aef31e3cfaff3ebc";
        let y2 = x"22606845ff186793914e03e21df544c34ffe2f2f3504de8a79d9159eca2d98d9";
        let x3 = x"2bd368e28381e8eccb5fa81fc26cf3f048eea9abfdd85d7ed3ab3698d63e4f90";
        let y3 = x"2fe02e47887507adf0ff1743cbac6ba291e66f59be6bd763950bb16041a0a85e";
        let x4 = x"0000000000000000000000000000000000000000000000000000000000000001";
        let y4 = x"30644e72e131a029b85045b68181585d97816a916871ca8d3c208c16d87cfd45";
        let x5 = x"1971ff0471b09fa93caaf13cbf443c1aede09cc4328f5a62aad45f40ec133eb4";
        let y5 = x"091058a3141822985733cbdddfed0fd8d6c104e9e9eff40bf5abfef9ab163bc7";
        let x6 = x"2a23af9a5ce2ba2796c1f4e453a370eb0af8c212d9dc9acd8fc02c2e907baea2";
        let y6 = x"23a8eb0b0996252cb548a4487da97b02422ebc0e834613f954de6c7e0afdc1fc";

        vector::append<u8>(&mut x1, y1);
        vector::append<u8>(&mut x1, x2);
        vector::append<u8>(&mut x1, y2);
        vector::append<u8>(&mut x1, x3);
        vector::append<u8>(&mut x1, y3);
        vector::append<u8>(&mut x1, x4);
        vector::append<u8>(&mut x1, y4);
        vector::append<u8>(&mut x1, x5);
        vector::append<u8>(&mut x1, y5);
        vector::append<u8>(&mut x1, x6);
        vector::append<u8>(&mut x1, y6);

        let success = ec_pairing(x1);
        assert!(success == x"0000000000000000000000000000000000000000000000000000000000000001", ErrorEcPairingFailed);
    }

    #[test]
    #[expected_failure(abort_code = ErrorInvalidInputSize, location = Self)]
    fun test_ec_add_input_not_match() {
        let x1 = x"01";
        let y1 = x"02";
        let x2 = x"01";
        let y2 = x"02";

        ec_add(x1, y1, x2, y2);
    }

    #[test]
    #[expected_failure(abort_code = ErrorEcPairingFailed, location = Self)]
    fun test_ec_pairing_input_not_match() {
        let x1 = x"2cf44499d5d27bb186308b7af7af02ac5bc9eeb6a3d147c186b21fb1b76e18da";
        let y1 = x"2c0f001f52110ccfe69108924926e45f0b0c868df0e7bde1fe16d3242dc715f6";
        vector::append<u8>(&mut x1, y1);

        ec_pairing(x1);
    }

    #[test]
    fun test_blake2f() {
        let rounds = x"0000000c";
        let h = x"48c9bdf267e6096a3ba7ca8485ae67bb2bf894fe72f36e3cf1361d5f3af54fa5d182e6ad7f520e511f6c3e2b8c68059b6bbd41fbabd9831f79217e1319cde05b";
        let m = x"6162630000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000";
        let t = x"03000000000000000000000000000000";
        let f = x"01";

        let h = blake2f(rounds, h, m, t, f);
        assert!(h == x"ba80a53f981c4d0d6a2797b69f12f6e94c212f14685ac4b74b12bb6fdbffa2d17d87c5392aab792dc252d5de4533cc9518d38aa8dbf1925ab92386edd4009923", ErrorBlake2fFailed);
    }

    #[test]
    fun test_point_evaluation() {
        let versioned_hash = x"01e798154708fe7789429634053cbf9f99b619f9f084048927333fce637f549b";
        let x = x"73eda753299d7d483339d80809a1d80553bda402fffe5bfeffffffff00000000";
        let y = x"1522a4a7f34e1ea350ae07c29c96c7e79655aa926122e95fe69fcbd932ca49e9";
        let commitment = x"8f59a8d2a1a625a17f3fea0fe5eb8c896db3764f3185481bc22f91b4aaffcca25f26936857bc3a7c2539ea8ec3a952b7";
        let proof = x"a62ad71d14c5719385c0686f1871430475bf3a00f0aa3f7b8dd99a9abc2160744faf0070725e00b60ad9a026a15b1a8c";

        let (field_elements_per_blob, bls_modulus) = point_evaluation(versioned_hash, x, y, commitment, proof);
        assert!(field_elements_per_blob == x"0000000000000000000000000000000000000000000000000000000000001000", ErrorPointEvaluationFailed);
        assert!(bls_modulus == x"73eda753299d7d483339d80809a1d80553bda402fffe5bfeffffffff00000001", ErrorPointEvaluationFailed);
    }
}