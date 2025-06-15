// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

/// Defines the `did_key` module, providing operations specific to did:key identifiers.
///
/// ## Overview
///
/// The did:key method is a simple and convenient way to express public keys as DIDs.
/// This module builds on top of `multibase_key` to provide specialized functions for
/// generating and parsing did:key identifiers.
///
/// ## Format
///
/// A did:key identifier has the format:
/// `did:key:<multibase-encoded-public-key-with-multicodec-prefix>`
///
/// For example:
/// - Ed25519: `did:key:z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK`
/// - Secp256k1: `did:key:zQ3shokFTS3brHcDQrn82RUDfCZESWL1ZdCEJwekUDPQiYBme`
///
/// The multibase-encoded part is created by:
/// 1. Adding the appropriate multicodec prefix to the raw public key bytes
/// 2. Encoding the result with base58btc
/// 3. Adding the 'z' prefix (for base58btc)

module moveos_std::did_key {
    use std::string::{Self, String};
    use std::option::{Self, Option};
    use moveos_std::multibase_key;

    /// Error when the did:key string format is invalid
    const ErrorInvalidDidKeyFormat: u64 = 1;
    // General test failure code for asserts
    const ETestAssertionFailed: u64 = 100;

    /// The did:key method string
    const DID_KEY_METHOD: vector<u8> = b"did:key:";

    /// Generate a did:key identifier from a public key and key type
    /// 
    /// @param pubkey - The raw public key bytes
    /// @param key_type - The key type (1=Ed25519, 2=Secp256k1, 3=Secp256r1)
    /// @return - A did:key identifier string
    public fun generate(pubkey: &vector<u8>, key_type: u8): String {
        // First encode the key with multicodec prefix and multibase encoding
        let encoded_key = multibase_key::encode_with_type(pubkey, key_type);
        
        // Create the did:key identifier by prepending "did:key:"
        let did_key = string::utf8(DID_KEY_METHOD);
        string::append(&mut did_key, encoded_key);
        
        did_key
    }

    /// Generate a did:key identifier from an Ed25519 public key
    /// 
    /// @param pubkey - The raw Ed25519 public key bytes (32 bytes)
    /// @return - A did:key identifier string
    public fun generate_ed25519(pubkey: &vector<u8>): String {
        generate(pubkey, multibase_key::key_type_ed25519())
    }

    /// Generate a did:key identifier from a Secp256k1 public key
    /// 
    /// @param pubkey - The raw Secp256k1 compressed public key bytes (33 bytes)
    /// @return - A did:key identifier string
    public fun generate_secp256k1(pubkey: &vector<u8>): String {
        generate(pubkey, multibase_key::key_type_secp256k1())
    }

    /// Generate a did:key identifier from a Secp256r1 public key
    /// 
    /// @param pubkey - The raw Secp256r1 compressed public key bytes (33 bytes)
    /// @return - A did:key identifier string
    public fun generate_secp256r1(pubkey: &vector<u8>): String {
        generate(pubkey, multibase_key::key_type_ecdsar1())
    }

    /// Parse a did:key identifier to extract the key type and raw public key bytes
    /// 
    /// @param did_key_str - The did:key identifier string
    /// @return - A tuple of (key_type, raw_key_bytes), or abort if invalid
    public fun parse(did_key_str: &String): (u8, vector<u8>) {
        // Verify the did:key prefix
        let did_key_bytes = string::bytes(did_key_str);
        let prefix_len = std::vector::length(&DID_KEY_METHOD);
        
        // Check if the string is long enough and starts with "did:key:"
        if (std::vector::length(did_key_bytes) <= prefix_len) {
            abort ErrorInvalidDidKeyFormat
        };
        
        let i = 0;
        while (i < prefix_len) {
            if (*std::vector::borrow(did_key_bytes, i) != *std::vector::borrow(&DID_KEY_METHOD, i)) {
                abort ErrorInvalidDidKeyFormat
            };
            i = i + 1;
        };
        
        // Extract the multibase-encoded part
        let multibase_part = string::sub_string(did_key_str, prefix_len, string::length(did_key_str));
        
        // Decode the multibase-encoded part to get key type and raw key bytes
        multibase_key::decode_with_type(&multibase_part)
    }

    /// Parse a did:key identifier and check if it's an Ed25519 key
    /// 
    /// @param did_key_str - The did:key identifier string
    /// @return - Option containing the raw Ed25519 public key bytes, or none if not an Ed25519 key or invalid
    public fun parse_ed25519(did_key_str: &String): Option<vector<u8>> {
        // Try to parse the did:key identifier
        let parse_result = parse_option(did_key_str);
        
        if (option::is_none(&parse_result)) {
            return option::none()
        };
        
        let key_info = option::extract(&mut parse_result);
        
        // Check if it's an Ed25519 key
        if (multibase_key::key_info_type(&key_info) == multibase_key::key_type_ed25519()) {
            option::some(multibase_key::key_info_bytes(&key_info))
        } else {
            option::none()
        }
    }

    /// Parse a did:key identifier and check if it's a Secp256k1 key
    /// 
    /// @param did_key_str - The did:key identifier string
    /// @return - Option containing the raw Secp256k1 public key bytes, or none if not a Secp256k1 key or invalid
    public fun parse_secp256k1(did_key_str: &String): Option<vector<u8>> {
        // Try to parse the did:key identifier
        let parse_result = parse_option(did_key_str);
        
        if (option::is_none(&parse_result)) {
            return option::none()
        };
        
        let key_info = option::extract(&mut parse_result);
        
        // Check if it's a Secp256k1 key
        if (multibase_key::key_info_type(&key_info) == multibase_key::key_type_secp256k1()) {
            option::some(multibase_key::key_info_bytes(&key_info))
        } else {
            option::none()
        }
    }

    /// Parse a did:key identifier and check if it's a Secp256r1 key
    /// 
    /// @param did_key_str - The did:key identifier string
    /// @return - Option containing the raw Secp256r1 public key bytes, or none if not a Secp256r1 key or invalid
    public fun parse_secp256r1(did_key_str: &String): Option<vector<u8>> {
        // Try to parse the did:key identifier
        let parse_result = parse_option(did_key_str);
        
        if (option::is_none(&parse_result)) {
            return option::none()
        };
        
        let key_info = option::extract(&mut parse_result);
        
        // Check if it's a Secp256r1 key
        if (multibase_key::key_info_type(&key_info) == multibase_key::key_type_ecdsar1()) {
            option::some(multibase_key::key_info_bytes(&key_info))
        } else {
            option::none()
        }
    }

    /// Helper function to parse a did:key identifier, returning an Option instead of aborting on failure
    /// 
    /// @param did_key_str - The did:key identifier string
    /// @return - Option containing a KeyInfo struct with key_type and key_bytes, or none if invalid
    fun parse_option(did_key_str: &String): Option<multibase_key::KeyInfo> {
        // Verify the did:key prefix
        let did_key_bytes = string::bytes(did_key_str);
        let prefix_len = std::vector::length(&DID_KEY_METHOD);
        
        // Check if the string is long enough and starts with "did:key:"
        if (std::vector::length(did_key_bytes) <= prefix_len) {
            return option::none()
        };
        
        let i = 0;
        let prefix_match = true;
        while (i < prefix_len) {
            if (*std::vector::borrow(did_key_bytes, i) != *std::vector::borrow(&DID_KEY_METHOD, i)) {
                prefix_match = false;
                break
            };
            i = i + 1;
        };
        
        if (!prefix_match) {
            return option::none()
        };
        
        // Extract the multibase-encoded part
        let multibase_part = string::sub_string(did_key_str, prefix_len, string::length(did_key_str));
        
        // Use multibase_key::decode_with_type_option to decode the multibase-encoded part
        multibase_key::decode_with_type_option(&multibase_part)
    }

    #[test]
    fun test_generate_parse_ed25519() {
        // Create a test Ed25519 public key (32 bytes)
        let pubkey = std::vector::empty<u8>();
        let i = 0;
        while (i < 32) {
            std::vector::push_back(&mut pubkey, (i as u8));
            i = i + 1;
        };
        
        // Generate a did:key identifier
        let did_key = generate_ed25519(&pubkey);
        
        // Verify it starts with "did:key:"
        let prefix = string::utf8(DID_KEY_METHOD);
        assert!(string::sub_string(&did_key, 0, string::length(&prefix)) == prefix, ETestAssertionFailed + 1);
        
        // Parse the did:key identifier
        let (key_type, parsed_key) = parse(&did_key);
        
        // Verify the key type and key bytes
        assert!(key_type == multibase_key::key_type_ed25519(), ETestAssertionFailed + 2);
        assert!(parsed_key == pubkey, ETestAssertionFailed + 3);
        
        // Test the specific parse_ed25519 function
        let parsed_key_opt = parse_ed25519(&did_key);
        assert!(option::is_some(&parsed_key_opt), ETestAssertionFailed + 4);
        assert!(option::extract(&mut parsed_key_opt) == pubkey, ETestAssertionFailed + 5);
    }

    #[test]
    fun test_generate_parse_secp256k1() {
        // Create a test Secp256k1 public key (33 bytes)
        let pubkey = std::vector::empty<u8>();
        let i = 0;
        while (i < 33) {
            std::vector::push_back(&mut pubkey, (i as u8));
            i = i + 1;
        };
        
        // Generate a did:key identifier
        let did_key = generate_secp256k1(&pubkey);
        
        // Verify it starts with "did:key:"
        let prefix = string::utf8(DID_KEY_METHOD);
        assert!(string::sub_string(&did_key, 0, string::length(&prefix)) == prefix, ETestAssertionFailed + 6);
        
        // Parse the did:key identifier
        let (key_type, parsed_key) = parse(&did_key);
        
        // Verify the key type and key bytes
        assert!(key_type == multibase_key::key_type_secp256k1(), ETestAssertionFailed + 7);
        assert!(parsed_key == pubkey, ETestAssertionFailed + 8);
        
        // Test the specific parse_secp256k1 function
        let parsed_key_opt = parse_secp256k1(&did_key);
        assert!(option::is_some(&parsed_key_opt), ETestAssertionFailed + 9);
        assert!(option::extract(&mut parsed_key_opt) == pubkey, ETestAssertionFailed + 10);
    }

    #[test]
    fun test_generate_parse_secp256r1() {
        // Create a test Secp256r1 public key (33 bytes)
        let pubkey = std::vector::empty<u8>();
        let i = 0;
        while (i < 33) {
            std::vector::push_back(&mut pubkey, (i as u8));
            i = i + 1;
        };
        
        // Generate a did:key identifier
        let did_key = generate_secp256r1(&pubkey);
        
        // Verify it starts with "did:key:"
        let prefix = string::utf8(DID_KEY_METHOD);
        assert!(string::sub_string(&did_key, 0, string::length(&prefix)) == prefix, ETestAssertionFailed + 11);
        
        // Parse the did:key identifier
        let (key_type, parsed_key) = parse(&did_key);
        
        // Verify the key type and key bytes
        assert!(key_type == multibase_key::key_type_ecdsar1(), ETestAssertionFailed + 12);
        assert!(parsed_key == pubkey, ETestAssertionFailed + 13);
        
        // Test the specific parse_secp256r1 function
        let parsed_key_opt = parse_secp256r1(&did_key);
        assert!(option::is_some(&parsed_key_opt), ETestAssertionFailed + 14);
        assert!(option::extract(&mut parsed_key_opt) == pubkey, ETestAssertionFailed + 15);
    }

    #[test]
    fun test_cross_key_type_parsing() {
        // Create a test Ed25519 public key (32 bytes)
        let ed25519_key = std::vector::empty<u8>();
        let i = 0;
        while (i < 32) {
            std::vector::push_back(&mut ed25519_key, (i as u8));
            i = i + 1;
        };
        
        // Generate a did:key identifier for Ed25519
        let ed25519_did_key = generate_ed25519(&ed25519_key);
        
        // Try to parse as Secp256k1 - should return none
        let secp256k1_key_opt = parse_secp256k1(&ed25519_did_key);
        assert!(option::is_none(&secp256k1_key_opt), ETestAssertionFailed + 16);
        
        // Try to parse as Secp256r1 - should return none
        let secp256r1_key_opt = parse_secp256r1(&ed25519_did_key);
        assert!(option::is_none(&secp256r1_key_opt), ETestAssertionFailed + 17);
    }

    #[test]
    fun test_invalid_did_key_format() {
        // Test with an invalid did:key string
        let invalid_did_key = string::utf8(b"invalid:did:key");
        let parse_result = parse_option(&invalid_did_key);
        assert!(option::is_none(&parse_result), ETestAssertionFailed + 18);
        
        // Test with a string that starts with "did:" but is not a valid did:key
        let invalid_did = string::utf8(b"did:method:identifier");
        let parse_result = parse_option(&invalid_did);
        assert!(option::is_none(&parse_result), ETestAssertionFailed + 19);
    }

    #[test]
    #[expected_failure(abort_code = ErrorInvalidDidKeyFormat)]
    fun test_parse_invalid_did_key() {
        // This should abort with ErrorInvalidDidKeyFormat
        let invalid_did_key = string::utf8(b"invalid:did:key");
        parse(&invalid_did_key);
    }
} 