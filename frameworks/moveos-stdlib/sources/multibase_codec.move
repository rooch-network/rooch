// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

/// Defines the `multibase_codec` module, providing basic encoding and decoding operations
/// for multibase-formatted data without key type awareness.
///
/// ## Overview
///
/// Multibase is a protocol for self-identifying base encodings for binary data in text formats.
/// This module handles the core encoding/decoding operations without any key type or DID-specific logic.
///
/// The format is: `<base-encoding-code-point><base-encoded-data>`
///
/// ## Supported Encodings
///
/// This module currently supports the following encodings:
///
/// *   **Base16 (Hexadecimal)**:
///     *   Prefix: `'f'` (ASCII: 102)
///     *   Name: `"base16"` (alias: `"hex"`)
/// *   **Base58 Bitcoin (base58btc)**:
///     *   Prefix: `'z'` (ASCII: 122)
///     *   Name: `"base58btc"`
/// *   **Base64 with Padding (base64pad)**:
///     *   Prefix: `'M'` (ASCII: 77)
///     *   Name: `"base64pad"`
///
/// For more details on the Multibase standard, see: [https://github.com/multiformats/multibase](https://github.com/multiformats/multibase)

module moveos_std::multibase_codec {
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
    /// Error when the encoding process fails
    const ErrorEncodingFailed: u64 = 5;
    // General test failure code for asserts
    const ETestAssertionFailed: u64 = 100;

    /// The prefix for base58btc encoding ('z' in ASCII)
    const BASE58BTC_PREFIX: u8 = 122;
    /// The prefix for base32 encoding ('b' in ASCII)
    const BASE32_PREFIX: u8 = 98;
    /// The prefix for base64pad encoding ('M' in ASCII)
    const BASE64PAD_PREFIX: u8 = 77; // 'M' for base64pad (RFC4648 with padding)
    /// The prefix for base16 (hex) encoding ('f' in ASCII)
    const BASE16_PREFIX: u8 = 102;

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
} 