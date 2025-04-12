#!/usr/bin/env python3
# Copyright (c) Rooch Network
# SPDX-License-Identifier: Apache-2.0

"""Tests for address module"""

import pytest
from rooch.address.rooch import RoochAddress
from rooch.address.bitcoin import BitcoinAddress


class TestRoochAddress:
    """Tests for RoochAddress class"""
    
    # Use a known valid address from Rust codebase
    VALID_ADDRESS_STR_PREFIX = "0x0c71415a4e19ac8705e1817091a17a1a4a6a895c5f040627bf2a062607700f67"
    VALID_ADDRESS_STR_NOPREFIX = "0c71415a4e19ac8705e1817091a17a1a4a6a895c5f040627bf2a062607700f67"
    INVALID_ADDRESS_SHORT = "0x1234"
    # Ensure LONG address is actually longer than 64 hex chars
    INVALID_ADDRESS_LONG = VALID_ADDRESS_STR_PREFIX + "00"
    # Ensure invalid chars address contains non-hex chars
    INVALID_ADDRESS_CHARS = "0x0c71415a4e19ac8705e1817091a17a1a4a6a895c5f040627bf2a062607700f6g" # 'g' is invalid

    def test_validate_address(self):
        """Test address validation"""
        # Valid address
        assert RoochAddress.validate_address(self.VALID_ADDRESS_STR_PREFIX) is True
        assert RoochAddress.validate_address(self.VALID_ADDRESS_STR_NOPREFIX) is True
        assert RoochAddress.validate_address(self.VALID_ADDRESS_STR_PREFIX.upper()) is True
        
        # Invalid addresses
        assert RoochAddress.validate_address(self.INVALID_ADDRESS_SHORT) is False
        assert RoochAddress.validate_address(self.INVALID_ADDRESS_LONG) is False
        assert RoochAddress.validate_address(self.INVALID_ADDRESS_CHARS) is False
        assert RoochAddress.validate_address("0x") is False
        assert RoochAddress.validate_address("") is False
        assert RoochAddress.validate_address(None) is False
        assert RoochAddress.validate_address(123) is False

    def test_normalize_address(self):
        """Test address normalization"""
        # Standard format
        assert RoochAddress.normalize_address(self.VALID_ADDRESS_STR_PREFIX) == self.VALID_ADDRESS_STR_PREFIX
        
        # No prefix
        assert RoochAddress.normalize_address(self.VALID_ADDRESS_STR_NOPREFIX) == self.VALID_ADDRESS_STR_PREFIX
        
        # Uppercase
        assert RoochAddress.normalize_address(self.VALID_ADDRESS_STR_PREFIX.upper()) == self.VALID_ADDRESS_STR_PREFIX
        
        # Mixed case
        mixed_case = "0x1234ABCD" + "e" * 56
        assert RoochAddress.normalize_address(mixed_case) == "0x1234abcd" + "e" * 56
        
        # Invalid addresses
        with pytest.raises(ValueError): RoochAddress.normalize_address(self.INVALID_ADDRESS_SHORT)
        with pytest.raises(ValueError): RoochAddress.normalize_address(self.INVALID_ADDRESS_CHARS)
        with pytest.raises(ValueError): RoochAddress.normalize_address("")

    def test_from_hex(self):
        """Test creating address from hex string"""
        # Valid hex string with prefix
        address_prefix = RoochAddress.from_hex(self.VALID_ADDRESS_STR_PREFIX)
        assert str(address_prefix) == self.VALID_ADDRESS_STR_PREFIX
        
        # Valid hex string without prefix
        address_noprefix = RoochAddress.from_hex(self.VALID_ADDRESS_STR_NOPREFIX)
        assert str(address_noprefix) == self.VALID_ADDRESS_STR_PREFIX
        
        # Valid uppercase
        address_upper = RoochAddress.from_hex(self.VALID_ADDRESS_STR_PREFIX.upper())
        assert str(address_upper) == self.VALID_ADDRESS_STR_PREFIX

        # Invalid hex strings
        with pytest.raises(ValueError): RoochAddress.from_hex(self.INVALID_ADDRESS_SHORT)
        with pytest.raises(ValueError): RoochAddress.from_hex(self.INVALID_ADDRESS_LONG)
        with pytest.raises(ValueError): RoochAddress.from_hex(self.INVALID_ADDRESS_CHARS)
        with pytest.raises(ValueError): RoochAddress.from_hex("0x")
        with pytest.raises(ValueError): RoochAddress.from_hex("")

    def test_to_hex(self):
        """Test converting address to hex string"""
        address = RoochAddress.from_hex(self.VALID_ADDRESS_STR_PREFIX)
        assert address.to_hex() == self.VALID_ADDRESS_STR_PREFIX
        assert address.to_hex_no_prefix() == self.VALID_ADDRESS_STR_NOPREFIX

    def test_to_bytes(self):
        """Test converting address to bytes"""
        address = RoochAddress.from_hex(self.VALID_ADDRESS_STR_PREFIX)
        expected_bytes = bytes.fromhex(self.VALID_ADDRESS_STR_NOPREFIX)
        assert address.to_bytes() == expected_bytes
        assert len(address.to_bytes()) == 32

    def test_comparison(self):
        """Test address comparison"""
        addr1 = RoochAddress.from_hex(self.VALID_ADDRESS_STR_PREFIX)
        addr2 = RoochAddress.from_hex(self.VALID_ADDRESS_STR_NOPREFIX)
        addr3 = RoochAddress.from_hex("0x" + "0"*64)
        
        assert addr1 == addr2
        assert addr1 != addr3
        assert hash(addr1) == hash(addr2)
        assert hash(addr1) != hash(addr3)


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
        assert BitcoinAddress.validate_address(valid_mainnet, network="mainnet") is True
        
        # Valid testnet address - must specify network
        valid_testnet = "mipcBbFg9gMiCh81Kj8tqqdgoZub1ZJRfn"
        assert BitcoinAddress.validate_address(valid_testnet, network="testnet") is True
        
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