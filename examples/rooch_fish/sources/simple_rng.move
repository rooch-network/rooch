// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

/// A simple random number generator in Move language.
module rooch_fish::simple_rng {
    use moveos_std::tx_context;
    use moveos_std::timestamp;
    use moveos_std::bcs;
    use std::vector;
    use std::hash;
    use std::option;
    use rooch_framework::transaction::{Self, TransactionSequenceInfo};

    const ErrorInvalidArg: u64 = 0;
    const ErrorInvalidU64: u64 = 1;
    const ErrorInvalidU128: u64 = 2;
    const ErrorInvalidSeed: u64 = 3;

    fun seed(nonce: u64): vector<u8> {
        let nonce_bytes = bcs::to_bytes(&nonce);

        let sequence_number = tx_context::sequence_number();
        let sequence_number_bytes = bcs::to_bytes(&sequence_number);

        let sender_addr = tx_context::sender();
        let sender_addr_bytes = bcs::to_bytes(&sender_addr);

        let timestamp_ms = timestamp::now_milliseconds();
        let timestamp_ms_bytes = bcs::to_bytes(&timestamp_ms);

        let seed_bytes = vector::empty<u8>();
        
        let tx_sequence_info_opt = tx_context::get_attribute<TransactionSequenceInfo>();
        if (option::is_some(&tx_sequence_info_opt)) {
            let tx_sequence_info = option::extract(&mut tx_sequence_info_opt);
            let tx_accumulator_root = transaction::tx_accumulator_root(&tx_sequence_info);
            let tx_accumulator_root_bytes = bcs::to_bytes(&tx_accumulator_root);
            vector::append(&mut seed_bytes, tx_accumulator_root_bytes);
        } else {
            let tx_hash = tx_context::tx_hash();
            let tx_hash_bytes = bcs::to_bytes(&tx_hash);
            vector::append(&mut seed_bytes, tx_hash_bytes);
        };

        vector::append(&mut seed_bytes, timestamp_ms_bytes);
        vector::append(&mut seed_bytes, sender_addr_bytes);
        vector::append(&mut seed_bytes, sequence_number_bytes);
        vector::append(&mut seed_bytes, nonce_bytes);

        hash::sha3_256(seed_bytes)
    }

    public fun bytes_to_u64(bytes: vector<u8>): u64 {
        let value = 0u64;
        let i = 0u64;
        while (i < 8) {
            value = value | ((*vector::borrow(&bytes, i) as u64) << ((8 * (7 - i)) as u8));
            i = i + 1;
        };
        return value
    }

    public fun bytes_to_u128(bytes: vector<u8>): u128 {
        let value = 0u128;
        let i = 0u64;
        while (i < 16) {
            value = value | ((*vector::borrow(&bytes, i) as u128) << ((8 * (15 - i)) as u8));
            i = i + 1;
        };
        return value
    }

    /// Generate a random u64 from seed
    public fun rand_u64(nonce: u64): u64 {
        let seed_bytes = seed(nonce);
        bytes_to_u64(seed_bytes)
    }

    /// Generate a random u128 from seed
    public fun rand_u128(nonce: u64): u128 {
        let seed_bytes = seed(nonce);
        bytes_to_u128(seed_bytes)
    }

    /// Generate a random integer range in [low, high) for u64.
    public fun rand_u64_range(nonce: u64, low: u64, high: u64): u64 {
        assert!(high > low, ErrorInvalidArg);
        let value = rand_u64(nonce);
        (value % (high - low)) + low
    }

    /// Generate a random integer range in [low, high) for u128.
    public fun rand_u128_range(nonce: u64, low: u128, high: u128): u128 {
        assert!(high > low, ErrorInvalidArg);
        let value = rand_u128(nonce);
        (value % (high - low)) + low
    }

    #[test]
    fun test_bytes_to_u64() {
        // binary: 01010001 11010011 10101111 11001100 11111101 00001001 10001110 11001101
        // bytes = [81, 211, 175, 204, 253, 9, 142, 205];
        let dec: u64 = 5896249632111562445;

        let bytes = vector::empty<u8>();
        vector::push_back(&mut bytes, 81);
        vector::push_back(&mut bytes, 211);
        vector::push_back(&mut bytes, 175);
        vector::push_back(&mut bytes, 204);
        vector::push_back(&mut bytes, 253);
        vector::push_back(&mut bytes, 9);
        vector::push_back(&mut bytes, 142);
        vector::push_back(&mut bytes, 205);

        let value = bytes_to_u64(bytes);
        assert!(value == dec, ErrorInvalidU64);
    }

    #[test]
    fun test_bytes_to_u128() {
        // Example binary: 00000001 00100011 01000101 01100111 10001001 10101011 11001101 11101111
        //                00000000 00100010 01000100 01100110 10001000 10101010 11001100 11101110
        // bytes = [1, 35, 69, 103, 137, 171, 205, 239, 0, 34, 68, 102, 136, 170, 204, 238];
        let dec: u128 = 0x0123456789abcdef0022446688aaccee;

        let bytes = vector::empty<u8>();
        vector::push_back(&mut bytes, 1);
        vector::push_back(&mut bytes, 35);
        vector::push_back(&mut bytes, 69);
        vector::push_back(&mut bytes, 103);
        vector::push_back(&mut bytes, 137);
        vector::push_back(&mut bytes, 171);
        vector::push_back(&mut bytes, 205);
        vector::push_back(&mut bytes, 239);
        vector::push_back(&mut bytes, 0);
        vector::push_back(&mut bytes, 34);
        vector::push_back(&mut bytes, 68);
        vector::push_back(&mut bytes, 102);
        vector::push_back(&mut bytes, 136);
        vector::push_back(&mut bytes, 170);
        vector::push_back(&mut bytes, 204);
        vector::push_back(&mut bytes, 238);

        let value = bytes_to_u128(bytes);
        assert!(value == dec, ErrorInvalidU128);
    }

    #[test]
    fun test_generate_seed() {
        // Test with nonce = 0
        let nonce = 0;
        
        // Mock sequence number
        let sequence_number = 0;
        let sequence_number_bytes = bcs::to_bytes(&sequence_number);
        
        // Mock sender address
        let sender_addr = tx_context::sender();
        let sender_addr_bytes = bcs::to_bytes(&sender_addr);
        
        // Mock timestamp
        let timestamp_ms = 0;
        let timestamp_ms_bytes = bcs::to_bytes(&timestamp_ms);
        
        // Mock tx hash
        let tx_hash = tx_context::tx_hash();
        let tx_hash_bytes = bcs::to_bytes(&tx_hash);

        let nonce_bytes = bcs::to_bytes(&nonce);

        let expected_seed_bytes = vector::empty<u8>();
        vector::append(&mut expected_seed_bytes, tx_hash_bytes);
        vector::append(&mut expected_seed_bytes, timestamp_ms_bytes);
        vector::append(&mut expected_seed_bytes, sender_addr_bytes);
        vector::append(&mut expected_seed_bytes, sequence_number_bytes);
        vector::append(&mut expected_seed_bytes, nonce_bytes);

        let expected_seed = hash::sha3_256(expected_seed_bytes);
        let seed_bytes = seed(nonce);

        assert!(seed_bytes == expected_seed, ErrorInvalidSeed);
    }

    #[test]
    fun test_rand_u64_range() {
        let nonce = 0;
        let low = 1;
        let high = 101;
        let value = rand_u64_range(nonce, low, high);
        assert!(value >= low && value < high, ErrorInvalidArg);
    }

    #[test]
    fun test_rand_u128_range() {
        let nonce = 0;
        let low = 1;
        let high = 101;
        let value = rand_u128_range(nonce, low, high);
        assert!(value >= low && value < high, ErrorInvalidArg);
    }

    #[test]
    #[expected_failure(abort_code = ErrorInvalidArg)]
    fun test_rand_u64_range_invalid_range() {
        rand_u64_range(0, 100, 50);
    }

    #[test]
    #[expected_failure(abort_code = ErrorInvalidArg)]
    fun test_rand_u128_range_invalid_range() {
        rand_u128_range(0, 100, 50);
    }
}
