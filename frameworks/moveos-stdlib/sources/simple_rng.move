// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

/// A simple random number generator in Move language.
module moveos_std::simple_rng {
    use moveos_std::tx_context;
    use moveos_std::timestamp;

    // TODO: review

    // const EINVALID_ARG: u64 = 101;

    // fun generate_magic_number(): u128 {
    //     // generate a random number from tx_context
    //     let bytes = vector::empty<u8>();
    //     vector::append(&mut bytes, bcs::to_bytes(&tx_context::sequence_number()));
    //     vector::append(&mut bytes, bcs::to_bytes(&tx_context::sender()));
    //     vector::append(&mut bytes, bcs::to_bytes(&tx_context::tx_hash()));
    //     vector::append(&mut bytes, bcs::to_bytes(&timestamp::now_milliseconds()));

    //     let seed = hash::sha3_256(bytes);
    //     let magic_number = bytes_to_u128(seed);
    //     magic_number
    // }

    fun seed(): vector<u8> {
        // get sequence number
        let sequence_number = tx_context::sequence_number();
        let sequence_number_bytes = bcs::to_bytes(&sequence_number);

        // get sender address
        let sender_addr = tx_context::sender();
        let sender_addr_bytes = bcs::to_bytes(&addr);

        // get now milliseconds timestamp
        let timestamp_ms = timestamp::now_milliseconds();
        let timestamp_ms_bytes = bcs::to_bytes(&timestamp);
        
        // get tx accumulator root
        let tx_accumulator_root_bytes = tx_context::tx_accumulator_root();

        // construct a seed
        let seed_bytes = vector::empty<u8>();
        vector::append(&mut seed_bytes, tx_accumulator_root_bytes);
        vector::append(&mut seed_bytes, timestamp_ms_bytes);
        vector::append(&mut seed_bytes, sender_addr_bytes);
        vector::append(&mut seed_bytes, sequence_number_bytes);

        // hash seed bytes and return a seed
        let seed = hash::sha3_256(seed_bytes);
        seed
    }

    // TODO: review
    
    fun bytes_to_u128(bytes: vector<u8>): u128 {
        let value = 0u128;
        let i = 0u64;
        while (i < 16) {
            value = value | ((*Vector::borrow(&bytes, i) as u128) << ((8 * (15 - i)) as u8));
            i = i + 1;
        };
        return value
    }

    fun bytes_to_u64(bytes: vector<u8>): u64 {
        let value = 0u64;
        let i = 0u64;
        while (i < 8) {
            value = value | ((*Vector::borrow(&bytes, i) as u64) << ((8 * (7 - i)) as u8));
            i = i + 1;
        };
        return value
    }

    /// Generate a random u128
    public fun rand_u128(addr: &address): u128 acquires Counter {
        let _seed: vector<u8> = seed(addr);
        bytes_to_u128(_seed)
    }

    /// Generate a random integer range in [low, high).
    public fun rand_u128_range(addr: &address, low: u128, high: u128): u128 acquires Counter {
        assert!(high > low, EINVALID_ARG);
        let value = rand_u128(addr);
        (value % (high - low)) + low
    }

    /// Generate a random u64
    public fun rand_u64(addr: &address): u64 acquires Counter {
        let _seed: vector<u8> = seed(addr);
        bytes_to_u64(_seed)
    }

    /// Generate a random integer range in [low, high).
    public fun rand_u64_range(addr: &address, low: u64, high: u64): u64 acquires Counter {
        assert!(high > low, EINVALID_ARG);
        let value = rand_u64(addr);
        (value % (high - low)) + low
    }

    #[test]
    fun test_bytes_to_u64() {
        // binary: 01010001 11010011 10101111 11001100 11111101 00001001 10001110 11001101
        // bytes = [81, 211, 175, 204, 253, 9, 142, 205];
        let dec = 5896249632111562445;

        let bytes = Vector::empty<u8>();
        Vector::push_back(&mut bytes, 81);
        Vector::push_back(&mut bytes, 211);
        Vector::push_back(&mut bytes, 175);
        Vector::push_back(&mut bytes, 204);
        Vector::push_back(&mut bytes, 253);
        Vector::push_back(&mut bytes, 9);
        Vector::push_back(&mut bytes, 142);
        Vector::push_back(&mut bytes, 205);

        let value = bytes_to_u64(bytes);
        assert!(value == dec, 101);
    }
}