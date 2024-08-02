// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

module rooch_nursery::result_test {

    use rooch_nursery::result::{Self, Result, ok, err, assert_ok};

    const ErrorForTest: u64 = 1;

    #[test]
    public fun test_result() {
        let result: Result<u64> = ok(1);
        let value = assert_ok(result, ErrorForTest);
        assert!(value == 1, 1);
    }

    #[test]
    #[expected_failure(abort_code = ErrorForTest, location = Self)]
    public fun test_result_err() {
        let result: Result<u64> = err(b"error");
        let _value = assert_ok(result, ErrorForTest);
    }

    fun test_return_result() : Result<u64> {
        ok(1)
    }

    #[test]
    public fun test_and_then() {
        let result: Result<u64> = test_return_result();
        let result2 = result::and_then(result, |value| {
            ok(value + 1)
        });
        let value = assert_ok(result2, ErrorForTest);
        assert!(value == 2, 2);
    }

}