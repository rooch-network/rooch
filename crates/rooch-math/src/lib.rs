use num_traits::{Num, NumCast, One};
use std::ops::{Add, Div, Mul};

/// Returns the larger of two values.
///
/// If both values are equal, the function returns the first value.
///
/// # Examples
///
/// ```
/// use rooch_math::max;
///
/// let larger = max(5, 3);
/// assert_eq!(larger, 5);
///
/// ```
///
/// # Panics
///
/// This function does not panic.
///
/// # Errors
///
/// This function does not return errors.
///
/// # Safety
///
/// This function is safe to use with numeric types that implement the required traits.
///
/// # Parameters
///
/// - `a`: The first numeric value.
/// - `b`: The second numeric value.
pub fn max<T: Num + PartialOrd + Copy>(a: T, b: T) -> T {
    if a >= b {
        return a;
    } else {
        return b;
    }
}

/// Returns the smaller of two values.
///
/// If both values are equal, the function returns the first value.
///
/// # Examples
///
/// ```
/// use rooch_math::min;
///
/// let smaller = min(5, 3);
/// assert_eq!(smaller, 3);
///
/// ```
///
/// # Panics
///
/// This function does not panic.
///
/// # Errors
///
/// This function does not return errors.
///
/// # Safety
///
/// This function is safe to use with numeric types that implement the required traits.
///
/// # Parameters
///
/// - `a`: The first numeric value.
/// - `b`: The second numeric value.
pub fn min<T: Num + PartialOrd + Copy>(a: T, b: T) -> T {
    if a <= b {
        return a;
    } else {
        return b;
    }
}
/// Calculates the power of a numeric value raised to an unsigned integer exponent.
///
/// The function performs exponentiation by repeated multiplication.
///
/// # Examples
///
/// ```
/// use rooch_math::pow;
///
/// let result_int = pow(2i32, 3);
/// assert_eq!(result_int, 8);
///
/// let result_float = pow(2.5f64, 2);
/// assert_eq!(result_float, 6.25);
/// ```
///
/// # Panics
///
/// This function does not panic.
///
/// # Errors
///
/// This function does not return errors.
///
/// # Safety
///
/// This function is safe to use with numeric types that implement the required traits.
///
/// # Parameters
///
/// - `base`: The numeric base value.
/// - `exponent`: The non-negative integer exponent to which the base is raised.
///
/// # Returns
///
/// Returns the result of raising the base to the specified exponent.
///
/// # Notes
///
/// For floating-point types, the result may have rounding errors due to the nature of floating-point arithmetic.
pub fn pow<T: Num + One + Mul<Output = T> + Copy>(base: T, exponent: u32) -> T {
    (0..exponent).fold(T::one(), |acc, _| acc * base)
}

/// Calculates the square root of a numeric value using the Babylonian method.
///
/// If the value is greater than or equal to zero, returns the approximate square root.
/// For integer values, the result will be floored.
/// For example, `sqrt(3)` returns `Some(1)` but the actual square root is approximately `1.73205080757`.
/// Float values will provide a more accurate result.
///
/// If the value is less than zero, returns `None` since the square root is undefined for real numbers in that case.
///
/// # Examples
///
/// ```
/// use rooch_math::sqrt;
///
/// let result_positive = sqrt(4.0);
/// assert_eq!(result_positive, Some(2.0));
///
/// let result_negative = sqrt(-1.0);
/// assert_eq!(result_negative, None);
/// ```
///
/// # Panics
///
/// This function does not panic.
///
/// # Errors
///
/// Returns `None` if the value is less than zero, indicating that the square root is undefined for real numbers.
///
/// # Safety
///
/// This function is safe to use with numeric types that implement the required traits.
pub fn sqrt<T>(value: T) -> Option<T>
where
    T: std::ops::Add<Output = T> + std::ops::Div<Output = T> + Copy + PartialOrd + Num + NumCast,
{
    if value < T::zero() {
        None
    } else {
        let mut guess = value / num_traits::cast(2).unwrap();

        for _ in 0..10 {
            guess = (guess + value / guess) / num_traits::cast(2).unwrap();
        }

        Some(guess)
    }
}

/// Multiplies two values and then divides the result by a third value.
///
/// If the third value is zero, the function returns `None` to avoid division by zero.
///
/// # Examples
///
/// ```
/// use rooch_math::mul_div;
///
/// let result_valid = mul_div(4, 2, 3);
/// assert_eq!(result_valid, Some(2));
///
/// let result_invalid = mul_div(4, 2, 0);
/// assert_eq!(result_invalid, None);
/// ```
///
/// # Panics
///
/// This function does not panic.
///
/// # Errors
///
/// Returns `None` if the third value is zero, indicating division by zero.
///
/// # Safety
///
/// This function is safe to use with numeric types that implement the required traits.
///
/// # Parameters
///
/// - `a`: The first numeric value.
/// - `b`: The second numeric value.
/// - `c`: The third numeric value used for division.
///
/// # Returns
///
/// Returns the result of `(a * b) / c` if `c` is not zero, otherwise returns `None`.
pub fn mul_div<T>(a: T, b: T, c: T) -> Option<T>
where
    T: Num + Mul<Output = T> + Div<Output = T> + PartialEq + Copy,
{
    if c == T::zero() {
        None
    } else {
        Some((a * b) / c)
    }
}

/// Calculates the average of a collection of numeric values.
///
/// If the collection is empty, the function returns `None`.
///
/// # Examples
///
/// ```
/// use rooch_math::average;
///
/// let values = vec![1, 2, 3, 4];
/// let result = average(values);
/// assert_eq!(result, Some(2));
/// ```
///
/// # Panics
///
/// This function does not panic.
///
/// # Errors
///
/// Returns `None` if the collection is empty.
///
/// # Safety
///
/// This function is safe to use with numeric types that implement the required traits.
///
/// # Parameters
///
/// - `values`: A vector of numeric values.
///
/// # Returns
///
/// Returns `Some(average)` if the collection is not empty,
/// If value is not float average is going to be floored.
/// where `average` is the calculated average.
/// Returns `None` if the collection is empty.
pub fn average<T>(values: Vec<T>) -> Option<T>
where
    T: Num + Add<Output = T> + From<usize> + Copy,
{
    let mut result = T::zero();
    let len = values.len();
    if len == 0 {
        return None;
    }

    for value in values {
        result = result + value;
    }
    Some(result / len.into())
}

/// Calculates the sum of a collection of numeric values.
///
/// # Examples
///
/// ```
/// use rooch_math::sum;
///
/// let values = vec![1, 2, 3, 4];
/// let result = sum(values);
/// assert_eq!(result, 10);
/// ```
///
/// # Panics
///
/// This function does not panic.
///
/// # Errors
///
/// This function does not return errors.
///
/// # Safety
///
/// This function is safe to use with numeric types that implement the required traits.
///
/// # Parameters
///
/// - `values`: A vector of numeric values.
///
/// # Returns
///
/// Returns the sum of the numeric values in the collection.
pub fn sum<T>(values: Vec<T>) -> T
where
    T: Num + Add<Output = T> + Copy,
{
    let mut result = T::zero();
    for value in values {
        result = result + value;
    }
    result
}

/// Clamps a value within a specified range.
///
/// # Examples
///
/// ```
/// use rooch_math::clamp;
///
/// let result = clamp(5, 1, 10);
/// assert_eq!(result, 5);
///
/// let result_negative = clamp(-5, 1, 10);
/// assert_eq!(result_negative, 1);
///
/// let result_exceed_upper = clamp(15, 1, 10);
/// assert_eq!(result_exceed_upper, 10);
/// ```
///
/// # Panics
///
/// This function does not panic.
///
/// # Errors
///
/// This function does not return errors.
///
/// # Safety
///
/// This function is safe to use with numeric types that implement the required traits.
///
/// # Parameters
///
/// - `x`: The value to be clamped.
/// - `lower`: The lower bound of the range.
/// - `upper`: The upper bound of the range.
///
/// # Returns
///
/// Returns the clamped value within the specified range.
pub fn clamp<T>(x: T, lower: T, upper: T) -> T
where
    T: Num + Copy + PartialOrd,
{
    min(upper, max(lower, x))
}

#[cfg(test)]
mod tests {
    use crate::*;

    #[test]
    fn test_max() {
        let a = 10.0;
        let b = 25.0;

        let c = max(a, b);
        assert_eq!(c, 25.0);
    }
    #[test]
    fn test_min() {
        let a = 10;
        let b = 20;

        let c = min(a, b);
        assert_eq!(c, 10);
    }

    #[test]
    fn test_pow() {
        let a = 10.0;
        let b = 2;

        let c = pow(a, b);
        assert_eq!(c, 100.0);
    }

    #[test]
    fn test_sqrt() {
        let a = 10.0f32;

        let c = sqrt(a).unwrap();

        assert_eq!(c, 3.16227766017);
    }

    #[test]
    fn test_average() {
        let a = vec![2, 3, 4];
        let b = average(a);

        assert_eq!(b, Some(3));
    }
    #[test]
    fn test_sum() {
        let a = vec![2, 3, 4];
        let b = sum(a);

        assert_eq!(b, 9);
    }

    #[test]
    fn test_clamp() {
        let result = clamp(5, 1, 10);
        assert_eq!(result, 5);

        let result_negative = clamp(-5, 1, 10);
        assert_eq!(result_negative, 1);

        let result_exceed_upper = clamp(15, 1, 10);
        assert_eq!(result_exceed_upper, 10);
    }
}
