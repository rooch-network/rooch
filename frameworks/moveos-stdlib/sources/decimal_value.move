// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

module moveos_std::decimal_value {
    use std::string::{Self, String};
    use std::vector;
    use std::u256;
    use std::option::{Self, Option};

    // Error codes
    const ErrorUnderflow: u64 = 1;
    const ErrorDivisionByZero: u64 = 2;
    const ErrorInvalidPrecision: u64 = 3;
    const ErrorOverflow: u64 = 4;
    const ErrorInvalidDecimalString: u64 = 5;
    const ErrorDecimalPartTooLong: u64 = 6;

    #[data_struct]
    struct DecimalValue has store, drop, copy {
        value: u256,
        decimal: u8,
    }

    public fun new(value: u256, decimal: u8): DecimalValue {
        DecimalValue { value, decimal }
    }

    public fun value(self: &DecimalValue): u256 {
        self.value
    }

    public fun decimal(self: &DecimalValue): u8 {
        self.decimal
    }

    /// Create a new DecimalValue with the given decimal precision
    /// For example, convert 1.234 (value=1234, decimal=3) to 1.23400000 (value=123400000, decimal=8)
    public fun with_precision(self: &DecimalValue, new_decimal: u8): DecimalValue {
        if (self.decimal == new_decimal) {
            return *self
        };
        
        if (self.decimal < new_decimal) {
            // Increase precision (multiply)
            let scale_factor = u256::pow(10u256, (new_decimal - self.decimal));
            DecimalValue {
                value: self.value * scale_factor,
                decimal: new_decimal
            }
        } else {
            // Decrease precision (divide) - note: this can lose precision
            let scale_factor = u256::pow(10u256, (self.decimal - new_decimal));
            DecimalValue {
                value: self.value / scale_factor,
                decimal: new_decimal
            }
        }
    }

    /// Check if two DecimalValue instances represent the same numerical value
    public fun is_equal(a: &DecimalValue, b: &DecimalValue): bool {
        if (a.decimal == b.decimal) {
            // If decimal places match, just compare values directly
            return a.value == b.value
        };
        
        // Normalize to the larger decimal precision
        if (a.decimal > b.decimal) {
            // Scale up b.value
            let scale_factor = u256::pow(10u256, (a.decimal - b.decimal));
            b.value * scale_factor == a.value
        } else {
            // Scale up a.value
            let scale_factor = u256::pow(10u256, (b.decimal - a.decimal));
            a.value * scale_factor == b.value
        }
    }

    /// Add two DecimalValue instances
    public fun add(a: &DecimalValue, b: &DecimalValue): DecimalValue {
        // Normalize to the same decimal precision (use the larger one)
        let result_decimal = if (a.decimal > b.decimal) { a.decimal } else { b.decimal };
        
        let a_normalized = with_precision(a, result_decimal);
        let b_normalized = with_precision(b, result_decimal);
        
        // Add the normalized values
        DecimalValue {
            value: a_normalized.value + b_normalized.value,
            decimal: result_decimal
        }
    }
    
    /// Subtract b from a
    public fun sub(a: &DecimalValue, b: &DecimalValue): DecimalValue {
        // Normalize to the same decimal precision (use the larger one)
        let result_decimal = if (a.decimal > b.decimal) { a.decimal } else { b.decimal };
        
        let a_normalized = with_precision(a, result_decimal);
        let b_normalized = with_precision(b, result_decimal);
        
        // Ensure a >= b to avoid underflow
        assert!(a_normalized.value >= b_normalized.value, ErrorUnderflow);
        
        // Subtract the normalized values
        DecimalValue {
            value: a_normalized.value - b_normalized.value,
            decimal: result_decimal
        }
    }
    
    /// Multiply two DecimalValue instances
    public fun mul(a: &DecimalValue, b: &DecimalValue): DecimalValue {
        // When multiplying decimal values, we add the decimal places
        // For example: 1.23 * 4.56 = (123 * 456) / 10^(2+2) = 56088 / 10^4 = 5.6088
        
        // Multiply the raw values
        let result_value = a.value * b.value;
        
        // Add the decimal places
        let result_decimal = a.decimal + b.decimal;
        
        DecimalValue {
            value: result_value,
            decimal: result_decimal
        }
    }
    
    /// Divide a by b
    public fun div(a: &DecimalValue, b: &DecimalValue, precision: u8): DecimalValue {
        // Ensure b is not zero
        assert!(b.value > 0, ErrorDivisionByZero);
        
        // For division with decimal values, we need to adjust the scale
        // For example: 1.23 / 4.56 = (123 * 10^precision) / 456
        
        // Scale up the dividend to maintain precision
        let scale_factor = u256::pow(10u256, precision);
        let scaled_a = a.value * scale_factor;
        
        // Perform the division
        let result_value = scaled_a / b.value;
        
        // Calculate the result's decimal places: a.decimal + precision - b.decimal
        let result_decimal = a.decimal + precision;
        if (result_decimal >= b.decimal) {
            result_decimal = result_decimal - b.decimal;
        } else {
            // Handle underflow case (should be rare with sufficient precision)
            result_decimal = 0;
        };
        
        DecimalValue {
            value: result_value,
            decimal: result_decimal
        }
    }
    
    /// Multiply by an integer
    public fun mul_u256(a: &DecimalValue, b: u256): DecimalValue {
        DecimalValue {
            value: a.value * b,
            decimal: a.decimal
        }
    }
    
    /// Divide by an integer
    public fun div_u256(a: &DecimalValue, b: u256): DecimalValue {
        assert!(b > 0, ErrorDivisionByZero);
        
        DecimalValue {
            value: a.value / b,
            decimal: a.decimal
        }
    }
    
    /// Convert to integer part represented as a DecimalValue with decimal=0
    public fun as_integer_decimal(self: &DecimalValue): DecimalValue {
        if (self.decimal == 0) {
            return *self
        };
        
        let divisor = u256::pow(10u256, self.decimal);
        DecimalValue {
            value: self.value / divisor,
            decimal: 0
        }
    }

    /// Convert to integer by truncating decimal part and returning raw u256
    public fun to_integer(self: &DecimalValue): u256 {
        if (self.decimal == 0) {
            return self.value
        };
        
        let divisor = u256::pow(10u256, self.decimal);
        self.value / divisor
    }
    
    /// Round the decimal value to the specified number of decimal places
    public fun round(self: &DecimalValue, new_decimal: u8): DecimalValue {
        if (self.decimal <= new_decimal) {
            // If already has fewer decimal places, just adjust precision
            return with_precision(self, new_decimal)
        };
        
        // Get the digit after the rounding point
        let scale_down = u256::pow(10u256, (self.decimal - new_decimal - 1));
        let scale_up = u256::pow(10u256, (self.decimal - new_decimal));
        
        let rounding_digit = (self.value / scale_down) % 10;
        
        // Perform the rounding
        let rounded_value = self.value / scale_up;
        if (rounding_digit >= 5) {
            rounded_value = rounded_value + 1;
        };
        
        DecimalValue {
            value: rounded_value,
            decimal: new_decimal
        }
    }

    /// Parse a string representation of a decimal number into a DecimalValue
    /// Accepts strings like "123", "123.456", "0.123"
    public fun from_string(s: &String): DecimalValue {
        let bytes = string::bytes(s);
        let len = vector::length(bytes);
        
        // Empty string is invalid
        assert!(len > 0, ErrorInvalidDecimalString);
        
        let i = 0;
        let decimal_pos: Option<u64> = option::none();
        
        // Find decimal point position
        while (i < len) {
            let char = *vector::borrow(bytes, i);
            if (char == 46) { // '.' character
                // Ensure we don't have multiple decimal points
                assert!(option::is_none(&decimal_pos), ErrorInvalidDecimalString);
                decimal_pos = option::some(i);
            } else {
                // Ensure all other characters are digits
                assert!(char >= 48 && char <= 57, ErrorInvalidDecimalString); // '0' to '9'
            };
            i = i + 1;
        };
        
        let value = 0u256;
        let decimal = 0u8;
        
        if (option::is_none(&decimal_pos)) {
            // No decimal point, parse as integer
            i = 0;
            while (i < len) {
                let digit = ((*vector::borrow(bytes, i) - 48) as u8); // Convert ASCII to digit
                value = value * 10 + (digit as u256);
                i = i + 1;
            };
        } else {
            // Has decimal point
            let dp = option::extract(&mut decimal_pos);
            
            // Process integer part
            i = 0;
            while (i < dp) {
                let digit = ((*vector::borrow(bytes, i) - 48) as u8);
                value = value * 10 + (digit as u256);
                i = i + 1;
            };
            
            // Skip decimal point
            i = dp + 1;
            
            // Process decimal part
            let decimal_start = i;
            while (i < len) {
                let digit = ((*vector::borrow(bytes, i) - 48) as u8);
                value = value * 10 + (digit as u256);
                i = i + 1;
            };
            
            // Calculate decimal places
            decimal = ((len - decimal_start) as u8);
            
            // Safety check
            assert!(decimal <= 77, ErrorDecimalPartTooLong); // u256 can handle at most 77 decimal places
        };
        
        DecimalValue { value, decimal }
    }
    
    /// Convert a DecimalValue to its string representation
    /// Returns strings like "123", "123.456", "0.123"
    public fun to_string(d: &DecimalValue): String {
        if (d.value == 0) {
            return string::utf8(b"0")
        };

        if (d.decimal == 0) {
            // Simple integer case
            return internal_to_string_no_decimal(d.value)
        };
        
        let divisor = u256::pow(10u256, d.decimal);
        let int_part = d.value / divisor;
        let frac_part = d.value % divisor;
        
        if (frac_part == 0) {
            // No fractional part, just return integer
            return internal_to_string_no_decimal(int_part)
        };
        
        let int_str = internal_to_string_no_decimal(int_part);
        let frac_str = internal_to_string_no_decimal(frac_part);
        
        // Pad fractional part with leading zeros if needed
        let frac_len = string::length(&frac_str);
        let padding_zeros = d.decimal - (frac_len as u8);
        
        let result = int_str;
        string::append(&mut result, string::utf8(b"."));
        
        // Add leading zeros
        let i = 0;
        while (i < padding_zeros) {
            string::append(&mut result, string::utf8(b"0"));
            i = i + 1;
        };
        
        string::append(&mut result, frac_str);
        
        // Remove trailing zeros
        let result_bytes = string::bytes(&result);
        let len = vector::length(result_bytes);
        let last_non_zero = len;
        
        while (last_non_zero > 0) {
            let char = *vector::borrow(result_bytes, last_non_zero - 1);
            if (char != 48) { // '0'
                break
            };
            last_non_zero = last_non_zero - 1;
        };
        
        // If we removed all decimal part, remove the decimal point too
        if (last_non_zero > 0 && *vector::borrow(result_bytes, last_non_zero - 1) == 46) { // '.'
            last_non_zero = last_non_zero - 1;
        };
        
        string::sub_string(&result, 0, last_non_zero)
    }
    
    /// Helper function to convert a u256 to a string without decimal places
    fun internal_to_string_no_decimal(value: u256): String {
        if (value == 0) {
            return string::utf8(b"0")
        };
        
        let result = vector<u8>[];
        
        while (value > 0) {
            let digit = ((value % 10) as u8) + 48; // Convert to ASCII
            vector::push_back(&mut result, digit);
            value = value / 10;
        };
        
        // Reverse the result
        let len = vector::length(&result);
        let i = 0;
        let j = len - 1;
        
        while (i < j) {
            let temp = *vector::borrow(&result, i);
            *vector::borrow_mut(&mut result, i) = *vector::borrow(&result, j);
            *vector::borrow_mut(&mut result, j) = temp;
            i = i + 1;
            j = j - 1;
        };
        
        string::utf8(result)
    }

    #[test]
    fun test_decimal_equality() {
        // Same value, same decimal
        assert!(is_equal(
            &DecimalValue { value: 1234, decimal: 2 },
            &DecimalValue { value: 1234, decimal: 2 }
        ), 0);
        
        // Different representations of the same value
        assert!(is_equal(
            &DecimalValue { value: 10000, decimal: 4 }, // 1.0000
            &DecimalValue { value: 1, decimal: 0 }      // 1
        ), 1);
        
        assert!(is_equal(
            &DecimalValue { value: 100, decimal: 2 },   // 1.00
            &DecimalValue { value: 10, decimal: 1 }     // 1.0
        ), 2);
        
        // Different values
        assert!(!is_equal(
            &DecimalValue { value: 1234, decimal: 2 },  // 12.34
            &DecimalValue { value: 123, decimal: 1 }    // 12.3
        ), 3);
        
        // Zero edge case
        assert!(is_equal(
            &DecimalValue { value: 0, decimal: 4 },     // 0.0000
            &DecimalValue { value: 0, decimal: 0 }      // 0
        ), 4);
    }

    #[test]
    fun test_with_precision() {
        // Increase precision
        let d1 = DecimalValue { value: 1234, decimal: 3 }; // 1.234
        let d2 = with_precision(&d1, 8); // 1.23400000
        assert!(d2.value == 123400000, 0);
        assert!(d2.decimal == 8, 1);
        assert!(is_equal(&d1, &d2), 2);
        
        // Decrease precision
        let d3 = DecimalValue { value: 123456789, decimal: 8 }; // 1.23456789
        let d4 = with_precision(&d3, 3); // 1.234
        assert!(d4.value == 1234, 3);
        assert!(d4.decimal == 3, 4);
        // Note: this loses precision
        assert!(!is_equal(&d3, &d4), 5);
        
        // Same precision
        let d5 = DecimalValue { value: 1234, decimal: 3 };
        let d6 = with_precision(&d5, 3);
        assert!(d6.value == 1234, 6);
        assert!(d6.decimal == 3, 7);
        assert!(is_equal(&d5, &d6), 8);
    }

    #[test]
    fun test_addition() {
        let a = DecimalValue { value: 1234, decimal: 2 }; // 12.34
        let b = DecimalValue { value: 567, decimal: 2 };  // 5.67
        
        let c = add(&a, &b);
        assert!(c.value == 1801, 0); // 18.01
        assert!(c.decimal == 2, 1);
        
        // Different decimal places
        let d = DecimalValue { value: 123, decimal: 1 }; // 12.3
        let e = add(&a, &d);
        assert!(e.value == 2464, 2); // 24.64 
        assert!(e.decimal == 2, 3);
    }
    
    #[test]
    fun test_subtraction() {
        let a = DecimalValue { value: 1234, decimal: 2 }; // 12.34
        let b = DecimalValue { value: 567, decimal: 2 };  // 5.67
        
        let c = sub(&a, &b);
        assert!(c.value == 667, 0); // 6.67
        assert!(c.decimal == 2, 1);
        
        // Different decimal places
        let d = DecimalValue { value: 123, decimal: 1 }; // 12.3
        let e = sub(&a, &d);
        assert!(e.value == 4, 2); // 0.04
        assert!(e.decimal == 2, 3);
    }
    
    #[test]
    fun test_multiplication() {
        let a = DecimalValue { value: 1234, decimal: 2 }; // 12.34
        let b = DecimalValue { value: 567, decimal: 2 };  // 5.67
        
        let c = mul(&a, &b);
        assert!(c.value == 699678, 0); // 69.9678
        assert!(c.decimal == 4, 1);
        
        // Multiply by integer
        let d = mul_u256(&a, 10);
        assert!(d.value == 12340, 2); // 123.4
        assert!(d.decimal == 2, 3);
    }
    
    #[test]
    fun test_division() {
        let a = DecimalValue { value: 1234, decimal: 2 }; // 12.34
        let b = DecimalValue { value: 567, decimal: 2 };  // 5.67
        
        // Using precision 4
        let c = div(&a, &b, 4);
        assert!(c.value == 21763, 0); // 2.1763...
        assert!(c.decimal == 4, 1);
        
        // Divide by integer
        let d = div_u256(&a, 10);
        assert!(d.value == 123, 2); // 1.23
        assert!(d.decimal == 2, 3);
    }
    
    #[test]
    fun test_to_integer() {
        let a = DecimalValue { value: 1234, decimal: 2 }; // 12.34
        let b = as_integer_decimal(&a);
        assert!(b.value == 12, 0);
        assert!(b.decimal == 0, 1);
        assert!(to_integer(&a) == 12, 2);
        
        // Already an integer
        let c = DecimalValue { value: 123, decimal: 0 }; // 123
        let d = as_integer_decimal(&c);
        assert!(d.value == 123, 2);
        assert!(d.decimal == 0, 3);
        assert!(to_integer(&c) == 123, 4);
    }
    
    #[test]
    fun test_rounding() {
        let a = DecimalValue { value: 12345, decimal: 3 }; // 12.345
        
        // Round to 2 decimal places (rounding up, since 5 at rounding position)
        let b = round(&a, 2);
        //std::debug::print(&b);
        assert!(b.value == 1235, 0); // 12.35 (correct rounding of 12.345)
        assert!(b.decimal == 2, 1);
        
        // Test a case that should round down
        let c = DecimalValue { value: 12344, decimal: 3 }; // 12.344
        let d = round(&c, 2);
        //std::debug::print(&d);
        assert!(d.value == 1234, 2); // 12.34
        assert!(d.decimal == 2, 3);
        
        // Round to 0 decimal places
        let e = round(&a, 0);
        assert!(e.value == 12, 4); // 12
        assert!(e.decimal == 0, 5);
    }

    #[test]
    #[expected_failure(abort_code = ErrorUnderflow)]
    fun test_subtraction_underflow() {
        let a = DecimalValue { value: 100, decimal: 2 }; // 1.00
        let b = DecimalValue { value: 200, decimal: 2 }; // 2.00
        
        // This should fail with ErrorUnderflow
        sub(&a, &b);
    }
    
    #[test]
    #[expected_failure(abort_code = ErrorDivisionByZero)]
    fun test_division_by_zero() {
        let a = DecimalValue { value: 1234, decimal: 2 }; // 12.34
        let b = DecimalValue { value: 0, decimal: 2 };    // 0.00
        
        // This should fail with ErrorDivisionByZero
        div(&a, &b, 4);
    }

    #[test]
    fun test_from_string() {
        // Integer
        let s = string::utf8(b"12345");
        let d = from_string(&s);
        assert!(d.value == 12345, 0);
        assert!(d.decimal == 0, 1);
        
        // Decimal
        let s = string::utf8(b"123.45");
        let d = from_string(&s);
        assert!(d.value == 12345, 2);
        assert!(d.decimal == 2, 3);
        
        // Decimal with leading zero
        let s = string::utf8(b"0.123");
        let d = from_string(&s);
        assert!(d.value == 123, 4);
        assert!(d.decimal == 3, 5);
        
        // Zero
        let s = string::utf8(b"0");
        let d = from_string(&s);
        assert!(d.value == 0, 6);
        assert!(d.decimal == 0, 7);
        
        // Zero with decimal
        let s = string::utf8(b"0.0");
        let d = from_string(&s);
        assert!(d.value == 0, 8);
        assert!(d.decimal == 1, 9);
    }
    
    #[test]
    #[expected_failure(abort_code = ErrorInvalidDecimalString)]
    fun test_from_string_invalid() {
        // Invalid character
        let s = string::utf8(b"123a.45");
        from_string(&s);
    }
    
    #[test]
    #[expected_failure(abort_code = ErrorInvalidDecimalString)]
    fun test_from_string_multiple_decimal_points() {
        // Multiple decimal points
        let s = string::utf8(b"123.45.6");
        from_string(&s);
    }
    
    #[test]
    fun test_to_string() {
        // Integer
        let d = DecimalValue { value: 12345, decimal: 0 };
        let s = to_string(&d);
        assert!(s == string::utf8(b"12345"), 0);
        
        // Decimal
        let d = DecimalValue { value: 12345, decimal: 2 };
        let s = to_string(&d);
        assert!(s == string::utf8(b"123.45"), 1);
        
        // Small decimal
        let d = DecimalValue { value: 123, decimal: 3 };
        let s = to_string(&d);
        assert!(s == string::utf8(b"0.123"), 2);
        
        // Zero
        let d = DecimalValue { value: 0, decimal: 0 };
        let s = to_string(&d);
        assert!(s == string::utf8(b"0"), 3);
        
        // Trailing zeros
        let d = DecimalValue { value: 12300, decimal: 2 };
        let s = to_string(&d);
        assert!(s == string::utf8(b"123"), 4);
        
        // Leading zeros in decimal part
        let d = DecimalValue { value: 12, decimal: 3 };
        let s = to_string(&d);
        assert!(s == string::utf8(b"0.012"), 5);
    }
    
    #[test]
    fun test_round_trip() {
        // Test round-trip conversion from string to DecimalValue and back
        let test_cases = vector[
            b"123",
            b"123.45",
            b"0.123",
            b"0",
            b"999999.999999"
        ];
        
        let i = 0;
        let len = vector::length(&test_cases);
        
        while (i < len) {
            let s = string::utf8(*vector::borrow(&test_cases, i));
            let d = from_string(&s);
            let s2 = to_string(&d);
            assert!(s == s2, i);
            i = i + 1;
        };
    }
}