/// Utilities for comparing Move values
module moveos_std::compare {
    use std::vector;
    use std::type_name;
    use moveos_std::bcs;

    // Move does not have signed integers, so we cannot use the usual 0, -1, 1 convention to
    // represent EQUAL, LESS_THAN, and GREATER_THAN. Instead, we fun a new convention using u8
    // constants:
    const EQUAL: u8 = 0;
    public fun result_equal(): u8 { EQUAL }

    const LESS_THAN: u8 = 1;
    public fun result_less_than(): u8 { LESS_THAN }

    const GREATER_THAN: u8 = 2;
    public fun result_greater_than(): u8 { GREATER_THAN }

    public fun compare<T>(v1: &T, v2: &T): u8 {
        let type = type_name::get<T>();

        if (type == type_name::get<u8>()) {
            let a = bcs::new(bcs::to_bytes(v1));
            let b = bcs::new(bcs::to_bytes(v2));
            let a = bcs::peel_u8(&mut a);
            let b = bcs::peel_u8(&mut b);
            if (a > b) {
                return GREATER_THAN
            }
            else if (a == b) {
                return EQUAL
            }
            else {
                return LESS_THAN
            }
        } else if (type == type_name::get<u16>()) {
            let a = bcs::new(bcs::to_bytes(v1));
            let b = bcs::new(bcs::to_bytes(v2));
            let a = bcs::peel_u16(&mut a);
            let b = bcs::peel_u16(&mut b);
            if (a > b) {
                return GREATER_THAN
            }
            else if (a == b) {
                return EQUAL
            }
            else {
                return LESS_THAN
            }
        } else if (type == type_name::get<u32>()) {
            let a = bcs::new(bcs::to_bytes(v1));
            let b = bcs::new(bcs::to_bytes(v2));
            let a = bcs::peel_u32(&mut a);
            let b = bcs::peel_u32(&mut b);
            if (a > b) {
                return GREATER_THAN
            }
            else if (a == b) {
                return EQUAL
            }
            else {
                return LESS_THAN
            }
        } else if (type == type_name::get<u64>()) {
            let a = bcs::new(bcs::to_bytes(v1));
            let b = bcs::new(bcs::to_bytes(v2));
            let a = bcs::peel_u64(&mut a);
            let b = bcs::peel_u64(&mut b);
            if (a > b) {
                return GREATER_THAN
            }
            else if (a == b) {
                return EQUAL
            }
            else {
                return LESS_THAN
            }
        } else if (type == type_name::get<u128>()) {
            let a = bcs::new(bcs::to_bytes(v1));
            let b = bcs::new(bcs::to_bytes(v2));
            let a = bcs::peel_u128(&mut a);
            let b = bcs::peel_u128(&mut b);
            if (a > b) {
                return GREATER_THAN
            }
            else if (a == b) {
                return EQUAL
            }
            else {
                return LESS_THAN
            }
        } else if (type == type_name::get<u256>()) {
            let a = bcs::new(bcs::to_bytes(v1));
            let b = bcs::new(bcs::to_bytes(v2));
            let a = bcs::peel_u256(&mut a);
            let b = bcs::peel_u256(&mut b);
            if (a > b) {
                return GREATER_THAN
            }
            else if (a == b) {
                return EQUAL
            }
            else {
                return LESS_THAN
            }
        } else {
            let a = bcs::to_bytes(v1);
            let b = bcs::to_bytes(v2);
            compare_vector_u8(&a, &b)
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
    public fun test_compare_integer() {
        let v1 = 8u8;
        let v2 = 1u8;
        assert!(compare<u8>(&v1, &v2) == GREATER_THAN, 8);

        let v1 = 16u16;
        let v2 = 20u16;
        assert!(compare<u16>(&v1, &v2) == LESS_THAN, 16);

        let v1 = 32u32;
        let v2 = 32u32;
        assert!(compare<u32>(&v1, &v2) == EQUAL, 32);
    }

    #[test_only]
    use std::string;

    #[test]
    public fun test_compare_string() {
        let value0 = string::utf8(b"alpha");
        let value1 = string::utf8(b"beta");
        let value2 = string::utf8(b"betaa");

        assert!(compare<string::String>(&value0, &value0) == EQUAL, 0);
        assert!(compare<string::String>(&value0, &value1) == GREATER_THAN, 1);
        assert!(compare<string::String>(&value2, &value0) == GREATER_THAN, 2);
        assert!(compare<string::String>(&value1, &value2) == LESS_THAN, 3);
    }
}
