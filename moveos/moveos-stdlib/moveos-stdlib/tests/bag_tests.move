// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

// Copyright (c) Mysten Labs, Inc.
// SPDX-License-Identifier: Apache-2.0

#[test_only]
module moveos_std::bag_tests {
    use moveos_std::bag::{Self, add, contains_with_type, borrow, borrow_mut, remove};

    #[test]
    fun simple_all_functions() {
        let bag = bag::new();
        // add fields
        add(&mut bag, b"hello", 0);
        add(&mut bag, 1, 1u8);
        // check they exist
        assert!(contains_with_type<vector<u8>, u64>(&bag, b"hello"), 0);
        assert!(contains_with_type<u64, u8>(&bag, 1), 0);
        // check the values
        assert!(*borrow(&bag, b"hello") == 0, 0);
        assert!(*borrow(&bag, 1) == 1u8, 0);
        // mutate them
        *borrow_mut(&mut bag, b"hello") = *borrow(&bag, b"hello") * 2;
        *borrow_mut(&mut bag, 1) = *borrow(&bag, 1) * 2u8;
        // check the new value
        assert!(*borrow(&bag, b"hello") == 0, 0);
        assert!(*borrow(&bag, 1) == 2u8, 0);
        // remove the value and check it
        assert!(remove(&mut bag, b"hello") == 0, 0);
        assert!(remove(&mut bag, 1) == 2u8, 0);
        // verify that they are not there
        assert!(!contains_with_type<vector<u8>, u64>(&bag, b"hello"), 0);
        assert!(!contains_with_type<u64, u8>(&bag, 1), 0);
        
        bag::destroy_empty(bag);
    }

    #[test]
    #[expected_failure(abort_code = moveos_std::object::ErrorAlreadyExists)]
    fun add_duplicate() {
        let bag = bag::new();
        add(&mut bag, b"hello", 0u8);
        add(&mut bag, b"hello", 1u8);
        bag::drop_unchecked(bag)
    }

    #[test]
    #[expected_failure(abort_code = moveos_std::object::ErrorAlreadyExists)]
    fun add_duplicate_mismatched_type() {
        let bag = bag::new();
        add(&mut bag, b"hello", 0u128);
        add(&mut bag, b"hello", 1u8);
        bag::drop_unchecked(bag)
    }

    #[test]
    #[expected_failure(abort_code = moveos_std::object::ErrorNotFound)]
    fun borrow_missing() {
        let bag = bag::new();
        borrow<u64, u64>(&bag, 0);
        bag::drop_unchecked(bag)
    }

    #[test]
    #[expected_failure(abort_code = moveos_std::object::ErrorTypeMismatch)]
    fun borrow_wrong_type() {
        let bag = bag::new();
        add(&mut bag, 0, 0);
        borrow<u64, u8>(&bag, 0);
        bag::drop_unchecked(bag)
    }

    #[test]
    #[expected_failure(abort_code = moveos_std::object::ErrorNotFound)]
    fun borrow_mut_missing() {
        let bag = bag::new();
        borrow_mut<u64, u64>(&mut bag, 0);
        bag::drop_unchecked(bag)
    }

    #[test]
    #[expected_failure(abort_code = moveos_std::object::ErrorTypeMismatch)]
    fun borrow_mut_wrong_type() {        
        let bag = bag::new();
        add(&mut bag, 0, 0);
        borrow_mut<u64, u8>(&mut bag, 0);
        bag::drop_unchecked(bag)
    }

    #[test]
    #[expected_failure(abort_code = moveos_std::object::ErrorNotFound)]
    fun remove_missing() {
        let bag = bag::new();
        remove<u64, u64>(&mut bag, 0);
        bag::drop_unchecked(bag)
    }

    #[test]
    #[expected_failure(abort_code = moveos_std::object::ErrorTypeMismatch)]
    fun remove_wrong_type() {
        let bag = bag::new();
        add(&mut bag, 0, 0);
        remove<u64, u8>(&mut bag, 0);
        bag::drop_unchecked(bag)
    }

    #[test]
    #[expected_failure(abort_code = moveos_std::object::ErrorFieldsNotEmpty)]
    fun destroy_non_empty() {
        let bag = bag::new();
        add(&mut bag, 0, 0);
        bag::destroy_empty(bag);
        
    }

    #[test]
    fun sanity_check_contains() {
        let bag = bag::new();
        assert!(!contains_with_type<u64, u64>(&bag, 0), 0);
        add(&mut bag, 0, 0);
        assert!(contains_with_type<u64, u64>(&bag, 0), 0);
        assert!(!contains_with_type<u64, u64>(&bag, 1), 0);
        
        bag::remove<u64, u64>(&mut bag, 0);
        bag::destroy_empty(bag);
    }

    #[test]
    fun sanity_check_size() {
        let bag = bag::new();
        assert!(bag::is_empty(&bag), 0);
        assert!(bag::length(&bag) == 0, 0);
        add(&mut bag, 0, 0);
        assert!(!bag::is_empty(&bag), 0);
        assert!(bag::length(&bag) == 1, 0);
        add(&mut bag, 1, 0);
        assert!(!bag::is_empty(&bag), 0);
        assert!(bag::length(&bag) == 2, 0);
        bag::remove<u64, u64>(&mut bag, 0);
        bag::remove<u64, u64>(&mut bag, 1);
        
        bag::destroy_empty(bag);
    }
}
