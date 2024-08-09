/// Utilities for comparing Move values
module moveos_std::compare {
    use std::vector;

    // Move does not have signed integers, so we cannot use the usual 0, -1, 1 convention to
    // represent EQUAL, LESS_THAN, and GREATER_THAN. Instead, we fun a new convention using u8
    // constants:
    const EQUAL: u8 = 0;
    public fun result_equal(): u8 { EQUAL}

    const LESS_THAN: u8 = 1;
    public fun result_less_than(): u8 { LESS_THAN }
    
    const GREATER_THAN: u8 = 2;
    public fun result_greater_than(): u8 { GREATER_THAN }

    //TODO provide a generic compare function
    //We need to auto detect the type of the input values
    //public fun compare<T>(v1: &T, v2: &T): u8;
    
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
}