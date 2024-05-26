// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

// Evm Precompiled Contracts https://www.evm.codes/precompiled?fork=cancun
module moveos_std::evm {

    const ErrorEcAddFailed: u64 = 6;
    const ErrorEcPairingFailed: u64 = 8;

    // The coordinate must be 32 bytes.
    const ErrorInvalidCoordinate: u64 = 11;

    /// @param data: Arbitrary binary data to hash
    /// 
    /// Hash function.
    public fun sha2_256(data: vector<u8>): vector<u8> {
        std::hash::sha2_256(data)
    }

    /// @param x1: X coordinate of the first point on the elliptic curve 'alt_bn128'.
    /// @param y1: Y coordinate of the first point on the elliptic curve 'alt_bn128'.
    /// @param x2: X coordinate of the second point on the elliptic curve 'alt_bn128'.
    /// @param y2: Y coordinate of the second point on the elliptic curve 'alt_bn128'.
    ///
    /// Notes: The point at infinity is encoded with both field x and y at 0.
    /// 
    /// Point addition (ADD) on the elliptic curve 'alt_bn128'
    public native fun ec_add(x1: vector<u8>, y1: vector<u8>, x2: vector<u8>, y2: vector<u8>): (vector<u8>, vector<u8>);

    /// @param data: Coordinates of the points. 
    /// The input must always be a multiple of 6 32-byte values. 0 inputs is valid and returns 1.
    ///
    /// Notes: The point at infinity is encoded with both field x and y at 0.
    /// 
    /// Bilinear function on groups on the elliptic curve 'alt_bn128'.
    public native fun ec_pairing(data: vector<u8>): vector<u8>;

    #[test_only]
    use std::vector;

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
    #[expected_failure(abort_code = ErrorInvalidCoordinate, location = Self)]
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
}