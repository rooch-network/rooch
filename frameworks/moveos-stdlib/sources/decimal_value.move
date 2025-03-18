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
}