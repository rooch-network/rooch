#!/usr/bin/env python3
# Copyright (c) Rooch Network
# SPDX-License-Identifier: Apache-2.0

"""Tests for BCS serialization module"""

import pytest
from rooch.bcs.serializer import BcsSerializer, Args


class TestBcsSerializer:
    """Tests for BcsSerializer"""
    
    def test_serialize_u8(self):
        """Test serializing u8 values"""
        serializer = BcsSerializer()
        
        # Test valid values
        assert serializer.serialize_u8(0) == bytes([0])
        assert serializer.serialize_u8(255) == bytes([255])
        
        # Test boundary values
        with pytest.raises(Exception):
            serializer.serialize_u8(-1)  # Too small
        
        with pytest.raises(Exception):
            serializer.serialize_u8(256)  # Too large
    
    def test_serialize_u64(self):
        """Test serializing u64 values"""
        serializer = BcsSerializer()
        
        # Test values
        assert serializer.serialize_u64(0) == bytes([0, 0, 0, 0, 0, 0, 0, 0])
        assert serializer.serialize_u64(1) == bytes([1, 0, 0, 0, 0, 0, 0, 0])
        assert serializer.serialize_u64(0xFFFFFFFFFFFFFFFF) == bytes([0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF])
    
    def test_serialize_u128(self):
        """Test serializing u128 values"""
        serializer = BcsSerializer()
        
        # Test values
        assert serializer.serialize_u128(0) == bytes([0] * 16)
        assert serializer.serialize_u128(1) == bytes([1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0])
    
    def test_serialize_bool(self):
        """Test serializing boolean values"""
        serializer = BcsSerializer()
        
        # Test values
        assert serializer.serialize_bool(True) == bytes([1])
        assert serializer.serialize_bool(False) == bytes([0])
    
    def test_serialize_address(self):
        """Test serializing address values"""
        serializer = BcsSerializer()
        
        # Test values
        address = "0x123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef"
        result = serializer.serialize_address(address)
        assert len(result) == 32  # 32 bytes for an address
        
        # Test invalid address length
        with pytest.raises(Exception):
            serializer.serialize_address("0x1234")
    
    def test_serialize_vector(self):
        """Test serializing vector values"""
        serializer = BcsSerializer()
        
        # Test empty vector
        empty_vector = serializer.serialize_vector([], lambda x: serializer.serialize_u8(x))
        assert empty_vector == bytes([0, 0, 0, 0])  # Length prefix 0
        
        # Test vector of u8
        u8_vector = serializer.serialize_vector([1, 2, 3], lambda x: serializer.serialize_u8(x))
        assert u8_vector == bytes([3, 0, 0, 0, 1, 2, 3])  # Length prefix 3 + data
    
    def test_serialize_string(self):
        """Test serializing string values"""
        serializer = BcsSerializer()
        
        # Test empty string
        assert serializer.serialize_string("") == bytes([0, 0, 0, 0])
        
        # Test ASCII string
        hello = serializer.serialize_string("Hello")
        assert hello == bytes([5, 0, 0, 0]) + b"Hello"
        
        # Test UTF-8 string
        utf8 = serializer.serialize_string("你好")
        assert len(utf8) == 4 + len("你好".encode("utf-8"))


class TestArgs:
    """Tests for Args utility"""
    
    def test_encode_bool(self):
        """Test encoding boolean values"""
        # Test true/false
        assert Args.encode_bool(True) == "true"
        assert Args.encode_bool(False) == "false"
    
    def test_encode_number(self):
        """Test encoding number values"""
        # Test various numbers
        assert Args.encode_number(0) == "0"
        assert Args.encode_number(123) == "123"
        assert Args.encode_number(-123) == "-123"
        assert Args.encode_number(123.45) == "123.45"
    
    def test_encode_hex_string(self):
        """Test encoding hex string values"""
        # Test with/without 0x prefix
        assert Args.encode_hex_string("0x1234") == "0x1234"
        assert Args.encode_hex_string("1234") == "0x1234"
        
        # Test capitalization
        assert Args.encode_hex_string("0xAbCd") == "0xabcd"
    
    def test_encode_string(self):
        """Test encoding string values"""
        # Test regular strings
        assert Args.encode_string("Hello") == "Hello"
        assert Args.encode_string("Special: \n\t\"'") == "Special: \n\t\"'"
    
    def test_encode(self):
        """Test general encode function"""
        # Test encoding different types
        assert Args.encode(True) == "true"
        assert Args.encode(123) == "123"
        assert Args.encode("0x1234") == "0x1234"
        assert Args.encode("Hello") == "Hello"
        
        # Test list encoding
        assert Args.encode([1, 2, 3]) == ["1", "2", "3"]
        assert Args.encode([True, False]) == ["true", "false"]