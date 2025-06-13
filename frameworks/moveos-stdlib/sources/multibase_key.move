// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

/// Defines the `multibase_key` module, providing key-type aware encoding and decoding
/// operations for cryptographic keys with multicodec prefixes.
///
/// ## Overview
///
/// This module builds on top of `multibase_codec` to provide specialized encoding/decoding
/// for cryptographic keys. It handles:
///
/// 1. Key type enumeration (Ed25519, Secp256k1, Secp256r1)
/// 2. Multicodec prefixes for different key types
/// 3. Key length validation
/// 4. Encoding/decoding with type information
///
/// ## Key Types and Multicodec Prefixes
///
/// * Ed25519: KEY_TYPE_ED25519 = 1, multicodec prefix = 0xed01
/// * Secp256k1: KEY_TYPE_SECP256K1 = 2, multicodec prefix = 0xe701
/// * Secp256r1 (ECDSA P-256): KEY_TYPE_ECDSAR1 = 3, multicodec prefix = 0x1200
///
/// The encoding process adds the appropriate multicodec prefix to the raw key bytes
/// before applying base58btc encoding.

module moveos_std::multibase_key {
    use std::string::String;
    use std::vector;
    use std::option::{Self, Option};
    use moveos_std::multibase_codec;

    /// Error when the Ed25519 key length is invalid
    const ErrorInvalidEd25519KeyLength: u64 = 1;
    /// Error when the Secp256k1 key length is invalid
    const ErrorInvalidSecp256k1KeyLength: u64 = 2;
    /// Error when the Secp256r1 key length is invalid
    const ErrorInvalidSecp256r1KeyLength: u64 = 3;
    /// Error when the did:key identifier is invalid
    const ErrorInvalidDidKeyIdentifier: u64 = 4;
    /// Error when an unsupported key type is used
    const ErrorUnsupportedKeyType: u64 = 5;
    /// Error when the format of the publicKeyMultibase string is invalid or cannot be parsed
    const ErrorInvalidPublicKeyMultibaseFormat: u64 = 6;
    // General test failure code for asserts
    const ETestAssertionFailed: u64 = 100;

    /// The length of Ed25519 public keys in bytes
    const ED25519_PUBLIC_KEY_LENGTH: u64 = 32;
    /// The length of Secp256k1 compressed public keys in bytes
    const SECP256K1_COMPRESSED_PUBLIC_KEY_LENGTH: u64 = 33;
    /// The length of Secp256r1 compressed public keys in bytes
    const ECDSAR1_COMPRESSED_PUBLIC_KEY_LENGTH: u64 = 33;

    // Key type constants as u8 values for efficient storage and comparison
    const KEY_TYPE_ED25519: u8 = 1;
    const KEY_TYPE_SECP256K1: u8 = 2;
    const KEY_TYPE_ECDSAR1: u8 = 3;

    // Multicodec prefixes for key types
    const MULTICODEC_ED25519_PREFIX: vector<u8> = vector[0xed, 0x01];
    const MULTICODEC_SECP256K1_PREFIX: vector<u8> = vector[0xe7, 0x01];
    const MULTICODEC_ECDSA_R1_PREFIX: vector<u8> = vector[0x12, 0x00];

    /// A struct to hold the key type and raw key bytes
    /// Used as a workaround for Move's lack of support for tuple types in Option
    struct KeyInfo has copy, drop, store{
        key_type: u8,
        key_bytes: vector<u8>,
    }

    /// Get the key type from a KeyInfo struct
    public fun key_info_type(key_info: &KeyInfo): u8 {
        key_info.key_type
    }

    /// Get the key bytes from a KeyInfo struct
    public fun key_info_bytes(key_info: &KeyInfo): vector<u8> {
        key_info.key_bytes
    }

    /// Returns the key type constant for Ed25519
    public fun key_type_ed25519(): u8 {
        KEY_TYPE_ED25519
    }

    /// Returns the key type constant for Secp256k1
    public fun key_type_secp256k1(): u8 {
        KEY_TYPE_SECP256K1
    }

    /// Returns the key type constant for Secp256r1 (ECDSA P-256)
    public fun key_type_ecdsar1(): u8 {
        KEY_TYPE_ECDSAR1
    }

    /// Get the multicodec prefix for a given key type
    /// 
    /// @param key_type - The key type (1=Ed25519, 2=Secp256k1, 3=Secp256r1)
    /// @return - The multicodec prefix bytes
    public fun multicodec_prefix_for_type(key_type: u8): vector<u8> {
        if (key_type == KEY_TYPE_ED25519) {
            MULTICODEC_ED25519_PREFIX
        } else if (key_type == KEY_TYPE_SECP256K1) {
            MULTICODEC_SECP256K1_PREFIX
        } else if (key_type == KEY_TYPE_ECDSAR1) {
            MULTICODEC_ECDSA_R1_PREFIX
        } else {
            abort ErrorUnsupportedKeyType
        }
    }

    /// Validate key length based on key type
    /// 
    /// @param key_bytes - The raw key bytes
    /// @param key_type - The key type
    fun validate_key_length(key_bytes: &vector<u8>, key_type: u8) {
        if (key_type == KEY_TYPE_ED25519) {
            assert!(vector::length(key_bytes) == ED25519_PUBLIC_KEY_LENGTH, ErrorInvalidEd25519KeyLength);
        } else if (key_type == KEY_TYPE_SECP256K1) {
            assert!(vector::length(key_bytes) == SECP256K1_COMPRESSED_PUBLIC_KEY_LENGTH, ErrorInvalidSecp256k1KeyLength);
        } else if (key_type == KEY_TYPE_ECDSAR1) {
            assert!(vector::length(key_bytes) == ECDSAR1_COMPRESSED_PUBLIC_KEY_LENGTH, ErrorInvalidSecp256r1KeyLength);
        } else {
            abort ErrorUnsupportedKeyType
        }
    }

    /// Encodes a public key with multicodec prefix and multibase encoding
    /// 
    /// @param pubkey - The raw public key bytes
    /// @param key_type - The key type (1=Ed25519, 2=Secp256k1, 3=Secp256r1)
    /// @return - A multibase encoded string with appropriate prefix
    public fun encode_with_type(pubkey: &vector<u8>, key_type: u8): String {
        // Validate key length based on type
        validate_key_length(pubkey, key_type);
        
        // Get multicodec prefix for this key type
        let prefixed_key = multicodec_prefix_for_type(key_type);
        
        // Append raw key bytes to prefix
        vector::append(&mut prefixed_key, *pubkey);
        
        // Encode with base58btc
        multibase_codec::encode_base58btc(&prefixed_key)
    }

    /// Decodes a multibase-encoded key string with multicodec prefix
    /// 
    /// @param encoded_str - The multibase encoded key string
    /// @return - A tuple of (key_type, raw_key_bytes), or abort if invalid
    public fun decode_with_type(encoded_str: &String): (u8, vector<u8>) {
        // First decode the multibase string
        let decoded_bytes_opt = multibase_codec::decode(encoded_str);
        if (option::is_none(&decoded_bytes_opt)) {
            abort ErrorInvalidPublicKeyMultibaseFormat
        };
        
        let decoded_bytes = option::extract(&mut decoded_bytes_opt);
        
        // Check minimum length (at least 2 bytes for multicodec prefix)
        if (vector::length(&decoded_bytes) < 2) {
            abort ErrorInvalidDidKeyIdentifier
        };
        
        // Extract multicodec prefix
        let first_byte = *vector::borrow(&decoded_bytes, 0);
        let second_byte = *vector::borrow(&decoded_bytes, 1);
        
        // Check for Ed25519 multicodec: 0xed01
        if (first_byte == 0xed && second_byte == 0x01) {
            // Extract raw Ed25519 public key (32 bytes after 2-byte prefix)
            if (vector::length(&decoded_bytes) == 34) { // 2 bytes prefix + 32 bytes key
                let raw_key = vector::empty<u8>();
                let i = 2;
                while (i < 34) {
                    vector::push_back(&mut raw_key, *vector::borrow(&decoded_bytes, i));
                    i = i + 1;
                };
                return (KEY_TYPE_ED25519, raw_key)
            }
        }
        // Check for Secp256k1 multicodec: 0xe701  
        else if (first_byte == 0xe7 && second_byte == 0x01) {
            // Extract raw Secp256k1 public key (33 bytes after 2-byte prefix)
            if (vector::length(&decoded_bytes) == 35) { // 2 bytes prefix + 33 bytes key
                let raw_key = vector::empty<u8>();
                let i = 2;
                while (i < 35) {
                    vector::push_back(&mut raw_key, *vector::borrow(&decoded_bytes, i));
                    i = i + 1;
                };
                return (KEY_TYPE_SECP256K1, raw_key)
            }
        }
        // Check for ECDSA R1 multicodec: 0x1200
        else if (first_byte == 0x12 && second_byte == 0x00) {
            // Extract raw ECDSA R1 public key (33 bytes after 2-byte prefix)
            if (vector::length(&decoded_bytes) == 35) { // 2 bytes prefix + 33 bytes key
                let raw_key = vector::empty<u8>();
                let i = 2;
                while (i < 35) {
                    vector::push_back(&mut raw_key, *vector::borrow(&decoded_bytes, i));
                    i = i + 1;
                };
                return (KEY_TYPE_ECDSAR1, raw_key)
            }
        };   
        
        // Unsupported multicodec or invalid format
        abort ErrorInvalidDidKeyIdentifier
    }

    /// Encodes an Ed25519 public key using base58btc with multibase prefix
    /// 
    /// @param pubkey - The raw Ed25519 public key bytes
    /// @return - A multibase encoded string with 'z' prefix
    public fun encode_ed25519_key(pubkey: &vector<u8>): String {
        encode_with_type(pubkey, KEY_TYPE_ED25519)
    }

    /// Encodes a Secp256k1 compressed public key using base58btc with multibase prefix
    /// 
    /// @param pubkey - The raw Secp256k1 compressed public key bytes (33 bytes)
    /// @return - A multibase encoded string with 'z' prefix
    public fun encode_secp256k1_key(pubkey: &vector<u8>): String {
        encode_with_type(pubkey, KEY_TYPE_SECP256K1)
    }

    /// Encodes a Secp256r1 compressed public key using base58btc with multibase prefix
    /// 
    /// @param pubkey - The raw Secp256r1 compressed public key bytes (33 bytes)
    /// @return - A multibase encoded string with 'z' prefix
    public fun encode_ecdsar1_key(pubkey: &vector<u8>): String {
        encode_with_type(pubkey, KEY_TYPE_ECDSAR1)
    }

    /// Decodes a multibase-encoded Ed25519 public key
    /// 
    /// @param pk_mb_str - The multibase encoded Ed25519 public key string
    /// @return - Option containing the decoded public key bytes, or none if decoding fails
    public fun decode_ed25519_key(pk_mb_str: &String): Option<vector<u8>> {
        let decoded_result = decode_with_type_option(pk_mb_str);
        
        if (option::is_none(&decoded_result)) {
            return option::none()
        };
        
        let key_info = option::extract(&mut decoded_result);
        
        if (key_info.key_type == KEY_TYPE_ED25519) {
            option::some(key_info.key_bytes)
        } else {
            option::none()
        }
    }

    /// Decodes a multibase-encoded Secp256k1 compressed public key
    /// 
    /// @param pk_mb_str - The multibase encoded Secp256k1 public key string
    /// @return - Option containing the decoded public key bytes, or none if decoding fails
    public fun decode_secp256k1_key(pk_mb_str: &String): Option<vector<u8>> {
        let decoded_result = decode_with_type_option(pk_mb_str);
        
        if (option::is_none(&decoded_result)) {
            return option::none()
        };
        
        let key_info = option::extract(&mut decoded_result);
        
        if (key_info.key_type == KEY_TYPE_SECP256K1) {
            option::some(key_info.key_bytes)
        } else {
            option::none()
        }
    }

    /// Decodes a Secp256r1 public key from a multibase encoded string
    /// 
    /// @param pk_mb_str - The multibase encoded string
    /// @return - Option containing the raw Secp256r1 public key bytes, or none if decoding fails
    public fun decode_secp256r1_key(pk_mb_str: &String): Option<vector<u8>> {
        let decoded_result = decode_with_type_option(pk_mb_str);
        
        if (option::is_none(&decoded_result)) {
            return option::none()
        };
        
        let key_info = option::extract(&mut decoded_result);
        
        if (key_info.key_type == KEY_TYPE_ECDSAR1) {
            option::some(key_info.key_bytes)
        } else {
            option::none()
        }
    }

    /// Helper function to decode a multibase-encoded key string with multicodec prefix,
    /// returning an Option instead of aborting on failure
    /// 
    /// @param encoded_str - The multibase encoded key string
    /// @return - Option containing a KeyInfo struct with key_type and key_bytes, or none if invalid
    public fun decode_with_type_option(encoded_str: &String): Option<KeyInfo> {
        // First decode the multibase string
        let decoded_bytes_opt = multibase_codec::decode(encoded_str);
        if (option::is_none(&decoded_bytes_opt)) {
            return option::none()
        };
        
        let decoded_bytes = option::extract(&mut decoded_bytes_opt);
        
        // Check minimum length (at least 2 bytes for multicodec prefix)
        if (vector::length(&decoded_bytes) < 2) {
            return option::none()
        };
        
        // Extract multicodec prefix
        let first_byte = *vector::borrow(&decoded_bytes, 0);
        let second_byte = *vector::borrow(&decoded_bytes, 1);
        
        // Check for Ed25519 multicodec: 0xed01
        if (first_byte == 0xed && second_byte == 0x01) {
            // Extract raw Ed25519 public key (32 bytes after 2-byte prefix)
            if (vector::length(&decoded_bytes) == 34) { // 2 bytes prefix + 32 bytes key
                let raw_key = vector::empty<u8>();
                let i = 2;
                while (i < 34) {
                    vector::push_back(&mut raw_key, *vector::borrow(&decoded_bytes, i));
                    i = i + 1;
                };
                return option::some(KeyInfo { key_type: KEY_TYPE_ED25519, key_bytes: raw_key })
            }
        }
        // Check for Secp256k1 multicodec: 0xe701  
        else if (first_byte == 0xe7 && second_byte == 0x01) {
            // Extract raw Secp256k1 public key (33 bytes after 2-byte prefix)
            if (vector::length(&decoded_bytes) == 35) { // 2 bytes prefix + 33 bytes key
                let raw_key = vector::empty<u8>();
                let i = 2;
                while (i < 35) {
                    vector::push_back(&mut raw_key, *vector::borrow(&decoded_bytes, i));
                    i = i + 1;
                };
                return option::some(KeyInfo { key_type: KEY_TYPE_SECP256K1, key_bytes: raw_key })
            }
        }
        // Check for ECDSA R1 multicodec: 0x1200
        else if (first_byte == 0x12 && second_byte == 0x00) {
            // Extract raw ECDSA R1 public key (33 bytes after 2-byte prefix)
            if (vector::length(&decoded_bytes) == 35) { // 2 bytes prefix + 33 bytes key
                let raw_key = vector::empty<u8>();
                let i = 2;
                while (i < 35) {
                    vector::push_back(&mut raw_key, *vector::borrow(&decoded_bytes, i));
                    i = i + 1;
                };
                return option::some(KeyInfo { key_type: KEY_TYPE_ECDSAR1, key_bytes: raw_key })
            }
        };   
        
        // Unsupported multicodec or invalid format
        option::none()
    }

    #[test]
    fun test_encode_decode_ed25519_key() {
        // Create a test Ed25519 public key (32 bytes)
        let pubkey = vector::empty<u8>();
        let i = 0;
        while (i < ED25519_PUBLIC_KEY_LENGTH) {
            vector::push_back(&mut pubkey, (i as u8));
            i = i + 1;
        };
        
        let encoded = encode_ed25519_key(&pubkey);
        let decoded_opt = decode_ed25519_key(&encoded);
        
        assert!(option::is_some(&decoded_opt), ETestAssertionFailed + 1);
        assert!(option::extract(&mut decoded_opt) == pubkey, ETestAssertionFailed + 2);
        
        // Test with generic encode/decode
        let encoded2 = encode_with_type(&pubkey, KEY_TYPE_ED25519);
        assert!(encoded == encoded2, ETestAssertionFailed + 3);
        
        let (key_type, raw_key) = decode_with_type(&encoded);
        assert!(key_type == KEY_TYPE_ED25519, ETestAssertionFailed + 4);
        assert!(raw_key == pubkey, ETestAssertionFailed + 5);
    }

    #[test]
    fun test_encode_decode_secp256k1_key() {
        // Create a test Secp256k1 public key (33 bytes)
        let pubkey = vector::empty<u8>();
        let i = 0;
        while (i < SECP256K1_COMPRESSED_PUBLIC_KEY_LENGTH) {
            vector::push_back(&mut pubkey, (i as u8));
            i = i + 1;
        };
        
        let encoded = encode_secp256k1_key(&pubkey);
        let decoded_opt = decode_secp256k1_key(&encoded);
        
        assert!(option::is_some(&decoded_opt), ETestAssertionFailed + 6);
        assert!(option::extract(&mut decoded_opt) == pubkey, ETestAssertionFailed + 7);
        
        // Test with generic encode/decode
        let encoded2 = encode_with_type(&pubkey, KEY_TYPE_SECP256K1);
        assert!(encoded == encoded2, ETestAssertionFailed + 8);
        
        let (key_type, raw_key) = decode_with_type(&encoded);
        assert!(key_type == KEY_TYPE_SECP256K1, ETestAssertionFailed + 9);
        assert!(raw_key == pubkey, ETestAssertionFailed + 10);
    }

    #[test]
    fun test_encode_decode_ecdsar1_key() {
        // Create a test Secp256r1 public key (33 bytes)
        let pubkey = vector::empty<u8>();
        let i = 0;
        while (i < ECDSAR1_COMPRESSED_PUBLIC_KEY_LENGTH) {
            vector::push_back(&mut pubkey, (i as u8));
            i = i + 1;
        };
        
        let encoded = encode_ecdsar1_key(&pubkey);
        let decoded_opt = decode_secp256r1_key(&encoded);
        
        assert!(option::is_some(&decoded_opt), ETestAssertionFailed + 11);
        assert!(option::extract(&mut decoded_opt) == pubkey, ETestAssertionFailed + 12);
        
        // Test with generic encode/decode
        let encoded2 = encode_with_type(&pubkey, KEY_TYPE_ECDSAR1);
        assert!(encoded == encoded2, ETestAssertionFailed + 13);
        
        let (key_type, raw_key) = decode_with_type(&encoded);
        assert!(key_type == KEY_TYPE_ECDSAR1, ETestAssertionFailed + 14);
        assert!(raw_key == pubkey, ETestAssertionFailed + 15);
    }

    #[test]
    #[expected_failure(abort_code = ErrorInvalidEd25519KeyLength)]
    fun test_invalid_ed25519_key_length() {
        // Try to encode a key with invalid length
        let invalid_key = vector::empty<u8>();
        vector::push_back(&mut invalid_key, 1);
        encode_ed25519_key(&invalid_key);
    }

    #[test]
    #[expected_failure(abort_code = ErrorInvalidSecp256k1KeyLength)]
    fun test_invalid_secp256k1_key_length() {
        // Try to encode a key with invalid length
        let invalid_key = vector::empty<u8>();
        vector::push_back(&mut invalid_key, 1);
        encode_secp256k1_key(&invalid_key);
    }

    #[test]
    #[expected_failure(abort_code = ErrorInvalidSecp256r1KeyLength)]
    fun test_invalid_secp256r1_key_length() {
        // Try to encode a key with invalid length
        let invalid_key = vector::empty<u8>();
        vector::push_back(&mut invalid_key, 1);
        encode_ecdsar1_key(&invalid_key);
    }

    #[test]
    #[expected_failure(abort_code = ErrorUnsupportedKeyType)]
    fun test_unsupported_key_type() {
        // Try to encode with unsupported key type
        let key = vector::empty<u8>();
        vector::push_back(&mut key, 1);
        encode_with_type(&key, 99); // 99 is not a valid key type
    }

    #[test]
    fun test_decode_invalid_format() {
        // Test with invalid multibase string
        let invalid_str = std::string::utf8(b"not-a-valid-multibase");
        let decoded_opt = decode_ed25519_key(&invalid_str);
        assert!(option::is_none(&decoded_opt), ETestAssertionFailed + 16);
        
        // Test with valid multibase but wrong key type
        let ed25519_key = vector::empty<u8>();
        let i = 0;
        while (i < ED25519_PUBLIC_KEY_LENGTH) {
            vector::push_back(&mut ed25519_key, (i as u8));
            i = i + 1;
        };
        
        let encoded_ed25519 = encode_ed25519_key(&ed25519_key);
        
        // Try to decode as Secp256k1
        let decoded_secp256k1_opt = decode_secp256k1_key(&encoded_ed25519);
        assert!(option::is_none(&decoded_secp256k1_opt), ETestAssertionFailed + 17);
    }
}
