// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

module moveos_std::decimal_value {
    use std::u256;

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
}