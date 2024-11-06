// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

module moveos_std::i8 {
    const ErrorOverflow: u64 = 0;

    const MIN_AS_U8: u8 = 1 << 7;
    const MAX_AS_U8: u8 = 0x7f;

    const LT: u8 = 0;
    const EQ: u8 = 1;
    const GT: u8 = 2;

    #[data_struct]
    struct I8 has copy, drop, store {
        bits: u8
    }

    public fun zero(): I8 {
        I8 {
            bits: 0
        }
    }

    public fun from_u8(v: u8): I8 {
        I8 {
            bits: v
        }
    }

    public fun from(v: u8): I8 {
        assert!(v <= MAX_AS_U8, ErrorOverflow);
        I8 {
            bits: v
        }
    }

    public fun neg_from(v: u8): I8 {
        assert!(v <= MIN_AS_U8, ErrorOverflow);
        if (v == 0) {
            I8 {
                bits: v
            }
        } else {
            I8 {
                bits: (u8_neg(v) + 1) | (1 << 7)
            }
        }
    }

    public fun wrapping_add(num1: I8, num2: I8): I8 {
        let sum = num1.bits ^ num2.bits;
        let carry = (num1.bits & num2.bits) << 1;
        while (carry != 0) {
            let a = sum;
            let b = carry;
            sum = a ^ b;
            carry = (a & b) << 1;
        };
        I8 {
            bits: sum
        }
    }

    public fun add(num1: I8, num2: I8): I8 {
        let sum = wrapping_add(num1, num2);
        let overflow = (sign(num1) & sign(num2) & u8_neg(sign(sum))) +
                (u8_neg(sign(num1)) & u8_neg(sign(num2)) & sign(sum));
        assert!(overflow == 0, ErrorOverflow);
        sum
    }

    public fun wrapping_sub(num1: I8, num2: I8): I8 {
        let sub_num = wrapping_add(I8 {
            bits: u8_neg(num2.bits)
        }, from(1));
        wrapping_add(num1, sub_num)
    }

    public fun sub(num1: I8, num2: I8): I8 {
        let sub_num = wrapping_add(I8 {
            bits: u8_neg(num2.bits)
        }, from(1));
        add(num1, sub_num)
    }

    public fun mul(num1: I8, num2: I8): I8 {
        let product = abs_u8(num1) * abs_u8(num2);
        if (sign(num1) != sign(num2)) {
            return neg_from(product)
        };
        return from(product)
    }

    public fun div(num1: I8, num2: I8): I8 {
        let result = abs_u8(num1) / abs_u8(num2);
        if (sign(num1) != sign(num2)) {
            return neg_from(result)
        };
        return from(result)
    }

    public fun abs(v: I8): I8 {
        if (sign(v) == 0) {
            v
        } else {
            assert!(v.bits > MIN_AS_U8, ErrorOverflow);
            I8 {
                bits: u8_neg(v.bits - 1)
            }
        }
    }

    public fun abs_u8(v: I8): u8 {
        if (sign(v) == 0) {
            v.bits
        } else {
            u8_neg(v.bits - 1)
        }
    }

    public fun shl(v: I8, shift: u8): I8 {
        I8 {
            bits: v.bits << shift
        }
    }

    public fun shr(v: I8, shift: u8): I8 {
        if (shift == 0) {
            return v
        };
        let mask = 0xff << (8 - shift);
        if (sign(v) == 1) {
            return I8 {
                bits: (v.bits >> shift) | mask
            }
        };
        I8 {
            bits: v.bits >> shift
        }
    }

    public fun mod(v: I8, n: I8): I8 {
        if (sign(v) == 1) {
            neg_from((abs_u8(v) % abs_u8(n)))
        } else {
            from((as_u8(v) % abs_u8(n)))
        }
    }

    public fun as_u8(v: I8): u8 {
        v.bits
    }

    public fun sign(v: I8): u8 {
        ((v.bits >> 7) as u8)
    }

    public fun is_neg(v: I8): bool {
        sign(v) == 1
    }

    public fun cmp(num1: I8, num2: I8): u8 {
        if (num1.bits == num2.bits) return EQ;
        if (sign(num1) > sign(num2)) return LT;
        if (sign(num1) < sign(num2)) return GT;
        if (num1.bits > num2.bits) {
            return GT
        } else {
            return LT
        }
    }

    public fun eq(num1: I8, num2: I8): bool {
        num1.bits == num2.bits
    }

    public fun gt(num1: I8, num2: I8): bool {
        cmp(num1, num2) == GT
    }

    public fun gte(num1: I8, num2: I8): bool {
        cmp(num1, num2) >= EQ
    }

    public fun lt(num1: I8, num2: I8): bool {
        cmp(num1, num2) == LT
    }

    public fun lte(num1: I8, num2: I8): bool {
        cmp(num1, num2) <= EQ
    }

    public fun or(num1: I8, num2: I8): I8 {
        I8 {
            bits: (num1.bits | num2.bits)
        }
    }

    public fun and(num1: I8, num2: I8): I8 {
        I8 {
            bits: (num1.bits & num2.bits)
        }
    }

    fun u8_neg(v: u8): u8 {
        v ^ 0xff
    }

    #[test]
    fun test_from_ok() {
        assert!(as_u8(from(0)) == 0, 0);
        assert!(as_u8(from(10)) == 10, 1);
    }

    #[test]
    #[expected_failure]
    fun test_from_overflow() {
        as_u8(from(MIN_AS_U8));
        as_u8(from(0xff));
    }

    #[test]
    fun test_neg_from() {
        assert!(as_u8(neg_from(0)) == 0, 0);
        assert!(as_u8(neg_from(1)) == 0xff, 1);
        assert!(as_u8(neg_from(0x7f)) == 129, 2);
        assert!(as_u8(neg_from(MIN_AS_U8)) == MIN_AS_U8, 2);
    }

    #[test]
    #[expected_failure]
    fun test_neg_from_overflow() {
        neg_from(MIN_AS_U8+1);
    }

    #[test]
    fun test_abs() {
        assert!(as_u8(from(10)) == 10u8, 0);
        assert!(as_u8(abs(neg_from(10))) == 10u8, 1);
        assert!(as_u8(abs(neg_from(0))) == 0u8, 2);
        assert!(as_u8(abs(neg_from(0x7f))) == 0x7f, 3);
        assert!(as_u8(neg_from(MIN_AS_U8)) == MIN_AS_U8, 4);
    }

    #[test]
    #[expected_failure]
    fun test_abs_overflow() {
        abs(neg_from(0x80));
    }

    #[test]
    fun test_wrapping_add() {
        assert!(as_u8(wrapping_add(from(0), from(1))) == 1, 0);
        assert!(as_u8(wrapping_add(from(1), from(0))) == 1, 0);
        assert!(as_u8(wrapping_add(from(100), from(27))) == 127, 0);
        assert!(as_u8(wrapping_add(from(27), from(100))) == 127, 0);
        assert!(as_u8(wrapping_add(from(MAX_AS_U8 - 1), from(1))) == MAX_AS_U8, 0);
        assert!(as_u8(wrapping_add(from(0), from(0))) == 0, 0);

        assert!(as_u8(wrapping_add(neg_from(0), neg_from(0))) == 0, 1);
        assert!(as_u8(wrapping_add(neg_from(1), neg_from(0))) == 0xff, 1);
        assert!(as_u8(wrapping_add(neg_from(0), neg_from(1))) == 0xff, 1);
        assert!(as_u8(wrapping_add(neg_from(100), neg_from(27))) == 129, 1);
        assert!(as_u8(wrapping_add(neg_from(27), neg_from(100))) == 129, 1);
        assert!(as_u8(wrapping_add(neg_from(MIN_AS_U8 - 1), neg_from(1))) == MIN_AS_U8, 1);

        assert!(as_u8(wrapping_add(from(0), neg_from(0))) == 0, 2);
        assert!(as_u8(wrapping_add(neg_from(0), from(0))) == 0, 2);
        assert!(as_u8(wrapping_add(neg_from(1), from(1))) == 0, 2);
        assert!(as_u8(wrapping_add(from(1), neg_from(1))) == 0, 2);
        assert!(as_u8(wrapping_add(from(100), neg_from(27))) == 73, 2);
        assert!(as_u8(wrapping_add(from(27), neg_from(100))) == 183, 2);
        assert!(as_u8(wrapping_add(neg_from(MIN_AS_U8), from(1))) == 129, 2);
        assert!(as_u8(wrapping_add(from(MAX_AS_U8), neg_from(1))) == 126, 2);

        assert!(as_u8(wrapping_add(from(MAX_AS_U8), from(1))) == MIN_AS_U8, 2);
    }

    #[test]
    fun test_add() {
        assert!(as_u8(add(from(0), from(0))) == 0, 0);
        assert!(as_u8(add(from(0), from(1))) == 1, 0);
        assert!(as_u8(add(from(1), from(0))) == 1, 0);
        assert!(as_u8(add(from(100), from(27))) == 127, 0);
        assert!(as_u8(add(from(27), from(100))) == 127, 0);
        assert!(as_u8(add(from(MAX_AS_U8 - 1), from(1))) == MAX_AS_U8, 0);

        assert!(as_u8(add(neg_from(0), neg_from(0))) == 0, 1);
        assert!(as_u8(add(neg_from(1), neg_from(0))) == 0xff, 1);
        assert!(as_u8(add(neg_from(0), neg_from(1))) == 0xff, 1);
        //std::debug::print(&as_u8(add(neg_from(100), neg_from(27))));
        assert!(as_u8(add(neg_from(100), neg_from(27))) == 129, 1);
        assert!(as_u8(add(neg_from(27), neg_from(100))) == 129, 1);
        assert!(as_u8(add(neg_from(MIN_AS_U8 - 1), neg_from(1))) == MIN_AS_U8, 1);

        assert!(as_u8(add(from(0), neg_from(0))) == 0, 2);
        assert!(as_u8(add(neg_from(0), from(0))) == 0, 2);
        assert!(as_u8(add(neg_from(1), from(1))) == 0, 2);
        assert!(as_u8(add(from(1), neg_from(1))) == 0, 2);
        assert!(as_u8(add(from(100), neg_from(99))) == 1, 2);
        // 99 - 100 = -1
        assert!(as_u8(add(from(99), neg_from(100))) == 255, 2);
        // -128 - 127 = -1
        assert!(as_u8(add(neg_from(MIN_AS_U8), from(MAX_AS_U8))) == 255, 2);
        assert!(as_u8(add(from(MAX_AS_U8), neg_from(1))) == MAX_AS_U8 - 1, 2);
    }

    #[test]
    #[expected_failure]
    fun test_add_overflow() {
        add(from(MAX_AS_U8), from(1));
    }

    #[test]
    #[expected_failure]
    fun test_add_underflow() {
        add(neg_from(MIN_AS_U8), neg_from(1));
    }

    #[test]
    fun test_wrapping_sub() {
        assert!(as_u8(wrapping_sub(from(0), from(0))) == 0, 0);
        assert!(as_u8(wrapping_sub(from(1), from(0))) == 1, 0);
        assert!(as_u8(wrapping_sub(from(0), from(1))) == as_u8(neg_from(1)), 0);
        assert!(as_u8(wrapping_sub(from(1), from(1))) == as_u8(neg_from(0)), 0);
        assert!(as_u8(wrapping_sub(from(1), neg_from(1))) == as_u8(from(2)), 0);
        assert!(as_u8(wrapping_sub(neg_from(1), from(1))) == as_u8(neg_from(2)), 0);
        assert!(as_u8(wrapping_sub(from(100), from(1))) == 99, 0);
        assert!(as_u8(wrapping_sub(neg_from(100), neg_from(1))) == as_u8(neg_from(99)), 0);
        assert!(as_u8(wrapping_sub(from(1), from(100))) == as_u8(neg_from(99)), 0);
        assert!(as_u8(wrapping_sub(from(MAX_AS_U8), from(MAX_AS_U8))) == as_u8(from(0)), 0);
        assert!(as_u8(wrapping_sub(from(MAX_AS_U8), from(1))) == as_u8(from(MAX_AS_U8 - 1)), 0);
        assert!(as_u8(wrapping_sub(from(MAX_AS_U8), neg_from(1))) == as_u8(neg_from(MIN_AS_U8)), 0);
        assert!(as_u8(wrapping_sub(neg_from(MIN_AS_U8), neg_from(1))) == as_u8(neg_from(MIN_AS_U8 - 1)), 0);
        assert!(as_u8(wrapping_sub(neg_from(MIN_AS_U8), from(1))) == as_u8(from(MAX_AS_U8)), 0);
    }

    #[test]
    fun test_sub() {
        assert!(as_u8(sub(from(0), from(0))) == 0, 0);
        assert!(as_u8(sub(from(1), from(0))) == 1, 0);
        assert!(as_u8(sub(from(0), from(1))) == as_u8(neg_from(1)), 0);
        assert!(as_u8(sub(from(1), from(1))) == as_u8(neg_from(0)), 0);
        assert!(as_u8(sub(from(1), neg_from(1))) == as_u8(from(2)), 0);
        assert!(as_u8(sub(neg_from(1), from(1))) == as_u8(neg_from(2)), 0);
        assert!(as_u8(sub(from(100), from(1))) == 99, 0);
        assert!(as_u8(sub(neg_from(100), neg_from(1))) == as_u8(neg_from(99)), 0);
        assert!(as_u8(sub(from(1), from(100))) == as_u8(neg_from(99)), 0);
        assert!(as_u8(sub(from(MAX_AS_U8), from(MAX_AS_U8))) == as_u8(from(0)), 0);
        assert!(as_u8(sub(from(MAX_AS_U8), from(1))) == as_u8(from(MAX_AS_U8 - 1)), 0);
        assert!(as_u8(sub(neg_from(MIN_AS_U8), neg_from(1))) == as_u8(neg_from(MIN_AS_U8 - 1)), 0);
    }

    #[test]
    #[expected_failure]
    fun test_sub_overflow() {
        sub(from(MAX_AS_U8), neg_from(1));
    }

    #[test]
    #[expected_failure]
    fun test_sub_underflow() {
        sub(neg_from(MIN_AS_U8), from(1));
    }

    #[test]
    fun test_mul() {
        assert!(as_u8(mul(from(1), from(1))) == 1, 0);
        assert!(as_u8(mul(from(10), from(10))) == 100, 0);

        assert!(as_u8(mul(neg_from(1), from(1))) == as_u8(neg_from(1)), 0);
        assert!(as_u8(mul(neg_from(10), from(10))) == as_u8(neg_from(100)), 0);

        assert!(as_u8(mul(from(1), neg_from(1))) == as_u8(neg_from(1)), 0);
        assert!(as_u8(mul(from(10), neg_from(10))) == as_u8(neg_from(100)), 0);
        assert!(as_u8(mul(from(MIN_AS_U8 / 2), neg_from(2))) == as_u8(neg_from(MIN_AS_U8)), 0);
    }

    #[test]
    #[expected_failure]
    fun test_mul_overflow() {
        mul(from(MIN_AS_U8 / 2), from(1));
        mul(neg_from(MIN_AS_U8 / 2), neg_from(2));
    }

    #[test]
    fun test_div() {
        assert!(as_u8(div(from(0), from(1))) == 0, 0);
        assert!(as_u8(div(from(10), from(1))) == 10, 0);
        assert!(as_u8(div(from(10), neg_from(1))) == as_u8(neg_from(10)), 0);
        assert!(as_u8(div(neg_from(10), neg_from(1))) == as_u8(from(10)), 0);

        assert!(abs_u8(neg_from(MIN_AS_U8)) == MIN_AS_U8, 0);
        assert!(as_u8(div(neg_from(MIN_AS_U8), from(1))) == MIN_AS_U8, 0);
    }

    #[test]
    #[expected_failure]
    fun test_div_overflow() {
        div(neg_from(MIN_AS_U8), neg_from(1));
    }

    #[test]
    fun test_shl() {
        assert!(as_u8(shl(from(10), 0)) == 10, 0);
        assert!(as_u8(shl(neg_from(10), 0)) == as_u8(neg_from(10)), 0);

        assert!(as_u8(shl(from(10), 1)) == 20, 0);
        assert!(as_u8(shl(neg_from(10), 1)) == as_u8(neg_from(20)), 0);

        assert!(as_u8(shl(from(10), 3)) == 80, 0);
        assert!(as_u8(shl(neg_from(10), 3)) == as_u8(neg_from(80)), 0);

        assert!(as_u8(shl(from(10), 7)) == 0, 0);
        assert!(as_u8(shl(neg_from(10), 7)) == 0, 0);
    }

    #[test]
    fun test_shr() {
        assert!(as_u8(shr(from(10), 0)) == 10, 0);
        assert!(as_u8(shr(neg_from(10), 0)) == as_u8(neg_from(10)), 0);

        assert!(as_u8(shr(from(10), 1)) == 5, 0);
        assert!(as_u8(shr(neg_from(10), 1)) == as_u8(neg_from(5)), 0);

        assert!(as_u8(shr(from(MAX_AS_U8), 7)) == 0, 0);
        assert!(as_u8(shr(neg_from(MIN_AS_U8), 7)) == 255, 0);
    }

    #[test]
    fun test_sign() {
        assert!(sign(neg_from(10)) == 1u8, 0);
        assert!(sign(from(10)) == 0u8, 0);
    }

    #[test]
    fun test_cmp() {
        assert!(cmp(from(1), from(0)) == GT, 0);
        assert!(cmp(from(0), from(1)) == LT, 0);

        assert!(cmp(from(0), neg_from(1)) == GT, 0);
        assert!(cmp(neg_from(0), neg_from(1)) == GT, 0);
        assert!(cmp(neg_from(1), neg_from(0)) == LT, 0);
        assert!(!lt(from(47), neg_from(47)), 0);

        assert!(cmp(neg_from(MIN_AS_U8), from(MAX_AS_U8)) == LT, 0);
        assert!(cmp(from(MAX_AS_U8), neg_from(MIN_AS_U8)) == GT, 0);

        assert!(cmp(from(MAX_AS_U8), from(MAX_AS_U8 - 1)) == GT, 0);
        assert!(cmp(from(MAX_AS_U8 - 1), from(MAX_AS_U8)) == LT, 0);

        assert!(cmp(neg_from(MIN_AS_U8), neg_from(MIN_AS_U8 - 1)) == LT, 0);
        assert!(cmp(neg_from(MIN_AS_U8 - 1), neg_from(MIN_AS_U8)) == GT, 0);
    }

    #[test]
    fun test_mod() {
        //use aptos_std::debug;
        let i = mod(neg_from(2), from(5));
        assert!(cmp(i, neg_from(2)) == EQ, 0);

        i = mod(neg_from(2), neg_from(5));
        assert!(cmp(i, neg_from(2)) == EQ, 0);

        i = mod(from(2), from(5));
        assert!(cmp(i, from(2)) == EQ, 0);

        i = mod(from(2), neg_from(5));
        assert!(cmp(i, from(2)) == EQ, 0);
    }
}
