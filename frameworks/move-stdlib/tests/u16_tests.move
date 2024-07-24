// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

// Copyright (c) Mysten Labs, Inc.
// SPDX-License-Identifier: Apache-2.0

#[test_only]
module std::u16_tests {
    use std::u16;

    const MAX_ERROR: u64 = 0;
    const MIN_ERROR: u64 = 1;
    const DIFF_ERROR: u64 = 2;
    const DIVIDE_AND_ROUND_UP_ERROR: u64 = 3;
    const MULTIPLE_AND_DIVIDE_ERROR: u64 = 4;
    const POW_ERROR: u64 = 5;
    const SQRT_ERROR: u64 = 6;

    #[test]
    fun test_max() {
        assert!(u16::max(1, 65535) == 65535, MAX_ERROR);
        assert!(u16::max(65535, 1) == 65535, MAX_ERROR);
        assert!(u16::max(0, 0) == 0, MAX_ERROR);
    }

    #[test]
    fun test_min() {
        assert!(u16::min(1, 65535) == 1, MIN_ERROR);
        assert!(u16::min(65535, 1) == 1, MIN_ERROR);
        assert!(u16::min(0, 0) == 0, MIN_ERROR);
    }

    #[test]
    fun test_diff() {
        assert!(u16::diff(1, 65535) == 65534, DIFF_ERROR);
        assert!(u16::diff(65535, 1) == 65534, DIFF_ERROR);
        assert!(u16::diff(0, 0) == 0, DIFF_ERROR);
    }

    #[test]
    fun test_divide_and_round_up() {
        assert!(u16::divide_and_round_up(6, 2) == 3, DIVIDE_AND_ROUND_UP_ERROR);
        assert!(u16::divide_and_round_up(7, 2) == 4, DIVIDE_AND_ROUND_UP_ERROR);
    }

    #[test, expected_failure(arithmetic_error, location = std::u16)]
    fun test_divide_and_round_up_error() {
        u16::divide_and_round_up(1, 0);
    }

    #[test]
    fun test_multiple_and_divide() {
        assert!(u16::multiple_and_divide(6, 2, 2) == 6, MULTIPLE_AND_DIVIDE_ERROR); // y == z
        assert!(u16::multiple_and_divide(7, 2, 7) == 2, MULTIPLE_AND_DIVIDE_ERROR); // x == z
        assert!(u16::multiple_and_divide(10, 2, 5) == 4, MULTIPLE_AND_DIVIDE_ERROR);
    }

    #[test, expected_failure(arithmetic_error, location = std::u16)]
    fun test_multiple_and_divide_error() {
        u16::multiple_and_divide(1, 1, 0);
    }

    #[test]
    fun test_pow() {
        assert!(u16::pow(1, 0) == 1, POW_ERROR);
        assert!(u16::pow(3, 1) == 3, POW_ERROR);
        assert!(u16::pow(2, 10) == 1024, POW_ERROR);
    }

    #[test]
    #[expected_failure]
    fun test_pow_overflow() {
        u16::pow(10, 100);
    }

    // TODO: fix arithmetic error given by i * i
    // #[test]
    // fun test_perfect_sqrt() {
    //     let i = 0;
    //     while (i < 100) {
    //         assert!(u16::sqrt(i * i) == i, SQRT_ERROR);
    //         i = i + 1;
    //     };
    //     let i = 0xFFu16;
    //     while (i < 0xFFu16 + 1) {
    //         assert!(u16::sqrt(i * i) == i, SQRT_ERROR);
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
            let root = u16::sqrt(i);

            assert!(i == root * root || root == prev, SQRT_ERROR);

            prev = root;
            i = i + 1;
        };
    }

    #[test]
    fun test_sqrt_big_numbers() {
        let u16_max = 65535;
        assert!(255 == u16::sqrt(u16_max), SQRT_ERROR);
    }
}
