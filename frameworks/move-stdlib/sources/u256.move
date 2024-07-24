// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

// Copyright (c) Mysten Labs, Inc.
// SPDX-License-Identifier: Apache-2.0

#[defines_primitive(u256)]
module std::u256 {
    /// Return the larger of `x` and `y`
    public fun max(x: u256, y: u256): u256 {
        if (x > y) {
            x
        } else {
            y
        }
    }

    /// Return the smaller of `x` and `y`
    public fun min(x: u256, y: u256): u256 {
        if (x < y) {
            x
        } else {
            y
        }
    }

    /// Return the absolute value of x - y
    public fun diff(x: u256, y: u256): u256 {
        if (x > y) {
            x - y
        } else {
            y - x
        }
    }

    /// Calculate x / y, but round up the result.
    public fun divide_and_round_up(x: u256, y: u256): u256 {
        if (x % y == 0) {
            x / y
        } else {
            x / y + 1
        }
    }

    /// Returns x * y / z with as little loss of precision as possible and avoid overflow
    public fun multiple_and_divide(x: u256, y: u256, z: u256): u256 {
        if (y == z) {
            return x
        };
        if (x == z) {
            return y
        };

        let a = x / z;
        let b = x % z;
        let c = y / z;
        let d = y % z;
        let res = a * c * z + a * d + b * c + b * d / z;

        res
    }

    /// Return the value of a base raised to a power
    public fun pow(base: u256, exponent: u8): u256 {
        let res = 1;
        while (exponent >= 1) {
            if (exponent % 2 == 0) {
                base = base * base;
                exponent = exponent / 2;
            } else {
                res = res * base;
                exponent = exponent - 1;
            };
        };

        res
    }
}