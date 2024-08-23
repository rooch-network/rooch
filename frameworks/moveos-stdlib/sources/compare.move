/// Utilities for comparing Move values
module moveos_std::compare {
    use std::vector;
    use std::type_name;

    use moveos_std::bcs;

    // Move does not have signed integers, so we cannot use the usual 0, -1, 1 convention to
    // represent EQUAL, LESS_THAN, and GREATER_THAN. Instead, we fun a new convention using u8
    // constants:
    const EQUAL: u8 = 0;
    public fun result_equal(): u8 { EQUAL}

    const LESS_THAN: u8 = 1;
    public fun result_less_than(): u8 { LESS_THAN }
    
    const GREATER_THAN: u8 = 2;
    public fun result_greater_than(): u8 { GREATER_THAN }

    /// Compare two values of the same type
    /// This function will detect the type of the value and compare them accordingly
    /// If the type is numeric, it will compare the numeric value, otherwise it will compare the bytes
    public fun compare<T>(a: &T, b: &T): u8 {
        let t = type_name::get<T>();
        let a = bcs::new(bcs::to_bytes(a));
        let b = bcs::new(bcs::to_bytes(b));

        if (t == type_name::get<u64>()) {
            let a = bcs::peel_u64(&mut a);
            let b = bcs::peel_u64(&mut b);
            if (a > b) {
                return GREATER_THAN
            } else if (a == b) {
                return EQUAL
            } else {
                return LESS_THAN
            }
        } else if (t == type_name::get<u128>()) {
            let a = bcs::peel_u128(&mut a);
            let b = bcs::peel_u128(&mut b);
            if (a > b) {
                return GREATER_THAN
            } else if (a == b) {
                return EQUAL
            } else {
                return LESS_THAN
            }
        }else if (t == type_name::get<u256>()) {
            let a = bcs::peel_u256(&mut a);
            let b = bcs::peel_u256(&mut b);
            if (a > b) {
                return GREATER_THAN
            } else if (a == b) {
                return EQUAL
            } else {
                return LESS_THAN
            }
        }else if (t == type_name::get<u8>()) {
            let a = bcs::peel_u8(&mut a);
            let b = bcs::peel_u8(&mut b);
            if (a > b) {
                return GREATER_THAN
            } else if (a == b) {
                return EQUAL
            } else {
                return LESS_THAN
            }
        }else if (t == type_name::get<u16>()) {
            let a = bcs::peel_u16(&mut a);
            let b = bcs::peel_u16(&mut b);
            if (a > b) {
                return GREATER_THAN
            } else if (a == b) {
                return EQUAL
            } else {
                return LESS_THAN
            }
        }else if (t == type_name::get<u32>()) {
            let a = bcs::peel_u32(&mut a);
            let b = bcs::peel_u32(&mut b);
            if (a > b) {
                return GREATER_THAN
            } else if (a == b) {
                return EQUAL
            } else {
                return LESS_THAN
            }
        }else if (t == type_name::get<bool>()) {
            let a = bcs::peel_bool(&mut a);
            let b = bcs::peel_bool(&mut b);
            if (a == b) {
                return EQUAL
            } else if (a) {
                return GREATER_THAN
            } else {
                return LESS_THAN
            }
        }else {
            compare_vector_u8(&bcs::into_remainder_bytes(a), &bcs::into_remainder_bytes(b))
        }
    }
    
    /// Compare two vector<u8> values
    /// This function is different with std::compare::cmp_bcs_bytes, which compares the vector contents from right to left,
    /// But this function compares the vector contents from left to right.
    public fun compare_vector_u8(v1: &vector<u8>, v2: &vector<u8>): u8 {
        let v1_length = vector::length(v1);
        let v2_length = vector::length(v2);

        let idx = 0;

        while (idx < v1_length && idx < v2_length) {
            let v1_byte = *vector::borrow(v1, idx);
            let v2_byte = *vector::borrow(v2, idx);

            if (v1_byte < v2_byte) {
                return LESS_THAN
            } else if (v1_byte > v2_byte) {
                return GREATER_THAN
            };
            idx = idx + 1;
        };

        if (v1_length < v2_length) {
            LESS_THAN
        } else if (v1_length > v2_length) {
            GREATER_THAN
        } else {
            EQUAL
        }
    }

    public fun cmp_bcs_bytes(v1: &vector<u8>, v2: &vector<u8>): u8 {
        std::compare::cmp_bcs_bytes(v1, v2)
    }

    #[test]
    fun test_compare_u8() {
        let a: u8 = 1;
        let b: u8 = 2;
        assert!(compare(&a, &b) == LESS_THAN, 1);
        assert!(compare(&b, &a) == GREATER_THAN, 1);
        assert!(compare(&a, &a) == EQUAL, 1);
    }

    #[test]
    fun test_compare_u16() {
        let a: u16 = 1;
        let b: u16 = 2;
        assert!(compare(&a, &b) == LESS_THAN, 1);
        assert!(compare(&b, &a) == GREATER_THAN, 1);
        assert!(compare(&a, &a) == EQUAL, 1);
    }

    #[test]
    fun test_compare_u32() {
        let a: u32 = 1;
        let b: u32 = 2;
        assert!(compare(&a, &b) == LESS_THAN, 1);
        assert!(compare(&b, &a) == GREATER_THAN, 1);
        assert!(compare(&a, &a) == EQUAL, 1);
    }

    #[test]
    fun test_compare_u64() {
        let a: u64 = 1;
        let b: u64 = 2;
        assert!(compare(&a, &b) == LESS_THAN, 1);
        assert!(compare(&b, &a) == GREATER_THAN, 1);
        assert!(compare(&a, &a) == EQUAL, 1);
    }

    #[test]
    fun test_compare_u128() {
        let a: u128 = 1;
        let b: u128 = 2;
        assert!(compare(&a, &b) == LESS_THAN, 1);
        assert!(compare(&b, &a) == GREATER_THAN, 1);
        assert!(compare(&a, &a) == EQUAL, 1);
    }

    #[test]
    fun test_compare_u256() {
        let a: u256 = 1;
        let b: u256 = 2;
        assert!(compare(&a, &b) == LESS_THAN, 1);
        assert!(compare(&b, &a) == GREATER_THAN, 1);
        assert!(compare(&a, &a) == EQUAL, 1);
    }

    #[test]
    fun test_compare_bool() {
        let a: bool = true;
        let b: bool = false;
        assert!(compare(&a, &b) == GREATER_THAN, 1);
        assert!(compare(&b, &a) == LESS_THAN, 1);
        assert!(compare(&a, &a) == EQUAL, 1);
    }

    #[test]
    fun test_compare_vector_u8() {
        let a: vector<u8> = b"1";
        let b: vector<u8> = b"2";
        assert!(compare_vector_u8(&a, &b) == LESS_THAN, 1);
        assert!(compare_vector_u8(&b, &a) == GREATER_THAN, 1);
        assert!(compare_vector_u8(&a, &a) == EQUAL, 1);
    }

}