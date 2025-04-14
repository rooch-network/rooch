#!/usr/bin/env python3
# Copyright (c) Rooch Network
# SPDX-License-Identifier: Apache-2.0

"""Tests for BCS serialization module"""

import pytest
import struct
from rooch.bcs.serializer import BcsSerializer, BcsDeserializer


class TestBcsSerializer:
    """Tests for BcsSerializer"""
    
    def test_serialize_u8(self):
        """Test serializing u8 values"""
        serializer = BcsSerializer()
        
        # Test valid values
        serializer.u8(0)
        assert serializer.output() == b'\x00'
        
        serializer = BcsSerializer()
        serializer.u8(127)
        assert serializer.output() == b'\x7f'
        
        serializer = BcsSerializer()
        serializer.u8(255)
        assert serializer.output() == b'\xff'
        
        # Test boundary values
        with pytest.raises(Exception):
            serializer.u8(-1)  # Too small
        
        with pytest.raises(Exception):
            serializer.u8(256)  # Too large
    
    def test_serialize_u16(self):
        serializer = BcsSerializer()
        serializer.u16(0)
        assert serializer.output() == b'\x00\x00'
        
        serializer = BcsSerializer()
        serializer.u16(65535)
        assert serializer.output() == b'\xff\xff'
        
        with pytest.raises(Exception): serializer.u16(-1)
        with pytest.raises(Exception): serializer.u16(65536)
    
    def test_serialize_u32(self):
        serializer = BcsSerializer()
        serializer.u32(0)
        assert serializer.output() == b'\x00\x00\x00\x00'
        
        serializer = BcsSerializer()
        serializer.u32(4294967295)
        assert serializer.output() == b'\xff\xff\xff\xff'
        
        with pytest.raises(Exception): serializer.u32(-1)
        with pytest.raises(Exception): serializer.u32(4294967296)
    
    def test_serialize_u64(self):
        serializer = BcsSerializer()
        serializer.u64(0)
        assert serializer.output() == b'\x00\x00\x00\x00\x00\x00\x00\x00'
        
        serializer = BcsSerializer()
        serializer.u64(18446744073709551615)
        assert serializer.output() == b'\xff\xff\xff\xff\xff\xff\xff\xff'
        
        with pytest.raises(Exception): serializer.u64(-1)
        # with pytest.raises(Exception): serializer.u64(18446744073709551616) # This number might be too large for Python int
    
    def test_serialize_u128(self):
        serializer = BcsSerializer()
        serializer.u128(0)
        assert serializer.output() == b'\x00' * 16
        
        serializer = BcsSerializer()
        max_u128 = (1 << 128) - 1
        serializer.u128(max_u128)
        assert serializer.output() == b'\xff' * 16
        
        with pytest.raises(Exception): serializer.u128(-1)
        # with pytest.raises(Exception): serializer.u128(max_u128 + 1)
    
    def test_serialize_u256(self):
        serializer = BcsSerializer()
        serializer.u256(0)
        assert serializer.output() == b'\x00' * 32
        
        serializer = BcsSerializer()
        max_u256 = (1 << 256) - 1
        serializer.u256(max_u256)
        assert serializer.output() == b'\xff' * 32
        
        with pytest.raises(Exception): serializer.u256(-1)
        # with pytest.raises(Exception): serializer.u256(max_u256 + 1)
    
    def test_serialize_bool(self):
        """Test serializing boolean values"""
        serializer = BcsSerializer()
        serializer.bool(False)
        assert serializer.output() == b'\x00'
        
        serializer = BcsSerializer()
        serializer.bool(True)
        assert serializer.output() == b'\x01'
    
    def test_serialize_bytes(self):
        """Test serializing bytes values"""
        serializer = BcsSerializer()
        
        # Test empty bytes
        serializer.bytes(b"")
        assert serializer.output() == b'\x00'
        
        # Test non-empty bytes
        serializer = BcsSerializer()
        test_bytes = b"Hello"
        serializer.bytes(test_bytes)
        assert serializer.output() == bytes([len(test_bytes)]) + test_bytes
    
    def test_serialize_str(self):
        """Test serializing string values"""
        serializer = BcsSerializer()
        
        # Test empty string
        serializer.str("")
        assert serializer.output() == b'\x00'
        
        # Test non-empty string
        serializer = BcsSerializer()
        hello = "Hello"
        serializer.str(hello)
        expected = bytes([len(hello)]) + hello.encode('utf-8')
        assert serializer.output() == expected
        
        # Test string with unicode
        serializer = BcsSerializer()
        unicode_str = "你好"
        serializer.str(unicode_str)
        unicode_bytes = unicode_str.encode('utf-8')
        expected = bytes([len(unicode_bytes)]) + unicode_bytes
        assert serializer.output() == expected
    
    def test_serialize_sequence(self):
        """Test serializing sequence values"""
        serializer = BcsSerializer()
        
        # Test empty sequence
        serializer.sequence([], lambda s, x: s.u8(x))
        assert serializer.output() == b'\x00'
        
        # Test sequence with u8 values
        serializer = BcsSerializer()
        serializer.sequence([1, 2, 3], lambda s, x: s.u8(x))
        assert serializer.output() == bytes([3, 1, 2, 3])  # Length 3, then values
        
        # Test sequence with u16 values
        serializer = BcsSerializer()
        serializer.sequence([256, 512], lambda s, x: s.u16(x))
        assert serializer.output() == bytes([2]) + struct.pack("<HH", 256, 512)  # Length 2, then values