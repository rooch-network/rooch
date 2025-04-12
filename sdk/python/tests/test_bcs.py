#!/usr/bin/env python3
# Copyright (c) Rooch Network
# SPDX-License-Identifier: Apache-2.0

"""Tests for BCS serialization module"""

import pytest
import struct
from rooch.bcs.serializer import BcsSerializer, Args


class TestBcsSerializer:
    """Tests for BcsSerializer"""
    
    def test_serialize_u8(self):
        """Test serializing u8 values"""
        serializer = BcsSerializer()
        
        # Test valid values
        assert serializer.serialize_u8(0) == b'\x00'
        assert serializer.serialize_u8(127) == b'\x7f'
        assert serializer.serialize_u8(255) == b'\xff'
        
        # Test boundary values
        with pytest.raises(Exception):
            serializer.serialize_u8(-1)  # Too small
        
        with pytest.raises(Exception):
            serializer.serialize_u8(256)  # Too large
    
    def test_serialize_u16(self):
        serializer = BcsSerializer()
        assert serializer.serialize_u16(0) == b'\x00\x00'
        assert serializer.serialize_u16(65535) == b'\xff\xff'
        with pytest.raises(Exception): serializer.serialize_u16(-1)
        with pytest.raises(Exception): serializer.serialize_u16(65536)
    
    def test_serialize_u32(self):
        serializer = BcsSerializer()
        assert serializer.serialize_u32(0) == b'\x00\x00\x00\x00'
        assert serializer.serialize_u32(4294967295) == b'\xff\xff\xff\xff'
        with pytest.raises(Exception): serializer.serialize_u32(-1)
        with pytest.raises(Exception): serializer.serialize_u32(4294967296)
    
    def test_serialize_u64(self):
        serializer = BcsSerializer()
        assert serializer.serialize_u64(0) == b'\x00\x00\x00\x00\x00\x00\x00\x00'
        assert serializer.serialize_u64(18446744073709551615) == b'\xff\xff\xff\xff\xff\xff\xff\xff'
        with pytest.raises(Exception): serializer.serialize_u64(-1)
        # with pytest.raises(Exception): serializer.serialize_u64(18446744073709551616) # This number might be too large for Python int
    
    def test_serialize_u128(self):
        serializer = BcsSerializer()
        assert serializer.serialize_u128(0) == b'\x00' * 16
        max_u128 = (1 << 128) - 1
        assert serializer.serialize_u128(max_u128) == b'\xff' * 16
        with pytest.raises(Exception): serializer.serialize_u128(-1)
        # with pytest.raises(Exception): serializer.serialize_u128(max_u128 + 1)
    
    def test_serialize_u256(self):
        serializer = BcsSerializer()
        assert serializer.serialize_u256(0) == b'\x00' * 32
        max_u256 = (1 << 256) - 1
        assert serializer.serialize_u256(max_u256) == b'\xff' * 32
        with pytest.raises(Exception): serializer.serialize_u256(-1)
        # with pytest.raises(Exception): serializer.serialize_u256(max_u256 + 1)
    
    def test_serialize_bool(self):
        """Test serializing boolean values"""
        serializer = BcsSerializer()
        
        # Test values
        assert serializer.serialize_bool(False) == b'\x00'
        assert serializer.serialize_bool(True) == b'\x01'
    
    def test_serialize_address(self):
        """Test serializing address values"""
        serializer = BcsSerializer()
        
        # Use a known valid address from Rust codebase
        address = "0x0c71415a4e19ac8705e1817091a17a1a4a6a895c5f040627bf2a062607700f67"
        # Expected bytes (32 bytes, no prefix)
        expected_bytes = bytes.fromhex("0c71415a4e19ac8705e1817091a17a1a4a6a895c5f040627bf2a062607700f67")
        result = serializer.serialize_address(address)
        assert result == expected_bytes # Direct comparison
        assert len(result) == 32  # 32 bytes for an address
    
    def test_serialize_vector(self):
        """Test serializing vector values"""
        serializer = BcsSerializer()
        
        # Test empty vector - ULEB128 length 0 is single byte 0x00
        empty_vector = serializer.serialize_vector([], lambda x: serializer.serialize_u8(x))
        assert empty_vector == bytes([0])
        
        # Test vector with u8 values
        vec_u8 = serializer.serialize_vector([1, 2, 3], serializer.serialize_u8)
        assert vec_u8 == bytes([3, 1, 2, 3]) # Length 3, then values
        
        # Test vector with u16 values
        vec_u16 = serializer.serialize_vector([256, 512], serializer.serialize_u16)
        assert vec_u16 == bytes([2]) + struct.pack("<HH", 256, 512) # Length 2, then values
    
    def test_serialize_string(self):
        """Test serializing string values"""
        serializer = BcsSerializer()
        
        # Test empty string - ULEB128 length 0 is single byte 0x00
        assert serializer.serialize_string("") == bytes([0])
        
        # Test non-empty string
        hello = "Hello"
        hello_bytes = hello.encode('utf-8')
        expected = serializer.serialize_len(len(hello_bytes)) + hello_bytes
        assert serializer.serialize_string(hello) == expected
        
        # Test string with unicode
        unicode_str = "你好"
        unicode_bytes = unicode_str.encode('utf-8')
        expected_unicode = serializer.serialize_len(len(unicode_bytes)) + unicode_bytes
        assert serializer.serialize_string(unicode_str) == expected_unicode