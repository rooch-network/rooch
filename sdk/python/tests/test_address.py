#!/usr/bin/env python3
# Copyright (c) Rooch Network
# SPDX-License-Identifier: Apache-2.0

"""Tests for address module"""

import pytest
from rooch.address.rooch import RoochAddress
from rooch.address.bitcoin import BitcoinAddress


class TestRoochAddress:
    """Tests for RoochAddress class"""
    
    def test_validate_address(self):
        """Test address validation"""
        # Valid address
        valid_addr = "0x123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef"
        assert RoochAddress.validate_address(valid_addr) is True
        
        # Invalid prefix
        invalid_prefix = "123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef"
        assert RoochAddress.validate_address(invalid_prefix) is False
        
        # Invalid length (too short)
        short_addr = "0x123456"
        assert RoochAddress.validate_address(short_addr) is False
        
        # Invalid length (too long)
        long_addr = "0x123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef00"
        assert RoochAddress.validate_address(long_addr) is False
        
        # Invalid characters
        invalid_chars = "0x123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdeg"
        assert RoochAddress.validate_address(invalid_chars) is False
    
    def test_normalize_address(self):
        """Test address normalization"""
        # Standard format
        std_addr = "0x123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef"
        assert RoochAddress.normalize_address(std_addr) == std_addr
        
        # Missing 0x prefix
        no_prefix = "123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef"
        assert RoochAddress.normalize_address(no_prefix) == "0x" + no_prefix
        
        # Mixed case
        mixed_case = "0x123456789AbCdEf0123456789aBcDeF0123456789abcdef0123456789abcdef"
        assert RoochAddress.normalize_address(mixed_case) == mixed_case.lower()
        
        # Both missing prefix and mixed case
        mixed_no_prefix = "123456789AbCdEf0123456789aBcDeF0123456789abcdef0123456789abcdef"
        expected = "0x" + mixed_no_prefix.lower()
        assert RoochAddress.normalize_address(mixed_no_prefix) == expected
        
        # Invalid address should raise exception
        with pytest.raises(Exception):
            RoochAddress.normalize_address("0x123")  # Too short
    
    def test_from_hex(self):
        """Test creating address from hex string"""
        # Valid hex string
        valid_hex = "123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef"
        address = RoochAddress.from_hex(valid_hex)
        assert address.to_hex() == "0x" + valid_hex
        
        # With 0x prefix
        with_prefix = "0x123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef"
        address = RoochAddress.from_hex(with_prefix)
        assert address.to_hex() == with_prefix
        
        # Invalid length
        with pytest.raises(Exception):
            RoochAddress.from_hex("0x123")
    
    def test_to_hex(self):
        """Test converting address to hex string"""
        # Create from hex
        hex_str = "0x123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef"
        address = RoochAddress.from_hex(hex_str)
        
        # Convert back to hex
        assert address.to_hex() == hex_str
    
    def test_to_bytes(self):
        """Test converting address to bytes"""
        # Create from hex
        hex_str = "0x123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef"
        address = RoochAddress.from_hex(hex_str)
        
        # Convert to bytes
        address_bytes = address.to_bytes()
        assert len(address_bytes) == 32  # 32 bytes
        assert address_bytes.hex() == hex_str[2:]  # excluding 0x prefix


class TestBitcoinAddress:
    """Tests for BitcoinAddress class"""
    
    def test_from_public_key_mainnet(self):
        """Test creating Bitcoin address from public key on mainnet"""
        # Example public key (compressed)
        pubkey_hex = "0279be667ef9dcbbac55a06295ce870b07029bfcdb2dce28d959f2815b16f81798"
        
        # Create address
        address = BitcoinAddress.from_public_key(bytes.fromhex(pubkey_hex), mainnet=True)
        
        # P2PKH address should start with "1"
        assert address.to_string().startswith("1")
    
    def test_from_public_key_testnet(self):
        """Test creating Bitcoin address from public key on testnet"""
        # Example public key (compressed)
        pubkey_hex = "0279be667ef9dcbbac55a06295ce870b07029bfcdb2dce28d959f2815b16f81798"
        
        # Create address
        address = BitcoinAddress.from_public_key(bytes.fromhex(pubkey_hex), mainnet=False)
        
        # Testnet P2PKH address should start with "m" or "n"
        assert address.to_string()[0] in ["m", "n"]
    
    def test_validate_address_p2pkh(self):
        """Test P2PKH address validation"""
        # Valid mainnet address
        valid_mainnet = "1BvBMSEYstWetqTFn5Au4m4GFg7xJaNVN2"
        assert BitcoinAddress.validate_address(valid_mainnet) is True
        
        # Valid testnet address
        valid_testnet = "mipcBbFg9gMiCh81Kj8tqqdgoZub1ZJRfn"
        assert BitcoinAddress.validate_address(valid_testnet) is True
        
        # Invalid address (wrong checksum)
        invalid_addr = "1BvBMSEYstWetqTFn5Au4m4GFg7xJaNVN3"
        assert BitcoinAddress.validate_address(invalid_addr) is False
        
        # Invalid address (wrong length)
        invalid_length = "1BvBMSEYstWetq"
        assert BitcoinAddress.validate_address(invalid_length) is False
    
    def test_to_string(self):
        """Test converting address to string"""
        # Example public key
        pubkey_hex = "0279be667ef9dcbbac55a06295ce870b07029bfcdb2dce28d959f2815b16f81798"
        
        # Create address
        address = BitcoinAddress.from_public_key(bytes.fromhex(pubkey_hex), mainnet=True)
        
        # Convert to string
        addr_str = address.to_string()
        
        # Should be valid
        assert BitcoinAddress.validate_address(addr_str) is True