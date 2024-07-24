// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

// Copyright (c) Mysten Labs, Inc.
// SPDX-License-Identifier: Apache-2.0

#[test_only]
module std::u256_tests {
    use std::u256;

    const MAX_ERROR: u64 = 0;
    const MIN_ERROR: u64 = 1;
    const DIFF_ERROR: u64 = 2;
    const DIVIDE_AND_ROUND_UP_ERROR: u64 = 3;
    const MULTIPLE_AND_DIVIDE_ERROR: u64 = 4;
    const POW_ERROR: u64 = 5;
    const SQRT_ERROR: u64 = 6;

    #[test]
    fun test_max() {
        assert!(u256::max(1, 57896044618658097711785492504343953926634992332820282019728792003956564819967) == 57896044618658097711785492504343953926634992332820282019728792003956564819967, MAX_ERROR);
        assert!(u256::max(57896044618658097711785492504343953926634992332820282019728792003956564819967, 1) == 57896044618658097711785492504343953926634992332820282019728792003956564819967, MAX_ERROR);
        assert!(u256::max(0, 0) == 0, MAX_ERROR);
    }

    #[test]
    fun test_min() {
        assert!(u256::min(1, 57896044618658097711785492504343953926634992332820282019728792003956564819967) == 1, MIN_ERROR);
        assert!(u256::min(57896044618658097711785492504343953926634992332820282019728792003956564819967, 1) == 1, MIN_ERROR);
        assert!(u256::min(0, 0) == 0, MIN_ERROR);
    }

    #[test]
    fun test_diff() {
        assert!(u256::diff(1, 57896044618658097711785492504343953926634992332820282019728792003956564819967) == 57896044618658097711785492504343953926634992332820282019728792003956564819966, DIFF_ERROR);
        assert!(u256::diff(57896044618658097711785492504343953926634992332820282019728792003956564819967, 1) == 57896044618658097711785492504343953926634992332820282019728792003956564819966, DIFF_ERROR);
        assert!(u256::diff(0, 0) == 0, DIFF_ERROR);
    }

    #[test]
    fun test_divide_and_round_up() {
        assert!(u256::divide_and_round_up(6, 2) == 3, DIVIDE_AND_ROUND_UP_ERROR);
        assert!(u256::divide_and_round_up(7, 2) == 4, DIVIDE_AND_ROUND_UP_ERROR);
    }

    #[test, expected_failure(arithmetic_error, location = std::u256)]
    fun test_divide_and_round_up_error() {
        u256::divide_and_round_up(1, 0);
    }

    #[test]
    fun test_multiple_and_divide() {
        assert!(u256::multiple_and_divide(6, 2, 2) == 6, MULTIPLE_AND_DIVIDE_ERROR); // y == z
        assert!(u256::multiple_and_divide(7, 2, 7) == 2, MULTIPLE_AND_DIVIDE_ERROR); // x == z
        assert!(u256::multiple_and_divide(10, 6, 5) == 12, MULTIPLE_AND_DIVIDE_ERROR);
    }

    #[test, expected_failure(arithmetic_error, location = std::u256)]
    fun test_multiple_and_divide_error() {
        u256::multiple_and_divide(1, 1, 0);
    }

    #[test]
    fun test_pow() {
        assert!(u256::pow(1, 0) == 1, POW_ERROR);
        assert!(u256::pow(3, 1) == 3, POW_ERROR);
        assert!(u256::pow(2, 6) == 64, POW_ERROR);
    }

    #[test]
    #[expected_failure]
    fun test_pow_overflow() {
        u256::pow(10, 100);
    }
}
