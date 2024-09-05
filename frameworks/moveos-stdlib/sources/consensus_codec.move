// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

/// This module implements the Bitcoin consensus encode/decode functions
module moveos_std::consensus_codec{

    use std::vector;

    const ErrorInvalidLength: u64 = 1;
    const ErrorNonMinimalVarInt: u64 = 2;
    const ErrorOutOfRange: u64 = 3;

    struct Encoder has copy, drop, store{
        data: vector<u8>,
    }

    struct Decoder has copy, drop, store {
        data: vector<u8>,
    }

    public fun encoder(): Encoder{
        Encoder{
            data: vector::empty()
        }
    }

    public fun decoder(data: vector<u8>): Decoder{
        // Reverse the data to make peeling easier
        vector::reverse(&mut data);
        Decoder{
            data
        }
    }

    public fun unpack_encoder(encoder: Encoder): vector<u8> {
        let Encoder{data} = encoder;
        data
    }

    public fun unpack_decoder(decoder: Decoder): vector<u8> {
        let Decoder{data} = decoder;
        // Because we reverse the data in `decoder`, we need to reverse it back to the original order
        vector::reverse(&mut data);
        data
    }

    public fun emit_u64(encoder: &mut Encoder, v: u64) {
        let i = 0;
        while (i < 8) {
            vector::push_back(&mut encoder.data, (((v >> (i * 8)) & 0xFF) as u8));
            i = i + 1;
        }
    }

    public fun emit_u32(encoder: &mut Encoder, v: u32) {
        let i = 0;
        while (i < 4) {
            vector::push_back(&mut encoder.data, (((v >> (i * 8)) & 0xFF) as u8));
            i = i + 1;
        }
    }

    public fun emit_u16(encoder: &mut Encoder, v: u16) {
        let i = 0;
        while (i < 2) {
            vector::push_back(&mut encoder.data, (((v >> (i * 8)) & 0xFF) as u8));
            i = i + 1;
        }
    }   

    public fun emit_u8(encoder: &mut Encoder, v: u8) {
        vector::push_back(&mut encoder.data, v);
    }
    
    //TODO should we support signed integers?

    public fun emit_bool(encoder: &mut Encoder, v: bool) {
        let value = if (v) 1u8 else 0u8;
        vector::push_back(&mut encoder.data, value);
    }

    public fun emit_var_int(encoder: &mut Encoder, v: u64) {
        if (v <= 0xFC) {
            emit_u8(encoder, (v as u8));
        } else if (v <= 0xFFFF) {
            emit_u8(encoder, 0xFD);
            emit_u16(encoder, (v as u16));
        } else if (v <= 0xFFFFFFFF) {
            emit_u8(encoder, 0xFE);
            emit_u32(encoder, (v as u32));
        } else {
            emit_u8(encoder, 0xFF);
            emit_u64(encoder, v);
        }
    }

    /// Emit a slice of bytes to the encoder
    /// This function appends the entire input vector to the encoder's data, 
    /// without any encoding or length indication.
    fun emit_slice(encoder: &mut Encoder, v: vector<u8>) {
        vector::append(&mut encoder.data, v);
    }

    /// Emit a slice of bytes to the encoder with a varint length
    public fun emit_var_slice(encoder: &mut Encoder, v: vector<u8>) {
        let size = vector::length(&v);
        emit_var_int(encoder, (size as u64));
        vector::append(&mut encoder.data, v);
    }

    public fun peel_var_int(decoder: &mut Decoder): u64 {
        let n = peel_u8(decoder);
        if (n == 0xFF) {
            let x = peel_u64(decoder);
            assert!(x >= 0x100000000, ErrorNonMinimalVarInt);
            x
        } else if (n == 0xFE) {
            let x = peel_u32(decoder);
            assert!(x >= 0x10000, ErrorNonMinimalVarInt);
            (x as u64)
        } else if (n == 0xFD) {
            let x = peel_u16(decoder);
            assert!(x >= 0xFD, ErrorNonMinimalVarInt);
            (x as u64)
        } else {
            (n as u64)
        }
    }

    /// Peel a slice of bytes from the decoder with a given length
    fun peel_slice_with_length(decoder: &mut Decoder, length: u64): vector<u8> {
        let slice = vector::empty<u8>();
        let i = 0;
        while (i < length) {
            if (vector::length(&decoder.data) > 0) {
                vector::push_back(&mut slice, vector::pop_back(&mut decoder.data));
            } else {
                break
            };
            i = i + 1;
        };
        // No need to reverse the slice
        slice
    }

    /// Peel a slice of bytes from the decoder with a varint length
    public fun peel_var_slice(decoder: &mut Decoder): vector<u8> {
        let size = peel_var_int(decoder);
        peel_slice_with_length(decoder, size)
    }

    public fun peel_bool(decoder: &mut Decoder): bool {
        let v = vector::pop_back(&mut decoder.data);
        if (v == 0) {
            false
        } else {
            true
        }
    }

    public fun peel_u64(decoder: &mut Decoder): u64 {
        let v = 0u64;
        let i = 0;
        while (i < 8) {
            let byte = vector::pop_back(&mut decoder.data);
            v = v | ((byte as u64) << (i * 8));
            i = i + 1;
        };
        v
    }

    public fun peel_u32(decoder: &mut Decoder): u32 {
        let v = 0u32;
        let i = 0;
        while (i < 4) {
            let byte = vector::pop_back(&mut decoder.data);
            v = v | ((byte as u32) << (i * 8));
            i = i + 1;
        };
        v
    }

    public fun peel_u16(decoder: &mut Decoder): u16 {
        let v = 0u16;
        let i = 0;
        while (i < 2) {
            let byte = vector::pop_back(&mut decoder.data);
            v = v | ((byte as u16) << (i * 8));
            i = i + 1;
        };
        v
    }

    public fun peel_u8(decoder: &mut Decoder): u8 {
        vector::pop_back(&mut decoder.data)
    }

    #[test]
    fun test_u64() {
        let encoder = encoder();
        emit_u64(&mut encoder, 1234567890);
        //std::debug::print(&encoder.data);
        let decoder = decoder(unpack_encoder(encoder));
        assert!(peel_u64(&mut decoder) == 1234567890, 1000);
        let data = unpack_decoder(decoder);
        assert!(vector::length(&data) == 0, 1001);
    }

    #[test]
    fun test_u32() {
        let encoder = encoder();
        emit_u32(&mut encoder, 4294967295u32);
        //std::debug::print(&encoder.data);
        let decoder = decoder(unpack_encoder(encoder));
        assert!(peel_u32(&mut decoder) == 4294967295u32, 1000);
        let data = unpack_decoder(decoder);
        assert!(vector::length(&data) == 0, 1001);
    }

    #[test]
    fun test_var_int() {
        let encoder = encoder();
        emit_var_int(&mut encoder, 12345678900u64);
        //std::debug::print(&encoder.data);
        let decoder = decoder(unpack_encoder(encoder));
        assert!(peel_var_int(&mut decoder) == 12345678900u64, 1000);
        let data = unpack_decoder(decoder);
        assert!(vector::length(&data) == 0, 1001);
    }

    #[test]
    fun test_var_slice() {
        let encoder = encoder();
        let test_vector = vector[1u8, 2u8, 3u8, 4u8, 5u8];
        emit_var_slice(&mut encoder, test_vector);
        
        let decoder = decoder(unpack_encoder(encoder));
        
        let decoded_vector = peel_var_slice(&mut decoder);
        
        assert!(decoded_vector == test_vector, 1000);
        let remaining_data = unpack_decoder(decoder);
        assert!(vector::length(&remaining_data) == 0, 1001);
    }

    #[test]
    fun test_encode_decode() {
        let encoder = encoder();
        emit_u64(&mut encoder, 12345678900u64);
        emit_u32(&mut encoder, 4294967295u32);
        emit_u16(&mut encoder, 65535u16);
        emit_u8(&mut encoder, 255u8);
        emit_var_int(&mut encoder, 12345678900u64);
        emit_var_slice(&mut encoder, vector[1u8, 2u8, 3u8, 4u8, 5u8]);

        let decoder = decoder(unpack_encoder(encoder));

        assert!(peel_u64(&mut decoder) == 12345678900u64, 1001);
        assert!(peel_u32(&mut decoder) == 4294967295u32, 1002);
        assert!(peel_u16(&mut decoder) == 65535u16, 1003);
        assert!(peel_u8(&mut decoder) == 255u8, 1004);
        assert!(peel_var_int(&mut decoder) == 12345678900u64, 1005);
        assert!(peel_var_slice(&mut decoder) == vector[1u8, 2u8, 3u8, 4u8, 5u8], 1006);
        let data = unpack_decoder(decoder);
        assert!(vector::length(&data) == 0, 1007);
    }

    #[test]
    fun test_var_int_encoding() {
        let encoder = encoder();

        // Test case 1: VarInt(10)
        emit_var_int(&mut encoder, 10);
        assert!(unpack_encoder(encoder) == vector[10u8], 1001);

        // Test case 2: VarInt(0xFC)
        encoder = encoder();
        emit_var_int(&mut encoder, 0xFC);
        assert!(unpack_encoder(encoder) == vector[0xFCu8], 1002);

        // Test case 3: VarInt(0xFD)
        encoder = encoder();
        emit_var_int(&mut encoder, 0xFD);
        assert!(unpack_encoder(encoder) == vector[0xFDu8, 0xFD, 0], 1003);

        // Test case 4: VarInt(0xFFF)
        encoder = encoder();
        emit_var_int(&mut encoder, 0xFFF);
        assert!(unpack_encoder(encoder) == vector[0xFDu8, 0xFF, 0xF], 1004);

        // Test case 5: VarInt(0xF0F0F0F)
        encoder = encoder();
        emit_var_int(&mut encoder, 0xF0F0F0F);
        assert!(unpack_encoder(encoder) == vector[0xFEu8, 0xF, 0xF, 0xF, 0xF], 1005);

        // Test case 6: VarInt(0xF0F0F0F0F0E0)
        encoder = encoder();
        emit_var_int(&mut encoder, 0xF0F0F0F0F0E0);
        assert!(unpack_encoder(encoder) == vector[0xFFu8, 0xE0, 0xF0, 0xF0, 0xF0, 0xF0, 0xF0, 0, 0], 1006);
    }

    #[test]
    fun test_var_int_decoding() {
        // Test case 7: VarInt(0x100000000)
        let decoder = decoder(vector[0xFF, 0, 0, 0, 0, 1, 0, 0, 0]);
        assert!(peel_var_int(&mut decoder) == 0x100000000, 1007);

        // Test case 8: VarInt(0x10000)
        decoder = decoder(vector[0xFE, 0, 0, 1, 0]);
        assert!(peel_var_int(&mut decoder) == 0x10000, 1008);
    }

    #[test]
    fun test_var_int_decoding_fd() {
        // Test case VarInt(0xFD)
        let decoder = decoder(vector[0xFD, 0xFD, 0]);
        let result = peel_var_int(&mut decoder);
        //std::debug::print(&result);
        assert!(result == 0xFD, 1009);
    }

}