// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

module rooch_framework::rs256 {
    use std::vector;

    /// Minimum modulus (n) length (bits) for RSASSA-PKCS1-V1_5 with SHA-256 (RS256)
    const RSASSA_PKCS1_V1_5_MINIMUM_MODULUS_LENGTH: u64 = 2048;
    /// Minimum exponent (e) length (bytes) for RSASSA-PKCS1-V1_5 with SHA-256 (RS256)
    const RSASSA_PKCS1_V1_5_MINIMUM_EXPONENT_LENGTH: u64 = 1;
    /// Maximum exponent (e) length (bytes) for RSASSA-PKCS1-V1_5 with SHA-256 (RS256)
    const RSASSA_PKCS1_V1_5_MAXIMUM_EXPONENT_LENGTH: u64 = 512;
    /// Message length for the Sha2-256 hash function
    const SHA256_MESSAGE_LENGTH: u64 = 32;

    // Error codes
    const ErrorInvalidSignature: u64 = 1;
    const ErrorInvalidPubKey: u64 = 2;
    const ErrorInvalidHashType: u64 = 3;
    const ErrorInvalidMessageLength: u64 = 4;

    // Hash type
    const SHA256: u8 = 0;

    // functions to the constants
    public fun sha256(): u8 {
        SHA256
    }

    /// Verifies a RSA signature from public modulus (n) and public exponent (e) over RSASSA-PKCS1-V1_5 with SHA-256 (RS256).
    /// The message will be the original message with hashing in-function.
    public fun verify(
        signature: &vector<u8>,
        n: &vector<u8>,
        e: &vector<u8>,
        msg: &vector<u8>
    ): bool {
        // check conditions for verify function
        check_conditions_verify(signature, n, e);
        // call native_verify
        native_verify(signature, n, e, msg)
    }

    native fun native_verify(
        signature: &vector<u8>,
        n: &vector<u8>,
        e: &vector<u8>,
        msg: &vector<u8>,
    ): bool;

    /// Verifies a RSA signature from public modulus (n) and public exponent (e) over RSASSA-PKCS1-V1_5 with SHA-256 (RS256).
    /// The message will be the hashed using SHA256 before the verification.
    public fun verify_prehash(
        signature: &vector<u8>,
        n: &vector<u8>,
        e: &vector<u8>,
        msg: &vector<u8>,
        hash_type: u8
    ): bool {
        // check conditions for verify prehash function
        check_conditions_verify_prehash(signature, n, e, msg, hash_type);
        // call native_verify
        native_verify_prehash(signature, n, e, msg, hash_type)
    }

    native fun native_verify_prehash(
        signature: &vector<u8>,
        n: &vector<u8>,
        e: &vector<u8>,
        msg: &vector<u8>,
        hash_type: u8
    ): bool;

    fun check_conditions_verify(signature: &vector<u8>, n: &vector<u8>, e: &vector<u8>) {
        // signature length is modulus (n) length (in bits) / 8
        assert!(vector::length(signature) == vector::length(n) * 8 / 8, ErrorInvalidSignature); // cast to bytes first
        // ensure modulus (n) length meet the minimum modulus length requirements
        assert!(vector::length(n) >= RSASSA_PKCS1_V1_5_MINIMUM_MODULUS_LENGTH / 8, ErrorInvalidPubKey); // cast to bytes first
        // ensure exponent (e) length meet the minimum exponent length requirements
        assert!(vector::length(e) >= RSASSA_PKCS1_V1_5_MINIMUM_EXPONENT_LENGTH, ErrorInvalidPubKey);
        // ensure exponent (e) length meet the maximum exponent length requirements
        assert!(vector::length(e) <= RSASSA_PKCS1_V1_5_MAXIMUM_EXPONENT_LENGTH, ErrorInvalidPubKey);
    }

    fun check_conditions_verify_prehash(signature: &vector<u8>, n: &vector<u8>, e: &vector<u8>, msg: &vector<u8>, hash_type: u8) {
        // include all verify conditions
        check_conditions_verify(signature, n, e);
        assert!(hash_type == SHA256, ErrorInvalidHashType);
        assert!(vector::length(msg) == SHA256_MESSAGE_LENGTH, ErrorInvalidMessageLength);
    }
}
