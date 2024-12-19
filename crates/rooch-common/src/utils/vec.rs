// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

// find the last true element in the array:
// the array is sorted by the predicate, and the predicate is true for the first n elements and false for the rest.
pub fn find_last_true<T>(arr: &[T], predicate: impl Fn(&T) -> bool) -> Option<&T> {
    if arr.is_empty() {
        return None;
    }
    if !predicate(&arr[0]) {
        return None;
    }
    if predicate(&arr[arr.len() - 1]) {
        return Some(&arr[arr.len() - 1]);
    }

    // binary search
    let mut left = 0;
    let mut right = arr.len() - 1;

    while left + 1 < right {
        let mid = left + (right - left) / 2;
        if predicate(&arr[mid]) {
            left = mid; // mid is true, the final answer is mid or on the right
        } else {
            right = mid; // mid is false, the final answer is on the left
        }
    }

    // left is the last true position
    Some(&arr[left])
}

#[cfg(test)]
mod tests {
    use super::*;

    mod find_last_true {
        use super::*;

        #[derive(Debug, PartialEq)]
        struct TestItem {
            id: usize,
            value: bool,
        }

        impl TestItem {
            fn new(id: usize, value: bool) -> Self {
                Self { id, value }
            }
        }

        #[test]
        fn test_empty_array() {
            let items: Vec<TestItem> = vec![];
            let result = find_last_true(&items, |item| item.value);
            assert!(result.is_none());
        }

        #[test]
        fn test_single_element_true() {
            let items = vec![TestItem::new(0, true)];
            let result = find_last_true(&items, |item| item.value).map(|item| item.id);
            assert_eq!(result, Some(0));
        }

        #[test]
        fn test_single_element_false() {
            let items = vec![TestItem::new(0, false)];
            let result = find_last_true(&items, |item| item.value);
            assert_eq!(result, None);
        }

        #[test]
        fn test_all_true() {
            let items = vec![
                TestItem::new(1, true),
                TestItem::new(2, true),
                TestItem::new(3, true),
            ];
            let result = find_last_true(&items, |item| item.value).map(|item| item.id);
            assert_eq!(result, Some(3));
        }

        #[test]
        fn test_all_false() {
            let items = vec![
                TestItem::new(1, false),
                TestItem::new(2, false),
                TestItem::new(3, false),
            ];
            let result = find_last_true(&items, |item| item.value);
            assert_eq!(result, None);
        }

        #[test]
        fn test_odd_length_middle_transition() {
            let items = vec![
                TestItem::new(1, true),
                TestItem::new(2, true),
                TestItem::new(3, true),
                TestItem::new(4, false),
                TestItem::new(5, false),
            ];
            let result = find_last_true(&items, |item| item.value).map(|item| item.id);
            assert_eq!(result, Some(3));
        }

        #[test]
        fn test_even_length_middle_transition() {
            let items = vec![
                TestItem::new(1, true),
                TestItem::new(2, true),
                TestItem::new(3, false),
                TestItem::new(4, false),
            ];
            let result = find_last_true(&items, |item| item.value).map(|item| item.id);
            assert_eq!(result, Some(2));
        }

        #[test]
        fn test_only_first_true() {
            let items = vec![
                TestItem::new(1, true),
                TestItem::new(2, false),
                TestItem::new(3, false),
                TestItem::new(4, false),
            ];
            let result = find_last_true(&items, |item| item.value).map(|item| item.id);
            assert_eq!(result, Some(1));
        }

        #[test]
        fn test_only_last_true() {
            let items = vec![
                TestItem::new(1, false),
                TestItem::new(2, false),
                TestItem::new(3, false),
                TestItem::new(4, true),
            ];
            let result = find_last_true(&items, |item| item.value);
            assert_eq!(result, None); // no sorted array, so no result
        }

        #[test]
        fn test_large_array() {
            let mut items = Vec::new();
            for i in 1..1000 {
                items.push(TestItem::new(i, i <= 500));
            }
            let result = find_last_true(&items, |item| item.value).map(|item| item.id);
            assert_eq!(result, Some(500));

            let mut items = Vec::new();
            for i in 1..1001 {
                items.push(TestItem::new(i, i <= 500));
            }
            let result = find_last_true(&items, |item| item.value).map(|item| item.id);
            assert_eq!(result, Some(500));

            let mut items = Vec::new();
            for i in 1..1001 {
                items.push(TestItem::new(i, i <= 501));
            }
            let result = find_last_true(&items, |item| item.value).map(|item| item.id);
            assert_eq!(result, Some(501));
        }

        #[test]
        fn test_various_transition_points() {
            // Test cases with different transition points
            let test_cases = [
                (vec![true], 0),
                (vec![true, false], 0),
                (vec![true, true, false], 1),
                (vec![true, true, true, false], 2),
                (vec![true, true, true, true, false], 3),
            ];

            for (i, (values, expected)) in test_cases.iter().enumerate() {
                let items: Vec<TestItem> = values
                    .iter()
                    .enumerate()
                    .map(|(id, &v)| TestItem::new(id, v))
                    .collect();

                let result = find_last_true(&items, |item| item.value).map(|item| item.id);
                assert_eq!(result, Some(*expected), "Failed at test case {}", i);
            }
        }
    }
}
