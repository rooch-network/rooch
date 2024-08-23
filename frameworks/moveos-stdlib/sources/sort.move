/// Utility functions for sorting vector.
module moveos_std::sort {

    use std::vector;

    use moveos_std::compare;
    
    /// Sorts a vector using quick sort algorithm.
    public fun quick_sort<T: copy + drop>(data: vector<T>): vector<T> {
        if (vector::length(&data) <= 1) {
            return data
        };
        let data_mut = data;
        let len = vector::length(&data_mut);
        quick_sort_helper(&mut data_mut, 0, len - 1);
        data_mut
    }

    fun quick_sort_helper<T: copy + drop>(data: &mut vector<T>, low: u64, high: u64) {
        if (low < high) {
            let p = partition(data, low, high);
            if (p > 0) {
                quick_sort_helper(data, low, p - 1);
            };
            quick_sort_helper(data, p + 1, high);
        }
    }

    fun partition<T: copy + drop>(data: &mut vector<T>, low: u64, high: u64): u64 {
        let pivot = *vector::borrow(data, high);
        let i = low;
        let j = low;
        while (j < high) {
            let value = vector::borrow(data, j);
            let cmp = compare::compare(value, &pivot);
            if (cmp == compare::result_less_than()) {
                vector::swap(data, i, j);
                i = i + 1;
            };
            j = j + 1;
        };
        vector::swap(data, i, high);
        i
    }

    /// Sorts a vector, returning a new vector with the sorted elements.
    /// The sort algorithm used is quick sort, it maybe changed in the future.
    public fun sort<T: copy + drop>(data: vector<T>): vector<T> {
        quick_sort(data)
    }
    
    #[test]
    fun test_quick_sort() {
        let data = vector<u64>[1, 3, 2, 5, 4];
        let sortedData = quick_sort(data);
        assert!(vector::length<u64>(&sortedData) == 5, 0);
        assert!(*vector::borrow(&sortedData, 0) == 1, 0);
        assert!(*vector::borrow(&sortedData, 1) == 2, 0);
        assert!(*vector::borrow(&sortedData, 2) == 3, 0);
        assert!(*vector::borrow(&sortedData, 3) == 4, 0);
        assert!(*vector::borrow(&sortedData, 4) == 5, 0);
    }

    #[test]
    fun test_quick_sort_u128() {
        let data = vector<u128>[1, 3, 2, 5, 4];
        let sortedData = quick_sort(data);
        assert!(vector::length<u128>(&sortedData) == 5, 0);
        assert!(*vector::borrow(&sortedData, 0) == 1, 0);
        assert!(*vector::borrow(&sortedData, 1) == 2, 0);
        assert!(*vector::borrow(&sortedData, 2) == 3, 0);
        assert!(*vector::borrow(&sortedData, 3) == 4, 0);
        assert!(*vector::borrow(&sortedData, 4) == 5, 0);
    }
}