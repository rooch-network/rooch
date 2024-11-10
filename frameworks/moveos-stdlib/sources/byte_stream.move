// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

module moveos_std::byte_stream {

    use std::vector;

    const ERROR_NOT_ENOUGH_BYTES: u64 = 1;

    struct ByteStream has copy, drop, store {
        bytes: vector<u8>,
    }

    /// Create a new byte stream with the given bytes in big endian order.
    public fun new(bytes: vector<u8>): ByteStream {
        vector::reverse(&mut bytes);
        ByteStream { bytes}
    }

    public fun read_u8(stream: &mut ByteStream): u8 {
        assert!(vector::length(&stream.bytes) > 0, ERROR_NOT_ENOUGH_BYTES);
        vector::pop_back(&mut stream.bytes)
    }

    public fun read_u16(stream: &mut ByteStream): u16 {
        let bytes = read_to_vec(stream, 2);
        let result = 0u16;
        let i = 0;
        while (i < 2) {
            result = result << 8;
            result = result | (*vector::borrow(&bytes, i) as u16);
            i = i + 1;
        };
        result
    }

    public fun read_u32(stream: &mut ByteStream): u32 {
        let bytes = read_to_vec(stream, 4);
        let result = 0u32;
        let i = 0;
        while (i < 4) {
            result = result << 8;
            result = result | (*vector::borrow(&bytes, i) as u32);
            i = i + 1;
        };
        result
    }

    //TODO support read_u64, read_u128, read_u256

    public fun read_to_vec(stream: &mut ByteStream, len: u64): vector<u8> {
        assert!(vector::length(&stream.bytes) >= len, ERROR_NOT_ENOUGH_BYTES);
        let bytes = vector::empty<u8>();
        let i = 0;
        while (i < len) {
            vector::push_back(&mut bytes, read_u8(stream));
            i = i + 1;
        };
        bytes
    }

    public fun read_var_size(stream: &mut ByteStream, size: u8): u64 {
        let bytes = read_to_vec(stream, (size as u64));
        let result = 0u64;
        let i = 0u64;
        let size_u64 = (size as u64);
        while (i < size_u64) {
            result = result << 8;
            result = result | (*vector::borrow(&bytes, i) as u64);
            i = i + 1;
        };
        result
    }

    public fun skip(stream: &mut ByteStream, len: u64) {
        assert!(vector::length(&stream.bytes) >= len, ERROR_NOT_ENOUGH_BYTES);
        let i = 0;
        while (i < len) {
            vector::pop_back(&mut stream.bytes);
            i = i + 1;
        };
    }

    /// Read all remaining bytes in the stream.
    public fun read_all(stream: &mut ByteStream): vector<u8> {
        let len = vector::length(&stream.bytes);
        read_to_vec(stream, len)
    }

    #[test]
    fun test_byte_stream(){
        let bytes = x"0102030405060708090a0b0c0d0e0f";
        let stream = new(bytes);
        let v: u8 = read_u8(&mut stream);
        assert!(v == 0x01, 1);
        let result = read_u16(&mut stream);
        assert!(result == 0x0203, 2);
        let result = read_u32(&mut stream);
        assert!(result == 0x04050607, 3);
        let result = read_var_size(&mut stream, 3);
        assert!(result == 0x08090a, 4);
        let result = read_to_vec(&mut stream, 3);
        assert!(result == x"0b0c0d", 5);
        let result = read_all(&mut stream);
        std::debug::print(&result);
        assert!(result == x"0e0f", 6);
    }
}
