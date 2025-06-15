// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

#[test_only]
module rooch_framework::rs256_test {
    use std::vector;
    use rooch_framework::rs256::{ verify };
    use moveos_std::base64::{ decode };

    // Base64 URL identifiers
    const MINUS: u8 = 45; // minus in ASCII. 62nd of Value Encoding.
    const UNDERLINE: u8 = 95; // underline in ASCII. 63rd of Value Encoding.

    // Base64 identifiers
    const PLUS: u8 = 43; // plus in ASCII. 62nd of Value Encoding.
    const SLASH: u8 = 47; // slash in ASCII. 63rd or Value Encoding.

    // Error codes
    const ErrorVerificationFailure: u64 = 0;

    // From https://www.rfc-editor.org/rfc/rfc4648#page-7
    fun decode_base64url(base64url: vector<u8>): vector<u8> {
        while (vector::contains(&base64url, &MINUS)) {
            let (_, minus_index) = vector::index_of(&base64url, &MINUS);
            vector::remove(&mut base64url, minus_index);
            vector::insert(&mut base64url, minus_index, PLUS);
        };
        while (vector::contains(&base64url, &UNDERLINE)) {
            let (_, underline_index) = vector::index_of(&base64url, &UNDERLINE);
            vector::remove(&mut base64url, underline_index);
            vector::insert(&mut base64url, underline_index, SLASH);
        };

        decode(&base64url)
    }

    #[test]
    // Test cases taken from https://www.rfc-editor.org/rfc/rfc7515#page-38 and later
    fun test_verify_success() {
        let msg = b"eyJhbGciOiJSUzI1NiJ9.eyJpc3MiOiJqb2UiLA0KICJleHAiOjEzMDA4MTkzODAsDQogImh0dHA6Ly9leGFtcGxlLmNvbS9pc19yb290Ijp0cnVlfQ";
        let n_base64url = b"ofgWCuLjybRlzo0tZWJjNiuSfb4p4fAkd_wWJcyQoTbji9k0l8W26mPddxHmfHQp-Vaw-4qPCJrcS2mJPMEzP1Pt0Bm4d4QlL-yRT-SFd2lZS-pCgNMsD1W_YpRPEwOWvG6b32690r2jZ47soMZo9wGzjb_7OMg0LOL-bSf63kpaSHSXndS5z5rexMdbBYUsLA9e-KXBdQOS-UTo7WTBEMa2R2CapHg665xsmtdVMTBQY4uDZlxvb3qCo5ZwKh9kG4LT6_I5IhlJH7aGhyxXFvUK-DWNmoudF8NAco9_h9iaGNj8q2ethFkMLs91kzk2PAcDTW9gb54h4FRWyuXpoQ";
        let n_bytes = decode_base64url(n_base64url);
        std::debug::print(&n_bytes);
        let e_base64url = b"AQAB";
        let e_bytes = decode_base64url(e_base64url);
        std::debug::print(&e_bytes);
        let signature_base64url = b"cC4hiUPoj9Eetdgtv3hF80EGrhuB__dzERat0XF9g2VtQgr9PJbu3XOiZj5RZmh7AAuHIm4Bh-0Qc_lF5YKt_O8W2Fp5jujGbds9uJdbF9CUAr7t1dnZcAcQjbKBYNX4BAynRFdiuB--f_nZLgrnbyTyWzO75vRK5h6xBArLIARNPvkSjtQBMHlb1L07Qe7K0GarZRmB_eSN9383LcOLn6_dO--xi12jzDwusC-eOkHWEsqtFZESc6BfI7noOPqvhJ1phCnvWh6IeYI2w9QOYEUipUTI8np6LbgGY9Fs98rqVt5AXLIhWkWywlVmtVrBp0igcN_IoypGlUPQGe77Rw";
        let signature_bytes = decode_base64url(signature_base64url);
        std::debug::print(&signature_bytes);
        let result = verify(&signature_bytes, &n_bytes, &e_bytes, &msg);
        assert!(result, ErrorVerificationFailure);
    }

    // #[test]
    // #[expected_failure(location=Self, abort_code = ErrorInvalidSignature)]
    // fun test_verify_fails_invalid_sig() {
    //     // Test case with invalid signature length
    //     let msg = b"hello world";
    //     let pubkey = x"0258a618066814098f8ddb3cbde73838b59028d843958031e50be0a5f4b0a9796d";
    //     let sig = x"0000"; // Invalid length
    //     verify(&sig, &pubkey, &msg);
    // }

    // #[test]
    // #[expected_failure(location=Self, abort_code = ErrorInvalidPubKey)]
    // fun test_verify_fails_invalid_pubkey() {
    //     // Test case with invalid public key length
    //     let msg = b"hello world";
    //     let pubkey = x"0000"; // Invalid length
    //     let sig = x"74133905657c1992d8d6bd72ffa7ccf8d2adf3e4a3ca25f8dc8eec175752cb5a40459f71b549a25cba3cddf4157e946bbff7b18fc82774e9c4c54e362b97ccb5";
    //     verify(&sig, &pubkey, &msg);
    // }

    // #[test]
    // fun test_verify_with_different_message() {
    //     // Test case with different message
    //     let msg = b"different message";
    //     let pubkey = x"0258a618066814098f8ddb3cbde73838b59028d843958031e50be0a5f4b0a9796d";
    //     let sig = x"74133905657c1992d8d6bd72ffa7ccf8d2adf3e4a3ca25f8dc8eec175752cb5a40459f71b549a25cba3cddf4157e946bbff7b18fc82774e9c4c54e362b97ccb5";
    //     let result = verify(&sig, &pubkey, &msg);
    //     assert!(!result, 0);
    // }

    // #[test]
    // fun test_verify_with_empty_message() {
    //     // Test case with empty message
    //     let msg = b"";
    //     let pubkey = x"0258a618066814098f8ddb3cbde73838b59028d843958031e50be0a5f4b0a9796d";
    //     let sig = x"74133905657c1992d8d6bd72ffa7ccf8d2adf3e4a3ca25f8dc8eec175752cb5a40459f71b549a25cba3cddf4157e946bbff7b18fc82774e9c4c54e362b97ccb5";
    //     let result = verify(&sig, &pubkey, &msg);
    //     assert!(!result, 0);
    // }

    // #[test]
    // fun test_verify_with_long_message() {
    //     // Test case with a longer message
    //     let msg = x"000102030405060708090a0b0c0d0e0f101112131415161718191a1b1c1d1e1f202122232425262728292a2b2c2d2e2f303132333435363738393a3b3c3d3e3f";
    //     let pubkey = x"0258a618066814098f8ddb3cbde73838b59028d843958031e50be0a5f4b0a9796d";
    //     let sig = x"74133905657c1992d8d6bd72ffa7ccf8d2adf3e4a3ca25f8dc8eec175752cb5a40459f71b549a25cba3cddf4157e946bbff7b18fc82774e9c4c54e362b97ccb5";
    //     let result = verify(&sig, &pubkey, &msg);
    //     assert!(!result, 0);
    // }
}
