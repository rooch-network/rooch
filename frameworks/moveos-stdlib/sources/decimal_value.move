// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

module moveos_std::decimal_value {
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
}