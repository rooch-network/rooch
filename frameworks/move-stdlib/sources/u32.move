// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

// Copyright (c) Mysten Labs, Inc.
// SPDX-License-Identifier: Apache-2.0

#[defines_primitive(u32)]
module std::u32 {
    /// Return the larger of `x` and `y`
    public fun max(x: u32, y: u32): u32 {
        if (x > y) {
            x
        } else {
            y
        }
    }

    /// Return the smaller of `x` and `y`
    public fun min(x: u32, y: u32): u32 {
        if (x < y) {
            x
        } else {
            y
        }
    }

    /// Return the absolute value of x - y
    public fun diff(x: u32, y: u32): u32 {
        if (x > y) {
            x - y
        } else {
            y - x
        }
    }

    /// Calculate x / y, but round up the result.
    public fun divide_and_round_up(x: u32, y: u32): u32 {
        if (x % y == 0) {
            x / y
        } else {
            x / y + 1
        }
    }

    /// Returns x * y / z with as little loss of precision as possible and avoid overflow
    public fun multiple_and_divide(x: u32, y: u32, z: u32): u32 {
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
    public fun pow(base: u32, exponent: u8): u32 {
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

    /// Get a nearest lower integer Square Root for `x`. Given that this
    /// function can only operate with integers, it is impossible
    /// to get perfect (or precise) integer square root for some numbers.
    ///
    /// Example:
    /// ```
    /// math::sqrt(9) => 3
    /// math::sqrt(8) => 2 // the nearest lower square root is 4;
    /// ```
    ///
    /// In integer math, one of the possible ways to get results with more
    /// precision is to use higher values or temporarily multiply the
    /// value by some bigger number. Ideally if this is a square of 10 or 100.
    ///
    /// Example:
    /// ```
    /// math::sqrt(8) => 2;
    /// math::sqrt(8 * 10000) => 282;
    /// // now we can use this value as if it was 2.82;
    /// // but to get the actual result, this value needs
    /// // to be divided by 100 (because sqrt(10000)).
    ///
    ///
    /// math::sqrt(8 * 1000000) => 2828; // same as above, 2828 / 1000 (2.828)
    /// ```
    public fun sqrt(x: u32): u32 {
        let bit = 1u64 << 32;
        let res = 0u64;
        let x = (x as u64);

        while (bit != 0) {
            if (x >= res + bit) {
                x = x - (res + bit);
                res = (res >> 1) + bit;
            } else {
                res = res >> 1;
            };
            bit = bit >> 2;
        };

        (res as u32)
    }
}