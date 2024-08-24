/// Utility functions for sorting vector.
module moveos_std::sort {

    use std::vector;
    use moveos_std::compare;
    
    /// Sorts a vector using quick sort algorithm.
    public fun quick_sort<T: copy + drop>(data: &mut vector<T>){
        let len = vector::length(data);
        if (len <= 1) {
            return
        };
        quick_sort_helper(data, 0, len - 1);
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

    inline fun bubble_sort<T: copy + drop>(data: &mut vector<T>, cmp: |&T, &T|bool) {
        let len = vector::length(data);
        let swapped = true;
        while(swapped) {
            swapped = false;
            let i = 1;
            while(i < len) {
                let a = vector::borrow(data, i - 1);
                let b = vector::borrow(data, i);
                if(cmp(a, b)) {
                    vector::swap(data, i - 1, i);
                    swapped = true;
                };
                i = i + 1;
            };
            len = len - 1;
        };
    }


    /// Sorts a vector, returning a new vector with the sorted elements.
    /// The sort algorithm used is quick sort, it maybe changed in the future.
    public fun sort<T: copy + drop>(data: &mut vector<T>){
        quick_sort(data)
    }

    /// Sorts a vector using a custom comparison function.
    /// The comparison function should return true if the first element is greater than the second.
    public inline fun sort_by_cmp<T: copy + drop>(data: &mut vector<T>, cmp: |&T, &T|bool){
        bubble_sort(data, |a,b|{cmp(a,b)});
    }

    /// Sorts a vector using a custom key function.
    public inline fun sort_by_key<T: copy + drop, K: copy + drop>(data: &mut vector<T>, key: |&T|K){
        bubble_sort(data, |a,b|{
            let a_key = key(a);
            let b_key = key(b);
            compare::compare(&a_key, &b_key) == compare::result_greater_than()
        });
    }
    
    #[test]
    fun test_quick_sort() {
        let data = vector<u64>[1, 3, 2, 5, 4];
        quick_sort(&mut data);
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
        quick_sort(&mut data);
        assert!(vector::length<u128>(&data) == 5, 0);
        assert!(*vector::borrow(&data, 0) == 1, 0);
        assert!(*vector::borrow(&data, 1) == 2, 0);
        assert!(*vector::borrow(&data, 2) == 3, 0);
        assert!(*vector::borrow(&data, 3) == 4, 0);
        assert!(*vector::borrow(&data, 4) == 5, 0);
    }

    #[test]
    fun test_sort_by_cmp(){
        let data = vector<u64>[1, 3, 2, 5, 4, 1];
        sort_by_cmp(&mut data, |a,b|{*a > *b});
        assert!(vector::length<u64>(&data) == 6, 0);
        assert!(*vector::borrow(&data, 0) == 1, 0);
        assert!(*vector::borrow(&data, 1) == 1, 0);
        assert!(*vector::borrow(&data, 2) == 2, 0);
        assert!(*vector::borrow(&data, 3) == 3, 0);
        assert!(*vector::borrow(&data, 4) == 4, 0);
        assert!(*vector::borrow(&data, 5) == 5, 0);
    }
}