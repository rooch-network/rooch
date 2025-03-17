// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use anyhow::anyhow;

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

/// Joins two vectors element-wise and extracts valid values from an `Option` in the second vector.
///
/// This function processes two vectors of equal length, `left` and `right`. Each pair of elements (from `left` and `right`)
/// is combined into a result after ensuring the `Option` inside `right` contains a value. If a value is missing (`None`),
/// a custom error is generated using the provided `err_msg` callback.
///
/// # Parameters
/// - `left`: The first vector containing any type of items.
/// - `right`: The second vector containing `Option<R>` elements.
/// - `err_msg`: A callback function that takes an element from `left` and generates a `String` error message
///   when the corresponding element in `right` is `None`.
///
/// # Returns
/// On success, returns a `Result<Vec<R>>` containing a vector with the successfully extracted values from `right`.
/// On failure, returns an `anyhow::Error` if the vector lengths do not match or if any element in `right` is `None`.
///
/// # Errors
/// - Returns an error if `left` and `right` have different lengths.
/// - Returns an error if any `Option` in the `right` vector is `None`, using the custom error message.
///
/// # Examples
/// ```
/// use rooch_common::vec::validate_and_extract;
/// let left = vec![1, 2, 3];
/// let right = vec![Some("a"), Some("b"), Some("c")];
/// let result = validate_and_extract(left, right, |l| format!("Missing right value for {}", l));
/// assert_eq!(result.unwrap(), vec!["a", "b", "c"]);
/// ```
pub fn validate_and_extract<F, L, R>(
    left: Vec<L>,
    right: Vec<Option<R>>,
    err_msg: F,
) -> anyhow::Result<Vec<R>>
where
    F: Fn(&L) -> String,
{
    if left.len() != right.len() {
        return Err(anyhow!("Vectors must have the same length"));
    }

    left.into_iter()
        .zip(right)
        .map(|(left_item, right_item)| {
            let value = right_item.ok_or_else(|| anyhow!(err_msg(&left_item)))?;
            Ok(value)
        })
        .collect()
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

        #[test]
        fn test_validate_and_extract_success() {
            let left = vec![1, 2, 3];
            let right = vec![Some("a"), Some("b"), Some("c")];
            let result =
                validate_and_extract(left, right, |l| format!("Missing right value for {}", l));
            assert_eq!(result.unwrap(), vec!["a", "b", "c"]);
        }

        #[test]
        fn test_validate_and_extract_none_value() {
            let left = vec![1, 2, 3];
            let right = vec![Some("a"), None, Some("c")];
            let result =
                validate_and_extract(left, right, |l| format!("Missing right value for {}", l));
            assert!(result.is_err());
            assert_eq!(result.unwrap_err().to_string(), "Missing right value for 2");
        }

        #[test]
        fn test_validate_and_extract_length_mismatch() {
            let left = vec![1, 2];
            let right = vec![Some("a"), Some("b"), Some("c")];
            let result =
                validate_and_extract(left, right, |l| format!("This should not be used: {}", l));
            assert!(result.is_err());
            assert_eq!(
                result.unwrap_err().to_string(),
                "Vectors must have the same length"
            );
        }

        #[test]
        fn test_validate_and_extract_empty_vectors() {
            let left: Vec<u32> = vec![];
            let right: Vec<Option<&str>> = vec![];
            let result =
                validate_and_extract(left, right, |_l| String::from("This should not happen"));
            assert!(result.is_ok());
            assert_eq!(result.unwrap(), Vec::<&str>::new());
        }

        #[test]
        fn test_validate_and_extract_custom_error_message() {
            let left = vec![10, 20, 30];
            let right = vec![Some(100), None, Some(300)];
            let result =
                validate_and_extract(left, right, |l| format!("Value missing for key: {}", l));
            assert!(result.is_err());
            assert_eq!(result.unwrap_err().to_string(), "Value missing for key: 20");
        }
    }
}
