// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

module moveos_std::result_tests {

    use std::string::String;
    use moveos_std::result::{Self, Result, ok, err_str, assert_ok};

    const ErrorForTest: u64 = 1;

    #[test]
    public fun test_result() {
        let result: Result<u64, String> = ok(1);
        let value = assert_ok(result, ErrorForTest);
        assert!(value == 1, 1);
    }

    #[test]
    #[expected_failure(abort_code = ErrorForTest, location = Self)]
    public fun test_result_err() {
        let result: Result<u64, String> = err_str(b"error");
        let _value = assert_ok(result, ErrorForTest);
    }

    fun test_return_result() : Result<u64, String> {
        ok(1)
    }

    #[test]
    public fun test_and_then() {
        let result = test_return_result();
        let result2 = result::and_then(result, |value| {
            ok(value + 1)
        });
        let value = assert_ok(result2, ErrorForTest);
        assert!(value == 2, 2);
    }

}