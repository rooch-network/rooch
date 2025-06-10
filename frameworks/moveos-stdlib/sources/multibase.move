// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

/// Defines the `multibase` module, a protocol for self-identifying base encodings
/// for binary data expressed in text formats. This module allows disambiguation
/// of the encoding (e.g., base16, base58btc, base64pad) directly from the
/// encoded string itself by prepending a unique prefix character.
///
/// ## Overview
///
/// When binary data is encoded into text, various base encodings can be used.
/// Multibase prepends a single character code to the base-encoded data,
/// indicating which encoding was used. This allows data to be self-describing
/// as it travels beyond its original context.
///
/// The format is: `<base-encoding-code-point><base-encoded-data>`
///
/// ## Supported Encodings
///
/// This module currently supports the following encodings, along with their
/// respective multibase prefixes and standard names:
///
/// *   **Base16 (Hexadecimal)**:
///     *   Prefix: `'f'` (ASCII: 102)
///     *   Name: `"base16"` (alias: `"hex"`)
///     *   Standard: RFC4648 (lowercase output)
/// *   **Base58 Bitcoin (base58btc)**:
///     *   Prefix: `'z'` (ASCII: 122)
///     *   Name: `"base58btc"`
///     *   Standard: Used in Bitcoin, common for cryptographic keys.
/// *   **Base64 with Padding (base64pad)**:
///     *   Prefix: `'M'` (ASCII: 77)
///     *   Name: `"base64pad"`
///     *   Standard: RFC4648 with padding characters (`=`).
///
/// The module is designed to be extensible for other encodings in the future.
///
/// ## Error Handling
///
/// Functions that can fail (e.g., `decode`, `encode` with an unsupported encoding)
/// return an `Option` type. Specific error codes are defined for internal assertions
/// and can be used in tests (e.g., `ErrorInvalidMultibasePrefix`, `ErrorInvalidEd25519KeyLength`).
/// Test assertions use `ETestAssertionFailed` plus an offset for unique error codes.
///
/// For more details on the Multibase standard, see: [https://github.com/multiformats/multibase](https://github.com/multiformats/multibase)

module moveos_std::multibase {
    use std::string::{Self, String};
    use std::vector;
    use std::option::{Self, Option, some, none};
    use moveos_std::base58;
    use moveos_std::base64;
    use moveos_std::hex;
    use moveos_std::string_utils;

    /// Error when an invalid multibase prefix is provided
    const ErrorInvalidMultibasePrefix: u64 = 1;
    /// Error when an unsupported encoding base is used
    const ErrorUnsupportedBase: u64 = 2;
    /// Error when an invalid base58 character is encountered
    const ErrorInvalidBase58Char: u64 = 3;
    /// Error when base58 decoding fails
    const ErrorBase58DecodingFailed: u64 = 4;
    /// Error when the Ed25519 key length is invalid
    const ErrorInvalidEd25519KeyLength: u64 = 5;
    /// Error when the Secp256r1 key length is invalid
    const ErrorInvalidSecp256r1KeyLength: u64 = 7;
    /// Error when the encoding process fails
    const ErrorEncodingFailed: u64 = 6;
    // General test failure code for asserts
    const ETestAssertionFailed: u64 = 100;

    /// The length of Ed25519 public keys in bytes
    const ED25519_PUBLIC_KEY_LENGTH: u64 = 32;
    /// The length of Secp256k1 compressed public keys in bytes
    const SECP256K1_COMPRESSED_PUBLIC_KEY_LENGTH: u64 = 33;
    /// The length of Secp256r1 compressed public keys in bytes
    const SECP256R1_COMPRESSED_PUBLIC_KEY_LENGTH: u64 = 33;

    /// The prefix for Ed25519 public keys in base58btc encoding ('z' in ASCII)
    const BASE58BTC_PREFIX: u8 = 122;
    /// The prefix for base32 encoding ('b' in ASCII)
    const BASE32_PREFIX: u8 = 98;
    /// The prefix for base64pad encoding ('M' in ASCII)
    const BASE64PAD_PREFIX: u8 = 77; // 'M' for base64pad (RFC4648 with padding)
    /// The prefix for base16 (hex) encoding ('f' in ASCII)
    const BASE16_PREFIX: u8 = 102;

    // Multicodec prefixes for did:key identifiers
    /// Ed25519 multicodec prefix (0xed01)
    const MULTICODEC_ED25519_PREFIX: vector<u8> = vector[0xed, 0x01];
    /// Secp256k1 multicodec prefix (0xe701)
    const MULTICODEC_SECP256K1_PREFIX: vector<u8> = vector[0xe7, 0x01];

    // Encoding name constants
    const ENCODING_BASE58BTC: vector<u8> = b"base58btc";
    const ENCODING_BASE32: vector<u8> = b"base32";
    const ENCODING_BASE64PAD: vector<u8> = b"base64pad";
    const ENCODING_BASE16: vector<u8> = b"base16";
    const ENCODING_HEX: vector<u8> = b"hex";

    /// Returns the name of base58btc encoding
    public fun base58btc_name(): String {
        string::utf8(ENCODING_BASE58BTC)
    }

    /// Returns the name of base32 encoding
    public fun base32_name(): String {
        string::utf8(ENCODING_BASE32)
    }

    /// Returns the name of base64pad encoding (RFC4648 with padding)
    public fun base64pad_name(): String {
        string::utf8(ENCODING_BASE64PAD)
    }

    /// Returns the name of base16/hex encoding
    public fun base16_name(): String {
        string::utf8(ENCODING_BASE16)
    }

    /// Returns the alternate name (hex) for base16 encoding
    public fun hex_name(): String {
        string::utf8(ENCODING_HEX)
    }

    /// Encodes bytes using base58btc and adds the multibase prefix 'z'
    /// 
    /// @param bytes - The raw bytes to encode
    /// @return - A multibase encoded string with 'z' prefix
    public fun encode_base58btc(bytes: &vector<u8>): String {
        let encoded_bytes = base58::encoding(bytes);
        // Prepend the 'z' prefix
        vector::insert(&mut encoded_bytes, 0, BASE58BTC_PREFIX);
        string::utf8(encoded_bytes)
    }

    /// Encodes bytes using base64pad (RFC4648 with padding) and adds the multibase prefix 'M'
    /// 
    /// @param bytes - The raw bytes to encode
    /// @return - A multibase encoded string with 'M' prefix
    public fun encode_base64pad(bytes: &vector<u8>): String {
        let encoded_bytes = base64::encode(bytes); // Assumes moveos_std::base64::encode produces padded output
        // Prepend the 'M' prefix
        vector::insert(&mut encoded_bytes, 0, BASE64PAD_PREFIX);
        string::utf8(encoded_bytes)
    }

    /// Encodes bytes using base16 (hex) and adds the multibase prefix 'f'
    /// 
    /// @param bytes - The raw bytes to encode
    /// @return - A multibase encoded string with 'f' prefix
    public fun encode_base16(bytes: &vector<u8>): String {
        let encoded_bytes = hex::encode(*bytes);
        // Prepend the 'f' prefix
        vector::insert(&mut encoded_bytes, 0, BASE16_PREFIX);
        string::utf8(encoded_bytes)
    }

    /// Encodes bytes using a specified encoding format
    /// 
    /// @param bytes - The raw bytes to encode
    /// @param encoding - The encoding format to use (e.g., "base58btc", "base64pad")
    /// @return - Option containing a multibase encoded string, or none if encoding is unsupported
    public fun encode(bytes: &vector<u8>, encoding: &String): Option<String> {
        let encoding_lowercase = string_utils::to_lower_case(encoding);
        
        if (encoding_lowercase == base58btc_name()) {
            return some(encode_base58btc(bytes))
        };
        
        if (encoding_lowercase == base64pad_name()) {
            return some(encode_base64pad(bytes))
        };
        
        if (encoding_lowercase == base16_name() || encoding_lowercase == hex_name()) {
            return some(encode_base16(bytes))
        };
        
        // Other encoding formats could be added here
        none()
    }

    /// Encodes an Ed25519 public key using base58btc with multibase prefix
    /// 
    /// @param pubkey - The raw Ed25519 public key bytes
    /// @return - A multibase encoded string with 'z' prefix
    public fun encode_ed25519_key(pubkey: &vector<u8>): String {
        assert!(vector::length(pubkey) == ED25519_PUBLIC_KEY_LENGTH, ErrorInvalidEd25519KeyLength);
        encode_base58btc(pubkey)
    }

    /// Encodes a Secp256k1 compressed public key using base58btc with multibase prefix
    /// 
    /// @param pubkey - The raw Secp256k1 compressed public key bytes (33 bytes)
    /// @return - A multibase encoded string with 'z' prefix
    public fun encode_secp256k1_key(pubkey: &vector<u8>): String {
        assert!(vector::length(pubkey) == SECP256K1_COMPRESSED_PUBLIC_KEY_LENGTH, ErrorInvalidEd25519KeyLength);
        encode_base58btc(pubkey)
    }

    /// Encodes a Secp256r1 compressed public key using base58btc with multibase prefix
    /// 
    /// @param pubkey - The raw Secp256r1 compressed public key bytes (33 bytes)
    /// @return - A multibase encoded string with 'z' prefix
    public fun encode_secp256r1_key(pubkey: &vector<u8>): String {
        assert!(vector::length(pubkey) == SECP256R1_COMPRESSED_PUBLIC_KEY_LENGTH, ErrorInvalidSecp256r1KeyLength);
        encode_base58btc(pubkey)
    }

    /// Alias for encode_secp256r1_key, encodes an ECDSA R1 (P-256) compressed public key using base58btc with multibase prefix
    /// 
    /// @param pubkey - The raw ECDSA R1 compressed public key bytes (33 bytes)
    /// @return - A multibase encoded string with 'z' prefix
    public fun encode_ecdsa_r1_key(pubkey: &vector<u8>): String {
        encode_secp256r1_key(pubkey)
    }

    /// Encodes an Ed25519 public key as a did:key identifier with multicodec prefix
    /// 
    /// @param pubkey - The raw Ed25519 public key bytes (32 bytes)
    /// @return - A did:key identifier string with multicodec prefix (e.g., "z6Mk...")
    public fun encode_ed25519_did_key_identifier(pubkey: &vector<u8>): String {
        assert!(vector::length(pubkey) == ED25519_PUBLIC_KEY_LENGTH, ErrorInvalidEd25519KeyLength);
        
        // Prepend multicodec prefix for Ed25519
        let prefixed_key = MULTICODEC_ED25519_PREFIX;
        vector::append(&mut prefixed_key, *pubkey);
        
        // Encode with base58btc and multibase prefix
        encode_base58btc(&prefixed_key)
    }

    /// Encodes a Secp256k1 compressed public key as a did:key identifier with multicodec prefix
    /// 
    /// @param pubkey - The raw Secp256k1 compressed public key bytes (33 bytes)
    /// @return - A did:key identifier string with multicodec prefix (e.g., "zQ3s...")
    public fun encode_secp256k1_did_key_identifier(pubkey: &vector<u8>): String {
        assert!(vector::length(pubkey) == SECP256K1_COMPRESSED_PUBLIC_KEY_LENGTH, ErrorInvalidEd25519KeyLength);
        
        // Prepend multicodec prefix for Secp256k1
        let prefixed_key = MULTICODEC_SECP256K1_PREFIX;
        vector::append(&mut prefixed_key, *pubkey);
        
        // Encode with base58btc and multibase prefix
        encode_base58btc(&prefixed_key)
    }

    /// Decodes a multibase-encoded string to its raw bytes
    /// 
    /// @param encoded_str - The multibase encoded string
    /// @return - Option containing the decoded bytes, or none if decoding fails
    public fun decode(encoded_str: &String): Option<vector<u8>> {
        let str_bytes = string::bytes(encoded_str);
        let len = vector::length(str_bytes);
        
        // Must have at least a prefix character
        if (len < 1) {
            return none()
        };
        
        // Get the prefix byte
        let prefix = *vector::borrow(str_bytes, 0);
        
        // Handle different encodings based on prefix
        if (prefix == BASE58BTC_PREFIX) {
            // Extract the base58btc encoded part (everything after 'z')
            let payload_str = string::sub_string(encoded_str, 1, string::length(encoded_str));
            
            // Decode using base58
            let payload_bytes = string::bytes(&payload_str);
            
            // Use base58 decoding function from the native module
            option::some(base58::decoding(payload_bytes))
        } else if (prefix == BASE64PAD_PREFIX) {
            // Extract the base64pad encoded part (everything after 'M')
            let payload_str = string::sub_string(encoded_str, 1, string::length(encoded_str));
            
            // Decode using base64
            let payload_bytes = string::bytes(&payload_str);
            
            // Use base64 decoding function from the native module (assumes it handles padding correctly)
            option::some(base64::decode(payload_bytes))
        } else if (prefix == BASE16_PREFIX) {
            // Extract the base16/hex encoded part (everything after 'f')
            let payload_str = string::sub_string(encoded_str, 1, string::length(encoded_str));
            
            // Decode using hex
            let payload_bytes = string::bytes(&payload_str);
            
            // First try using hex::decode_option which returns Option
            let hex_decoded = hex::decode_option(payload_bytes);
            if (option::is_some(&hex_decoded)) {
                return hex_decoded
            };
            
            // If the above fails, return none
            none()
        } else {
            // Other encoding bases could be implemented here
            none()
        }
    }

    /// Decodes a multibase-encoded Ed25519 public key
    /// 
    /// @param pk_mb_str - The multibase encoded Ed25519 public key string
    /// @return - Option containing the decoded public key bytes, or none if decoding fails
    public fun decode_ed25519_key(pk_mb_str: &String): Option<vector<u8>> {
        let decoded_opt = decode(pk_mb_str);
        
        if (option::is_none(&decoded_opt)) {
            return none()
        };
        
        let decoded_bytes = option::extract(&mut decoded_opt);
        
        // Validate the length for Ed25519 keys
        if (vector::length(&decoded_bytes) == ED25519_PUBLIC_KEY_LENGTH) {
            some(decoded_bytes)
        } else {
            // Decoded key has an invalid length for an Ed25519 public key
            none()
        }
    }

    /// Decodes a multibase-encoded Secp256k1 compressed public key
    /// 
    /// @param pk_mb_str - The multibase encoded Secp256k1 public key string
    /// @return - Option containing the decoded public key bytes, or none if decoding fails
    public fun decode_secp256k1_key(pk_mb_str: &String): Option<vector<u8>> {
        let decoded_opt = decode(pk_mb_str);
        
        if (option::is_none(&decoded_opt)) {
            return none()
        };
        
        let decoded_bytes = option::extract(&mut decoded_opt);
        
        // Validate the length for Secp256k1 compressed keys
        if (vector::length(&decoded_bytes) == SECP256K1_COMPRESSED_PUBLIC_KEY_LENGTH) {
            some(decoded_bytes)
        } else {
            // Decoded key has an invalid length for a Secp256k1 public key
            none()
        }
    }

    /// Decodes a Secp256r1 public key from a multibase encoded string
    /// 
    /// @param multibase_str - The multibase encoded string
    /// @return - Option containing the raw Secp256r1 public key bytes, or none if decoding fails
    public fun decode_secp256r1_key(multibase_str: &String): Option<vector<u8>> {
        let bytes = *string::bytes(multibase_str);
        if (vector::length(&bytes) == 0) {
            return none()
        };

        let prefix = vector::borrow(&bytes, 0);
        if (*prefix != BASE58BTC_PREFIX) {
            return none()
        };

        let encoded_bytes = vector::empty<u8>();
        let i = 1;
        while (i < vector::length(&bytes)) {
            vector::push_back(&mut encoded_bytes, *vector::borrow(&bytes, i));
            i = i + 1;
        };

        let decoded_bytes = base58::decoding(&encoded_bytes);
        if (vector::length(&decoded_bytes) != SECP256R1_COMPRESSED_PUBLIC_KEY_LENGTH) {
            return none()
        };

        some(decoded_bytes)
    }

    /// Decodes a did:key identifier to extract the raw public key bytes
    /// 
    /// @param did_key_identifier - The did:key identifier (e.g., "z6Mk..." or "zQ3s...")
    /// @return - Option containing the raw public key bytes, or none if decoding fails
    public fun decode_did_key_identifier(did_key_identifier: &String): Option<vector<u8>> {
        // First decode the multibase string
        let decoded_bytes_opt = decode(did_key_identifier);
        if (option::is_none(&decoded_bytes_opt)) {
            return option::none()
        };
        
        let decoded_bytes = option::destroy_some(decoded_bytes_opt);
        
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
                return option::some(raw_key)
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
                return option::some(raw_key)
            }
        };
        
        // Unsupported multicodec or invalid format
        option::none()
    }

    /// Generate a complete did:key string from an Ed25519 public key
    /// 
    /// @param pubkey - The raw Ed25519 public key bytes (32 bytes)
    /// @return - A complete did:key string (e.g., "did:key:z6Mk...")
    public fun generate_ed25519_did_key_string(pubkey: &vector<u8>): String {
        let identifier = encode_ed25519_did_key_identifier(pubkey);
        let did_key_string = string::utf8(b"did:key:");
        string::append(&mut did_key_string, identifier);
        did_key_string
    }

    /// Generate a complete did:key string from a Secp256k1 public key
    /// 
    /// @param pubkey - The raw Secp256k1 compressed public key bytes (33 bytes)
    /// @return - A complete did:key string (e.g., "did:key:zQ3s...")
    public fun generate_secp256k1_did_key_string(pubkey: &vector<u8>): String {
        let identifier = encode_secp256k1_did_key_identifier(pubkey);
        let did_key_string = string::utf8(b"did:key:");
        string::append(&mut did_key_string, identifier);
        did_key_string
    }

    /// Extract the key type from a did:key identifier
    /// 
    /// @param did_key_identifier - The did:key identifier (e.g., "z6Mk..." or "zQ3s...")
    /// @return - Option containing the key type string ("Ed25519" or "Secp256k1"), or none if unknown
    public fun get_key_type_from_did_key_identifier(did_key_identifier: &String): Option<String> {
        // First decode the multibase string
        let decoded_bytes_opt = decode(did_key_identifier);
        if (option::is_none(&decoded_bytes_opt)) {
            return option::none()
        };
        
        let decoded_bytes = option::destroy_some(decoded_bytes_opt);
        
        // Check minimum length (at least 2 bytes for multicodec prefix)
        if (vector::length(&decoded_bytes) < 2) {
            return option::none()
        };
        
        // Extract multicodec prefix
        let first_byte = *vector::borrow(&decoded_bytes, 0);
        let second_byte = *vector::borrow(&decoded_bytes, 1);
        
        // Check for Ed25519 multicodec: 0xed01
        if (first_byte == 0xed && second_byte == 0x01) {
            return option::some(string::utf8(b"Ed25519"))
        }
        // Check for Secp256k1 multicodec: 0xe701  
        else if (first_byte == 0xe7 && second_byte == 0x01) {
            return option::some(string::utf8(b"Secp256k1"))
        };
        
        // Unsupported multicodec
        option::none()
    }

    /// Gets the multibase prefix character for a given encoding
    /// 
    /// @param encoding_name - The name of the encoding
    /// @return - Option containing the prefix byte, or none if encoding is unknown
    public fun get_prefix_for_encoding(encoding_name: &String): Option<u8> {
        let name = string_utils::to_lower_case(encoding_name);
        
        if (name == base58btc_name()) {
            return some(BASE58BTC_PREFIX)
        };
        
        if (name == base32_name()) {
            return some(BASE32_PREFIX)
        };
        
        if (name == base64pad_name()) {
            return some(BASE64PAD_PREFIX)
        };
        
        if (name == base16_name() || name == hex_name()) {
            return some(BASE16_PREFIX)
        };
        
        none()
    }

    /// Gets the encoding name from a multibase prefix character
    /// 
    /// @param prefix - The multibase prefix byte
    /// @return - Option containing the encoding name, or none if prefix is unknown
    public fun get_encoding_from_prefix(prefix: u8): Option<String> {
        if (prefix == BASE58BTC_PREFIX) {
            return some(base58btc_name())
        };
        
        if (prefix == BASE32_PREFIX) {
            return some(base32_name())
        };
        
        if (prefix == BASE64PAD_PREFIX) {
            return some(base64pad_name())
        };
        
        if (prefix == BASE16_PREFIX) {
            return some(base16_name())
        };
        
        none()
    }

    /// Extracts the multibase prefix from an encoded string
    /// 
    /// @param encoded_str - The multibase encoded string
    /// @return - Option containing the prefix byte, or none if string is empty
    public fun extract_prefix(encoded_str: &String): Option<u8> {
        let str_bytes = string::bytes(encoded_str);
        if (vector::is_empty(str_bytes)) {
            none()
        } else {
            some(*vector::borrow(str_bytes, 0))
        }
    }

    #[test]
    fun test_encode_decode_base58btc() {
        // Test vector from base58 tests
        let original = x"0062e907b15cbf27d5425399ebf6f0fb50ebb88f18";
        let encoded = encode_base58btc(&original);
        
        // Verify prefix
        let prefix_opt = extract_prefix(&encoded);
        assert!(option::is_some(&prefix_opt), ETestAssertionFailed + 1);
        assert!(option::extract(&mut prefix_opt) == BASE58BTC_PREFIX, ETestAssertionFailed + 2);
        
        // Verify round-trip
        let decoded_opt = decode(&encoded);
        assert!(option::is_some(&decoded_opt), ETestAssertionFailed + 3);
        assert!(option::extract(&mut decoded_opt) == original, ETestAssertionFailed + 4);
    }

    #[test]
    fun test_encode_decode_base64pad() {
        // Test vector
        let original = b"Hello, World!"; // Standard Base64 (with padding) is "SGVsbG8sIFdvcmxkIQ=="
        let encoded = encode_base64pad(&original);
        
        // Verify prefix
        let prefix_opt = extract_prefix(&encoded);
        assert!(option::is_some(&prefix_opt), ETestAssertionFailed + 5);
        assert!(option::extract(&mut prefix_opt) == BASE64PAD_PREFIX, ETestAssertionFailed + 6);
        
        // Verify encoded content (without prefix)
        let expected_base64pad = b"SGVsbG8sIFdvcmxkIQ==";
        let encoded_bytes = string::bytes(&encoded);
        let encoded_without_prefix = vector::empty<u8>();
        let i = 1;
        while (i < vector::length(encoded_bytes)) {
            vector::push_back(&mut encoded_without_prefix, *vector::borrow(encoded_bytes, i));
            i = i + 1;
        };
        assert!(encoded_without_prefix == expected_base64pad, ETestAssertionFailed + 7);
        
        // Verify round-trip
        let decoded_opt = decode(&encoded);
        assert!(option::is_some(&decoded_opt), ETestAssertionFailed + 8);
        assert!(option::extract(&mut decoded_opt) == original, ETestAssertionFailed + 9);
    }

    #[test]
    fun test_encode_decode_base16() {
        // Test vector
        let original = b"Test";
        let encoded = encode_base16(&original);
        
        // Verify prefix
        let prefix_opt = extract_prefix(&encoded);
        assert!(option::is_some(&prefix_opt), ETestAssertionFailed + 10);
        assert!(option::extract(&mut prefix_opt) == BASE16_PREFIX, ETestAssertionFailed + 11);
        
        // Verify encoded content (without prefix)
        let expected_hex = b"54657374"; // "Test" in hex
        let encoded_bytes = string::bytes(&encoded);
        let encoded_without_prefix = vector::empty<u8>();
        let i = 1;
        while (i < vector::length(encoded_bytes)) {
            vector::push_back(&mut encoded_without_prefix, *vector::borrow(encoded_bytes, i));
            i = i + 1;
        };
        assert!(encoded_without_prefix == expected_hex, ETestAssertionFailed + 12);
        
        // Verify round-trip
        let decoded_opt = decode(&encoded);
        assert!(option::is_some(&decoded_opt), ETestAssertionFailed + 13);
        assert!(option::extract(&mut decoded_opt) == original, ETestAssertionFailed + 14);
    }

    #[test]
    fun test_encode_general_api() {
        let original = b"Test data";
        
        // Test base58btc encoding
        let base58btc_opt = encode(&original, &base58btc_name());
        assert!(option::is_some(&base58btc_opt), ETestAssertionFailed + 15);
        let base58btc_str = option::extract(&mut base58btc_opt);
        assert!(extract_prefix(&base58btc_str) == some(BASE58BTC_PREFIX), ETestAssertionFailed + 16);
        
        // Test base64pad encoding
        let base64pad_opt = encode(&original, &base64pad_name());
        assert!(option::is_some(&base64pad_opt), ETestAssertionFailed + 17);
        let base64pad_str = option::extract(&mut base64pad_opt);
        assert!(extract_prefix(&base64pad_str) == some(BASE64PAD_PREFIX), ETestAssertionFailed + 18);

        // Test base16/hex encoding
        let base16_opt = encode(&original, &base16_name());
        assert!(option::is_some(&base16_opt), ETestAssertionFailed + 19);
        let base16_str = option::extract(&mut base16_opt);
        assert!(extract_prefix(&base16_str) == some(BASE16_PREFIX), ETestAssertionFailed + 20);
        
        // Test hex encoding (alias for base16)
        let hex_opt = encode(&original, &hex_name());
        assert!(option::is_some(&hex_opt), ETestAssertionFailed + 21);
        let hex_str = option::extract(&mut hex_opt);
        assert!(extract_prefix(&hex_str) == some(BASE16_PREFIX), ETestAssertionFailed + 22);
        
        // Test unsupported encoding
        let unknown_opt = encode(&original, &string::utf8(b"unknown"));
        assert!(option::is_none(&unknown_opt), ETestAssertionFailed + 23);
    }

    #[test]
    fun test_encoding_name_constants() {
        assert!(base58btc_name() == string::utf8(b"base58btc"), ETestAssertionFailed + 24);
        assert!(base32_name() == string::utf8(b"base32"), ETestAssertionFailed + 25);
        assert!(base64pad_name() == string::utf8(b"base64pad"), ETestAssertionFailed + 26);
        assert!(base16_name() == string::utf8(b"base16"), ETestAssertionFailed + 27);
        assert!(hex_name() == string::utf8(b"hex"), ETestAssertionFailed + 28);
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
        
        assert!(option::is_some(&decoded_opt), ETestAssertionFailed + 29);
        assert!(option::extract(&mut decoded_opt) == pubkey, ETestAssertionFailed + 30);
    }

    #[test]
    fun test_get_prefix_for_encoding() {
        let base58_opt = get_prefix_for_encoding(&base58btc_name());
        assert!(option::is_some(&base58_opt), ETestAssertionFailed + 31);
        assert!(option::extract(&mut base58_opt) == BASE58BTC_PREFIX, ETestAssertionFailed + 32);
        
        let base64pad_opt = get_prefix_for_encoding(&base64pad_name());
        assert!(option::is_some(&base64pad_opt), ETestAssertionFailed + 33);
        assert!(option::extract(&mut base64pad_opt) == BASE64PAD_PREFIX, ETestAssertionFailed + 34);
        
        let base16_opt = get_prefix_for_encoding(&base16_name());
        assert!(option::is_some(&base16_opt), ETestAssertionFailed + 35);
        assert!(option::extract(&mut base16_opt) == BASE16_PREFIX, ETestAssertionFailed + 36);
        
        let hex_opt = get_prefix_for_encoding(&hex_name());
        assert!(option::is_some(&hex_opt), ETestAssertionFailed + 37);
        assert!(option::extract(&mut hex_opt) == BASE16_PREFIX, ETestAssertionFailed + 38);
        
        let unknown_opt = get_prefix_for_encoding(&string::utf8(b"unknown"));
        assert!(option::is_none(&unknown_opt), ETestAssertionFailed + 39);
    }

    #[test]
    fun test_get_encoding_from_prefix() {
        let base58_opt = get_encoding_from_prefix(BASE58BTC_PREFIX);
        assert!(option::is_some(&base58_opt), ETestAssertionFailed + 40);
        assert!(option::extract(&mut base58_opt) == base58btc_name(), ETestAssertionFailed + 41);
        
        let base64pad_opt = get_encoding_from_prefix(BASE64PAD_PREFIX);
        assert!(option::is_some(&base64pad_opt), ETestAssertionFailed + 42);
        assert!(option::extract(&mut base64pad_opt) == base64pad_name(), ETestAssertionFailed + 43);
        
        let base16_opt = get_encoding_from_prefix(BASE16_PREFIX);
        assert!(option::is_some(&base16_opt), ETestAssertionFailed + 44);
        assert!(option::extract(&mut base16_opt) == base16_name(), ETestAssertionFailed + 45);
        
        let unknown_opt = get_encoding_from_prefix(99); // some other prefix
        assert!(option::is_none(&unknown_opt), ETestAssertionFailed + 46);
    }

    #[test]
    fun test_extract_prefix() {
        let encoded_z = string::utf8(b"zHello");
        let prefix_opt_z = extract_prefix(&encoded_z);
        assert!(option::is_some(&prefix_opt_z), ETestAssertionFailed + 47);
        assert!(option::extract(&mut prefix_opt_z) == BASE58BTC_PREFIX, ETestAssertionFailed + 48);
        
        let encoded_M = string::utf8(b"MSGVsbG8sIFdvcmxkIQ=="); // "Hello, World!" in base64pad
        let prefix_opt_M = extract_prefix(&encoded_M);
        assert!(option::is_some(&prefix_opt_M), ETestAssertionFailed + 49);
        assert!(option::extract(&mut prefix_opt_M) == BASE64PAD_PREFIX, ETestAssertionFailed + 50);
        
        let encoded_f = string::utf8(b"f68656c6c6f"); // "hello" in hex
        let prefix_opt_f = extract_prefix(&encoded_f);
        assert!(option::is_some(&prefix_opt_f), ETestAssertionFailed + 51);
        assert!(option::extract(&mut prefix_opt_f) == BASE16_PREFIX, ETestAssertionFailed + 52);
        
        let empty = string::utf8(b"");
        let empty_opt = extract_prefix(&empty);
        assert!(option::is_none(&empty_opt), ETestAssertionFailed + 53);
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
    fun test_encode_decode_ed25519_did_key_identifier() {
        // Create a test Ed25519 public key (32 bytes)
        let pubkey = vector::empty<u8>();
        let i = 0;
        while (i < ED25519_PUBLIC_KEY_LENGTH) {
            vector::push_back(&mut pubkey, (i as u8));
            i = i + 1;
        };
        
        let encoded_identifier = encode_ed25519_did_key_identifier(&pubkey);
        
        // Verify it starts with 'z' (base58btc prefix)
        let prefix_opt = extract_prefix(&encoded_identifier);
        assert!(option::is_some(&prefix_opt), ETestAssertionFailed + 60);
        assert!(option::extract(&mut prefix_opt) == BASE58BTC_PREFIX, ETestAssertionFailed + 61);
        
        // Verify round-trip decoding
        let decoded_opt = decode_did_key_identifier(&encoded_identifier);
        assert!(option::is_some(&decoded_opt), ETestAssertionFailed + 62);
        assert!(option::extract(&mut decoded_opt) == pubkey, ETestAssertionFailed + 63);
        
        // Verify key type detection
        let key_type_opt = get_key_type_from_did_key_identifier(&encoded_identifier);
        assert!(option::is_some(&key_type_opt), ETestAssertionFailed + 64);
        assert!(option::extract(&mut key_type_opt) == string::utf8(b"Ed25519"), ETestAssertionFailed + 65);
    }

    #[test]
    fun test_encode_decode_secp256k1_did_key_identifier() {
        // Create a test Secp256k1 public key (33 bytes)
        let pubkey = vector::empty<u8>();
        let i = 0;
        while (i < SECP256K1_COMPRESSED_PUBLIC_KEY_LENGTH) {
            vector::push_back(&mut pubkey, (i as u8));
            i = i + 1;
        };
        
        let encoded_identifier = encode_secp256k1_did_key_identifier(&pubkey);
        
        // Verify it starts with 'z' (base58btc prefix)
        let prefix_opt = extract_prefix(&encoded_identifier);
        assert!(option::is_some(&prefix_opt), ETestAssertionFailed + 66);
        assert!(option::extract(&mut prefix_opt) == BASE58BTC_PREFIX, ETestAssertionFailed + 67);
        
        // Verify round-trip decoding
        let decoded_opt = decode_did_key_identifier(&encoded_identifier);
        assert!(option::is_some(&decoded_opt), ETestAssertionFailed + 68);
        assert!(option::extract(&mut decoded_opt) == pubkey, ETestAssertionFailed + 69);
        
        // Verify key type detection
        let key_type_opt = get_key_type_from_did_key_identifier(&encoded_identifier);
        assert!(option::is_some(&key_type_opt), ETestAssertionFailed + 70);
        assert!(option::extract(&mut key_type_opt) == string::utf8(b"Secp256k1"), ETestAssertionFailed + 71);
    }

    #[test]
    fun test_generate_ed25519_did_key_string() {
        // Create a test Ed25519 public key (32 bytes)
        let pubkey = vector::empty<u8>();
        let i = 0;
        while (i < ED25519_PUBLIC_KEY_LENGTH) {
            vector::push_back(&mut pubkey, (i as u8));
            i = i + 1;
        };
        
        let did_key_string = generate_ed25519_did_key_string(&pubkey);
        
        // Verify it starts with "did:key:"
        let did_key_prefix = string::utf8(b"did:key:");
        let did_key_bytes = string::bytes(&did_key_string);
        let prefix_bytes = string::bytes(&did_key_prefix);
        
        assert!(vector::length(did_key_bytes) > vector::length(prefix_bytes), ETestAssertionFailed + 72);
        
        let i = 0;
        while (i < vector::length(prefix_bytes)) {
            assert!(*vector::borrow(did_key_bytes, i) == *vector::borrow(prefix_bytes, i), ETestAssertionFailed + 73);
            i = i + 1;
        };
        
        // Extract identifier part and verify it can be decoded
        let identifier = string::sub_string(&did_key_string, 8, string::length(&did_key_string)); // Skip "did:key:"
        let decoded_opt = decode_did_key_identifier(&identifier);
        assert!(option::is_some(&decoded_opt), ETestAssertionFailed + 74);
        assert!(option::extract(&mut decoded_opt) == pubkey, ETestAssertionFailed + 75);
    }

    #[test]
    fun test_generate_secp256k1_did_key_string() {
        // Create a test Secp256k1 public key (33 bytes)
        let pubkey = vector::empty<u8>();
        let i = 0;
        while (i < SECP256K1_COMPRESSED_PUBLIC_KEY_LENGTH) {
            vector::push_back(&mut pubkey, (i as u8));
            i = i + 1;
        };
        
        let did_key_string = generate_secp256k1_did_key_string(&pubkey);
        
        // Verify it starts with "did:key:"
        let did_key_prefix = string::utf8(b"did:key:");
        let did_key_bytes = string::bytes(&did_key_string);
        let prefix_bytes = string::bytes(&did_key_prefix);
        
        assert!(vector::length(did_key_bytes) > vector::length(prefix_bytes), ETestAssertionFailed + 76);
        
        let i = 0;
        while (i < vector::length(prefix_bytes)) {
            assert!(*vector::borrow(did_key_bytes, i) == *vector::borrow(prefix_bytes, i), ETestAssertionFailed + 77);
            i = i + 1;
        };
        
        // Extract identifier part and verify it can be decoded
        let identifier = string::sub_string(&did_key_string, 8, string::length(&did_key_string)); // Skip "did:key:"
        let decoded_opt = decode_did_key_identifier(&identifier);
        assert!(option::is_some(&decoded_opt), ETestAssertionFailed + 78);
        assert!(option::extract(&mut decoded_opt) == pubkey, ETestAssertionFailed + 79);
    }

    #[test]
    fun test_decode_invalid_did_key_identifier() {
        // Test with invalid multibase prefix
        let invalid_identifier = string::utf8(b"a123456789"); // 'a' is not base58btc
        let decoded_opt = decode_did_key_identifier(&invalid_identifier);
        assert!(option::is_none(&decoded_opt), ETestAssertionFailed + 80);
        
        // Test with valid multibase but invalid multicodec
        let invalid_multicodec = string::utf8(b"z123"); // Valid base58btc but invalid multicodec
        let decoded_opt2 = decode_did_key_identifier(&invalid_multicodec);
        assert!(option::is_none(&decoded_opt2), ETestAssertionFailed + 81);
        
        // Test with empty string
        let empty_identifier = string::utf8(b"");
        let decoded_opt3 = decode_did_key_identifier(&empty_identifier);
        assert!(option::is_none(&decoded_opt3), ETestAssertionFailed + 82);
    }

    #[test]
    fun test_complete_did_key_workflow() {
        // Test the complete workflow: raw key -> did:key string -> extract raw key
        
        // Create a test Ed25519 public key (32 bytes)
        let original_pubkey = vector::empty<u8>();
        let i = 0;
        while (i < ED25519_PUBLIC_KEY_LENGTH) {
            vector::push_back(&mut original_pubkey, ((i + 42) as u8)); // Use different pattern
            i = i + 1;
        };
        
        // Step 1: Generate did:key string from raw public key
        let did_key_string = generate_ed25519_did_key_string(&original_pubkey);
        
        // Step 2: Verify the format
        assert!(string::length(&did_key_string) > 8, ETestAssertionFailed + 83);
        let prefix_part = string::sub_string(&did_key_string, 0, 8);
        assert!(prefix_part == string::utf8(b"did:key:"), ETestAssertionFailed + 84);
        
        // Step 3: Extract identifier part
        let identifier_part = string::sub_string(&did_key_string, 8, string::length(&did_key_string));
        
        // Step 4: Decode identifier to get raw public key
        let decoded_pubkey_opt = decode_did_key_identifier(&identifier_part);
        assert!(option::is_some(&decoded_pubkey_opt), ETestAssertionFailed + 85);
        let decoded_pubkey = option::destroy_some(decoded_pubkey_opt);
        
        // Step 5: Verify round-trip integrity
        assert!(decoded_pubkey == original_pubkey, ETestAssertionFailed + 86);
        
        // Step 6: Verify key type detection
        let key_type_opt = get_key_type_from_did_key_identifier(&identifier_part);
        assert!(option::is_some(&key_type_opt), ETestAssertionFailed + 87);
        assert!(option::extract(&mut key_type_opt) == string::utf8(b"Ed25519"), ETestAssertionFailed + 88);
    }
}