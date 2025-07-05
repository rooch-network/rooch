#!/usr/bin/env python3
# Copyright (c) Rooch Network
# SPDX-License-Identifier: Apache-2.0

import re
from typing import Optional, Union
from bech32 import bech32_decode, bech32_encode, convertbits

from ..utils.hex import ensure_hex_prefix, is_hex_string, strip_hex_prefix, to_hex
from ..utils.bytes import to_bytes
# Import BCS classes and protocols
from ..bcs.serializer import BcsSerializer, Serializable, BcsDeserializer, Deserializable


class RoochAddress(Serializable, Deserializable):
    """Class for handling Rooch addresses, implementing BCS protocols."""
    ADDRESS_REGEX = re.compile(r"^(0x)?[a-fA-F0-9]{1,64}$")
    ADDRESS_LENGTH = 32 # Address length in bytes
    BECH32_HRP = "rooch"
    BECH32_LENGTH = 64

    def __init__(self, addr: bytes):
        """
        Args:
            addr: Address as bytes (must be 32 bytes)
        Raises:
            ValueError: If the address is invalid
        """
        if not isinstance(addr, (bytes, bytearray)):
            raise TypeError("RoochAddress constructor only accepts bytes")
        if len(addr) != self.ADDRESS_LENGTH:
            raise ValueError(f"Address must be {self.ADDRESS_LENGTH} bytes")
        self._bytes = bytes(addr)

    @classmethod
    def from_str(cls, addr_str: str) -> 'RoochAddress':
        """Parse a Rooch address from a string (0x-prefixed hex, bare 64-char hex, or bech32)."""
        addr_str = addr_str.strip()
        if addr_str.startswith('0x') or (is_hex_string(addr_str) and len(addr_str) == 64):
            return cls.from_hex(addr_str)
        elif addr_str.startswith(cls.BECH32_HRP):
            return cls.from_bech32(addr_str)
        else:
            raise ValueError(f"Unsupported Rooch address format: {addr_str}")

    @staticmethod
    def from_hex(hex_str: str) -> 'RoochAddress':
        """Create from hex string (0x-prefixed or bare 64-char hex). Does NOT handle bech32."""
        hex_str = hex_str.lower().strip()
        if hex_str.startswith('0x'):
            hex_part = hex_str[2:]
            if not is_hex_string(hex_part) or len(hex_part) > 64:
                raise ValueError(f"Invalid hex address: {hex_str}")
            padded_hex = hex_part.rjust(64, '0')
            return RoochAddress(bytes.fromhex(padded_hex))
        elif is_hex_string(hex_str) and len(hex_str) == 64:
            return RoochAddress(bytes.fromhex(hex_str))
        else:
            raise ValueError(f"Unsupported hex address format: {hex_str}")

    @classmethod
    def from_bech32(cls, bech32_addr: str) -> 'RoochAddress':
        hrp, data = bech32_decode(bech32_addr)
        if hrp != cls.BECH32_HRP:
            raise ValueError(f"Invalid bech32 hrp: {hrp}")
        decoded = convertbits(data, 5, 8, False)
        if decoded is None or len(decoded) != cls.ADDRESS_LENGTH:
            raise ValueError(f"Invalid bech32 data length: {len(decoded) if decoded else 0}")
        return cls(bytes(decoded))

    @classmethod
    def from_hex_literal(cls, literal: str) -> 'RoochAddress':
        """Create a RoochAddress from a hex literal (e.g., "0x1", "0xabc")."""
        if not isinstance(literal, str) or not literal.startswith("0x"):
            raise ValueError('Hex literal must start with "0x"')
        hex_part = literal[2:]
        if not is_hex_string(hex_part):
            raise ValueError(f"Invalid characters in hex literal: {literal}")
        hex_len = len(hex_part)
        expected_hex_len = 64
        if hex_len == 0:
            raise ValueError("Hex literal cannot be empty (0x)")
        if hex_len > expected_hex_len:
            raise ValueError(f"Hex literal too long ({hex_len} chars > {expected_hex_len}): {literal}")
        padded_hex = '0' * (expected_hex_len - hex_len) + hex_part
        return cls(bytes.fromhex(padded_hex))

    @classmethod
    def is_valid_address(cls, address: str) -> bool:
        if not isinstance(address, str):
            return False
        address = address.strip().lower()
        if address.startswith("0x"):
            hex_part = address[2:]
            return is_hex_string(hex_part) and len(hex_part) == 64
        elif address.startswith(cls.BECH32_HRP):
            try:
                cls.from_bech32(address)
                return True
            except Exception:
                return False
        elif is_hex_string(address) and len(address) == 64:
            return True
        return False

    @classmethod
    def validate_address(cls, address: str) -> bool:
        return cls.is_valid_address(address)

    @classmethod
    def normalize_address(cls, address: str) -> str:
        address = address.strip().lower()
        if address.startswith("0x"):
            hex_part = address[2:]
            if not is_hex_string(hex_part) or len(hex_part) != 64:
                raise ValueError(f"Invalid Rooch address: {address}")
            padded_hex = hex_part.rjust(64, '0')
            return "0x" + padded_hex
        elif address.startswith(cls.BECH32_HRP):
            return cls.from_bech32(address).to_hex()
        elif is_hex_string(address) and len(address) == 64:
            return "0x" + address
        else:
            raise ValueError(f"Invalid Rooch address: {address}")

    # --- BCS Implementation ---
    def serialize(self, serializer: BcsSerializer):
        """Serialize the address as fixed 32 bytes."""
        serializer.fixed_bytes(self.to_bytes())
        
    @staticmethod
    def deserialize(deserializer: BcsDeserializer) -> 'RoochAddress':
        """Deserialize the address by reading fixed 32 bytes."""
        addr_bytes = deserializer.fixed_bytes(RoochAddress.ADDRESS_LENGTH)
        return RoochAddress(addr_bytes)
    # --- End BCS Implementation ---

    def __str__(self) -> str:
        """Convert to 0x-prefixed hex string"""
        return self.to_hex_full()

    def __repr__(self) -> str:
        return f"RoochAddress({str(self)})"
    
    def __eq__(self, other: object) -> bool:
        if not isinstance(other, RoochAddress):
            return NotImplemented
        return self._bytes == other._bytes
    
    def __hash__(self) -> int:
        return hash(self._bytes)
    
    def to_bytes(self) -> bytes:
        """Convert the address to bytes
        
        Returns:
            Address as bytes
        """
        return self._bytes
    
    def to_hex(self) -> str:
        """Convert the address to a 64-char hex string without 0x prefix"""
        return self._bytes.hex()

    def to_hex_full(self) -> str:
        """Convert the address to a 64-char hex string with 0x prefix"""
        return "0x" + self._bytes.hex()

    def to_hex_literal(self) -> str:
        """Convert the address to a short hex string with 0x prefix (no leading zeros)"""
        hex_str = self._bytes.hex().lstrip('0')
        if not hex_str:
            hex_str = '0'
        return "0x" + hex_str


def is_rooch_address(value: str) -> bool:
    """Check if a string is a valid Rooch address
    
    Args:
        value: String to check
        
    Returns:
        True if the string is a valid Rooch address
    """
    return RoochAddress.is_valid_address(value)


def normalize_rooch_address(address: str) -> str:
    """Normalize a Rooch address (lowercase with 0x prefix)
    
    Args:
        address: Address to normalize
        
    Returns:
        Normalized address
        
    Raises:
        ValueError: If the address is invalid
    """
    if not is_rooch_address(address):
        raise ValueError(f"Invalid Rooch address: {address}")
    
    return ensure_hex_prefix(address.lower())


def decode_to_rooch_address_str(address: Union[str, RoochAddress]) -> str:
    """Convert an address to a Rooch address string
    
    Args:
        address: Address as string or RoochAddress
        
    Returns:
        Rooch address string
    """
    if isinstance(address, RoochAddress):
        return address.to_hex()
    elif isinstance(address, str):
        return normalize_rooch_address(address)
    else:
        raise TypeError("Address must be a string or RoochAddress")


def decode_to_package_address_str(package_address: Union[str, RoochAddress]) -> str:
    """Convert a package address to a hex string (64-char, no 0x prefix)"""
    if isinstance(package_address, RoochAddress):
        return package_address.to_hex()
    elif isinstance(package_address, str):
        if package_address.startswith("0x"):
            return package_address[2:]
        return package_address
    else:
        raise TypeError("Package address must be a string or RoochAddress")