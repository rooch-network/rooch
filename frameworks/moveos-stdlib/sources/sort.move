/// Utility functions for sorting vector.
module moveos_std::sort {

    use std::vector;
    use moveos_std::compare;
    
    /// Sorts a vector using quick sort algorithm.
    public fun quick_sort<T>(data: &mut vector<T>){
        let len = vector::length(data);
        if (len <= 1) {
            return
        };
        quick_sort_helper(data, 0, len - 1);
    }

    fun quick_sort_helper<T>(data: &mut vector<T>, low: u64, high: u64) {
        if (low < high) {
            let p = partition(data, low, high);
            if (p > 0) {
                quick_sort_helper(data, low, p - 1);
            };
            quick_sort_helper(data, p + 1, high);
        }
    }

    fun partition<T>(data: &mut vector<T>, low: u64, high: u64): u64 {
        let i = low;
        let j = low;
        while (j < high) {
            //for avoid pivot reference still alive when vector::swap
            //we need to borrow it in the while loop, not before the loop
            let pivot = vector::borrow(data, high);
            let value = vector::borrow(data, j);
            let cmp = compare::compare(value, pivot);
            if (cmp == compare::result_less_than()) {
                vector::swap(data, i, j);
                i = i + 1;
            };
            j = j + 1;
        };
        vector::swap(data, i, high);
        i
    }

    inline fun bubble_sort<T>(data: &mut vector<T>, cmp: |&T, &T|bool) {
        let len = vector::length(data);
        let swapped = true;
        while(swapped) {
            //we can not return in the inline function,
            //so put the length check here.
            if(len <= 1){
                break
            };
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
    public fun sort<T>(data: &mut vector<T>){
        quick_sort(data)
    }

    /// Sorts a vector using a custom comparison function.
    /// The comparison function should return true if the first element is greater than the second.
    public inline fun sort_by_cmp<T>(data: &mut vector<T>, cmp: |&T, &T|bool){
        bubble_sort(data, |a,b|{cmp(a,b)});
    }

    /// Sorts a vector using a custom key function.
    public inline fun sort_by_key<T, K>(data: &mut vector<T>, key: |&T|&K){
        bubble_sort(data, |a,b|{
            let a_key = key(a);
            let b_key = key(b);
            compare::compare(a_key, b_key) == compare::result_greater_than()
        });
    }
    
}