/// Utilities for comparing Move values
module moveos_std::compare {
    use std::vector;
    use std::type_name;
    use std::u256;
    use moveos_std::decimal_value::DecimalValue;

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
        } else if (t == type_name::get<DecimalValue>()) {
            let a_value = bcs::peel_u256(&mut a);
            let a_decimal = bcs::peel_u8(&mut a);
            let b_value = bcs::peel_u256(&mut b);
            let b_decimal = bcs::peel_u8(&mut b);
            // Normalise the decimal values
            let a = (a_value as u256) * u256::pow(10, b_decimal);
            let b = (b_value as u256) * u256::pow(10, a_decimal);

            if (a > b) {
                return GREATER_THAN
            } else if (a == b) {
                return EQUAL
            } else {
                return LESS_THAN
            }
        } else if (t == type_name::get<vector<u8>>() || t == type_name::get<std::string::String>() || t == type_name::get<std::ascii::String>()) {
            let a_value = bcs::peel_vec_u8(&mut a);
            let b_value = bcs::peel_vec_u8(&mut b);
            return compare_vector_u8(&a_value, &b_value)
        } else {
            return compare_vector_u8(&bcs::into_remainder_bytes(a), &bcs::into_remainder_bytes(b))
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
        // 1. Simple byte comparison
        let v1 = b"0x1";
        let v2 = b"0x2";
        assert!(compare_vector_u8(&v1, &v2) == LESS_THAN, 1);
        assert!(compare_vector_u8(&v2, &v1) == GREATER_THAN, 2);
        
        // 2. Equal length comparison
        let v3 = b"abc";
        let v4 = b"abd";
        assert!(compare_vector_u8(&v3, &v4) == LESS_THAN, 3);
        
        // 3. Different length comparison
        let v5 = b"ab";
        let v6 = b"abc";
        assert!(compare_vector_u8(&v5, &v6) == LESS_THAN, 4);
    }

    #[test]
    fun test_compare_string() {
        use std::string;
        
        // 1. ASCII comparison
        assert!(compare<string::String>(&string::utf8(b"a"), &string::utf8(b"b")) == LESS_THAN, 1);
        assert!(compare<string::String>(&string::utf8(b"b"), &string::utf8(b"a")) == GREATER_THAN, 2);
        
        // 2. Equal strings
        assert!(compare<string::String>(&string::utf8(b"abc"), &string::utf8(b"abc")) == EQUAL, 3);
        
        // 3. Length difference
        assert!(compare<string::String>(&string::utf8(b"ab"), &string::utf8(b"abc")) == LESS_THAN, 4);
        
        // 4. Empty string
        assert!(compare<string::String>(&string::utf8(b""), &string::utf8(b"a")) == LESS_THAN, 5);
    }

    #[test]
    fun test_compare_module_address() {
        use std::string;
        
        // 1. Simple address comparison
        assert!(compare<string::String>(&string::utf8(b"0x1"), &string::utf8(b"0x2")) == LESS_THAN, 1);
        assert!(compare<string::String>(&string::utf8(b"0x2"), &string::utf8(b"0x1")) == GREATER_THAN, 2);
        
        // 2. Full address comparison
        assert!(compare<string::String>(
            &string::utf8(b"0x0000000000000000000000000000000000000000000000000000000000000001"),
            &string::utf8(b"0x0000000000000000000000000000000000000000000000000000000000000002")
        ) == LESS_THAN, 3);
        
        // 3. Module comparison (same address)
        assert!(compare<string::String>(
            &string::utf8(b"0x1::coin"),
            &string::utf8(b"0x1::token")
        ) == LESS_THAN, 4);
        
        // 4. Full module path comparison
        assert!(compare<string::String>(
            &string::utf8(b"0x1::coin::CoinStore"),
            &string::utf8(b"0x1::coin::Supply")
        ) == LESS_THAN, 5);
    }

}