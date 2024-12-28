// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

#[test_only]
module moveos_std::base64_tests {

    use std::vector;
    use moveos_std::base64::{ decode };

    const E_DECODE_FAILED: u64 = 1;

    #[test]
    fun test_decode() {
        let encoded_input = b"/////w==";
        let decoded = decode(&encoded_input);
        let v = vector::empty<u8>();

        vector::push_back(&mut v, 255);
        vector::push_back(&mut v, 255);
        vector::push_back(&mut v, 255);
        vector::push_back(&mut v, 255);
        decode(&encoded_input);

        assert!(v == decoded, E_DECODE_FAILED);
    }

    #[test]
    fun test_url_safe_decode() {
        let encoded_input = b"_____w";
        let decoded = decode(&encoded_input);
        let v = vector::empty<u8>();

        vector::push_back(&mut v, 255);
        vector::push_back(&mut v, 255);
        vector::push_back(&mut v, 255);
        vector::push_back(&mut v, 255);
        decode(&encoded_input);

        assert!(v == decoded, E_DECODE_FAILED);
    }


    #[test]
    fun test_decode1() {
        let encoded_input = b"aHR0cHM6Ly9naXRodWIuY29tL3Jvb2NoLW5ldHdvcmsvcm9vY2gvYmxvYi8zZTdjNjNjZjMyMmMzZGZkMmU2ZTE0MTkzOTM4OTdlYjEzY2M0Mjk0L2ZyYW1ld29ya3MvbW92ZW9zLXN0ZGxpYi9zcmMvbmF0aXZlcy9tb3Zlb3Nfc3RkbGliL2Jhc2U2NC5ycw==";
        let decoded = decode(&encoded_input);
        let expected = b"https://github.com/rooch-network/rooch/blob/3e7c63cf322c3dfd2e6e1419393897eb13cc4294/frameworks/moveos-stdlib/src/natives/moveos_stdlib/base64.rs";

        assert!(decoded == expected, E_DECODE_FAILED);
    }

    #[test]
    fun test_url_safe_decode1() {
        let encoded_input = b"aHR0cHM6Ly9naXRodWIuY29tL3Jvb2NoLW5ldHdvcmsvcm9vY2gvYmxvYi8zZTdjNjNjZjMyMmMzZGZkMmU2ZTE0MTkzOTM4OTdlYjEzY2M0Mjk0L2ZyYW1ld29ya3MvbW92ZW9zLXN0ZGxpYi9zcmMvbmF0aXZlcy9tb3Zlb3Nfc3RkbGliL2Jhc2U2NC5ycw";
        let decoded = decode(&encoded_input);
        let expected = b"https://github.com/rooch-network/rooch/blob/3e7c63cf322c3dfd2e6e1419393897eb13cc4294/frameworks/moveos-stdlib/src/natives/moveos_stdlib/base64.rs";

        assert!(decoded == expected, E_DECODE_FAILED);
    }
}