// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

#[test_only]
module moveos_std::sort_tests {

    use std::vector;
    use moveos_std::sort;

    #[test_only]
    struct TestStruct has drop{
        value: u64
    }

    #[test]
    fun test_quick_sort() {
        let data = vector<u64>[1, 3, 2, 5, 4];
        sort::quick_sort(&mut data);
        assert!(vector::length<u64>(&data) == 5, 0);
        assert!(*vector::borrow(&data, 0) == 1, 0);
        assert!(*vector::borrow(&data, 1) == 2, 0);
        assert!(*vector::borrow(&data, 2) == 3, 0);
        assert!(*vector::borrow(&data, 3) == 4, 0);
        assert!(*vector::borrow(&data, 4) == 5, 0);
    }

    #[test]
    fun test_quick_sort_u128() {
        let data = vector<u128>[1, 3, 2, 5, 4];
        sort::quick_sort(&mut data);
        assert!(vector::length<u128>(&data) == 5, 0);
        assert!(*vector::borrow(&data, 0) == 1, 0);
        assert!(*vector::borrow(&data, 1) == 2, 0);
        assert!(*vector::borrow(&data, 2) == 3, 0);
        assert!(*vector::borrow(&data, 3) == 4, 0);
        assert!(*vector::borrow(&data, 4) == 5, 0);
    }

    #[test]
    fun test_sort_by_cmp_u64(){
        let data = vector<u64>[1, 3, 2, 5, 4, 1];
        sort::sort_by_cmp(&mut data, |a,b|{*a > *b});
        assert!(vector::length<u64>(&data) == 6, 0);
        assert!(*vector::borrow(&data, 0) == 1, 0);
        assert!(*vector::borrow(&data, 1) == 1, 0);
        assert!(*vector::borrow(&data, 2) == 2, 0);
        assert!(*vector::borrow(&data, 3) == 3, 0);
        assert!(*vector::borrow(&data, 4) == 4, 0);
        assert!(*vector::borrow(&data, 5) == 5, 0);
    }

    #[test]
    fun test_sort_by_cmp_struct(){
        let data = vector<TestStruct>[
            TestStruct{value: 1},
            TestStruct{value: 3},
            TestStruct{value: 2},
            TestStruct{value: 5},
            TestStruct{value: 4},
            TestStruct{value: 1},
        ];
        sort::sort_by_cmp(&mut data, |a,b|{
            let a: &TestStruct = a;
            let b: &TestStruct = b;
            a.value > b.value
        }
        );
        assert!(vector::length(&data) == 6, 0);
        assert!(vector::borrow(&data, 0).value == 1, 0);
        assert!(vector::borrow(&data, 1).value == 1, 0);
        assert!(vector::borrow(&data, 2).value == 2, 0);
        assert!(vector::borrow(&data, 3).value == 3, 0);
        assert!(vector::borrow(&data, 4).value == 4, 0);
        assert!(vector::borrow(&data, 5).value == 5, 0);
    }

    #[test]
    fun test_sort_by_key(){
        let data = vector<TestStruct>[
            TestStruct{value: 1},
            TestStruct{value: 3},
            TestStruct{value: 2},
            TestStruct{value: 5},
            TestStruct{value: 4},
            TestStruct{value: 1},
        ];
        sort::sort_by_key(&mut data, |a|{
            let a: &TestStruct = a;
            &a.value
        }
        );
        assert!(vector::length(&data) == 6, 0);
        assert!(vector::borrow(&data, 0).value == 1, 0);
        assert!(vector::borrow(&data, 1).value == 1, 0);
        assert!(vector::borrow(&data, 2).value == 2, 0);
        assert!(vector::borrow(&data, 3).value == 3, 0);
        assert!(vector::borrow(&data, 4).value == 4, 0);
        assert!(vector::borrow(&data, 5).value == 5, 0);
    }
}