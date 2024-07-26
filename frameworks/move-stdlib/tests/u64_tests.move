// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

// Copyright (c) Mysten Labs, Inc.
// SPDX-License-Identifier: Apache-2.0

#[test_only]
module std::u64_tests {
    use std::u64;

    const MAX_ERROR: u64 = 0;
    const MIN_ERROR: u64 = 1;
    const DIFF_ERROR: u64 = 2;
    const DIVIDE_AND_ROUND_UP_ERROR: u64 = 3;
    const MULTIPLE_AND_DIVIDE_ERROR: u64 = 4;
    const POW_ERROR: u64 = 5;
    const SQRT_ERROR: u64 = 6;

    #[test]
    fun test_max() {
        assert!(u64::max(1, 18446744073709551615) == 18446744073709551615, MAX_ERROR);
        assert!(u64::max(18446744073709551615, 1) == 18446744073709551615, MAX_ERROR);
        assert!(u64::max(0, 0) == 0, MAX_ERROR);
    }

    #[test]
    fun test_min() {
        assert!(u64::min(1, 18446744073709551615) == 1, MIN_ERROR);
        assert!(u64::min(18446744073709551615, 1) == 1, MIN_ERROR);
        assert!(u64::min(0, 0) == 0, MIN_ERROR);
    }

    #[test]
    fun test_diff() {
        assert!(u64::diff(1, 18446744073709551615) == 18446744073709551614, DIFF_ERROR);
        assert!(u64::diff(18446744073709551615, 1) == 18446744073709551614, DIFF_ERROR);
        assert!(u64::diff(0, 0) == 0, DIFF_ERROR);
    }

    #[test]
    fun test_divide_and_round_up() {
        assert!(u64::divide_and_round_up(6, 2) == 3, DIVIDE_AND_ROUND_UP_ERROR);
        assert!(u64::divide_and_round_up(7, 2) == 4, DIVIDE_AND_ROUND_UP_ERROR);
    }

    #[test, expected_failure(arithmetic_error, location = std::u64)]
    fun test_divide_and_round_up_error() {
        u64::divide_and_round_up(1, 0);
    }

    #[test]
    fun test_multiple_and_divide() {
        assert!(u64::multiple_and_divide(6, 2, 2) == 6, MULTIPLE_AND_DIVIDE_ERROR); // y == z
        assert!(u64::multiple_and_divide(7, 2, 7) == 2, MULTIPLE_AND_DIVIDE_ERROR); // x == z
        assert!(u64::multiple_and_divide(10, 1, 5) == 2, MULTIPLE_AND_DIVIDE_ERROR);
    }

    #[test, expected_failure(arithmetic_error, location = std::u64)]
    fun test_multiple_and_divide_error() {
        u64::multiple_and_divide(1, 1, 0);
    }

    #[test]
    fun test_pow() {
        assert!(u64::pow(1, 0) == 1, POW_ERROR);
        assert!(u64::pow(3, 1) == 3, POW_ERROR);
        assert!(u64::pow(2, 9) == 512, POW_ERROR);
    }

    #[test]
    #[expected_failure]
    fun test_pow_overflow() {
        u64::pow(10, 100);
    }

    // TODO: fix arithmetic error given by i * i
    // #[test]
    // fun test_perfect_sqrt() {
    //     let i = 0;
    //     while (i < 100) {
    //         assert!(u64::sqrt(i * i) == i, SQRT_ERROR);
    //         i = i + 1;
    //     };
    //     let i = 0xFFu64;
    //     while (i < 0xFFu64 + 1) {
    //         assert!(u64::sqrt(i * i) == i, SQRT_ERROR);
    //         i = i + 1;
    //     }
    // }

    #[test]
    // This function tests whether the (square root)^2 equals the
    // initial value OR whether it is equal to the nearest lower
    // number that does.
    fun test_imperfect_sqrt() {
        let i = 1;
        let prev = 1;
        while (i <= 100) {
            let root = u64::sqrt(i);

            assert!(i == root * root || root == prev, SQRT_ERROR);

            prev = root;
            i = i + 1;
        };
    }

    #[test]
    fun test_sqrt_big_numbers() {
        let u64_max = 18446744073709551615;
        assert!(4294967295 == u64::sqrt(u64_max), SQRT_ERROR);
    }
}
