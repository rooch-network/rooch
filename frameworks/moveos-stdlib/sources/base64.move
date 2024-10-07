// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0


/// Module which defines base64 functions.
module moveos_std::base64 {

    // Decode failed error
    const E_DECODE_FAILED: u64 = 1;
   
    /// @param input: bytes to be encoded
    /// Encode the input bytes with Base64 algorithm and returns an encoded base64 string
    native public fun encode(input: &vector<u8>): vector<u8>;

    /// @param encoded_input: encoded base64 string
    /// Decode the base64 string and returns the original bytes
    native public fun decode(encoded_input: &vector<u8>): vector<u8>;

    #[test]
    fun test_encode() {
        let input = b"Hello, World!";
        let encoded = encode(&input);
        let expected = b"SGVsbG8sIFdvcmxkIQ==";

        assert!(encoded == expected, 1000);
    }

    #[test]
    fun test_decode() {
        let encoded_input = b"SGVsbG8sIFdvcmxkIQ==";
        let decoded = decode(&encoded_input);
        let expected = b"Hello, World!";

        assert!(decoded == expected, E_DECODE_FAILED);
    }
}