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

    def __init__(self, addr: Union[str, bytes]):
        """
        Args:
            addr: Address in hex string (with or without 0x prefix) or bytes
            
        Raises:
            ValueError: If the address is invalid
        """
        if isinstance(addr, str):
            addr = addr.strip()
            if addr.startswith("0x"):
                # Support short hex, pad left
                hex_part = addr[2:]
                if not is_hex_string(hex_part):
                    raise ValueError(f"Invalid hex characters in address: {addr}")
                if len(hex_part) > self.ADDRESS_LENGTH * 2:
                    raise ValueError(f"Hex address too long: {addr}")
                padded_hex = hex_part.rjust(self.ADDRESS_LENGTH * 2, '0')
                self._bytes = bytes.fromhex(padded_hex)
            elif addr.startswith(self.BECH32_HRP):
                self._bytes = self.from_bech32(addr)._bytes
            else:
                raise ValueError(f"Unsupported Rooch address format: {addr}")
        else:
            if len(addr) != self.ADDRESS_LENGTH:
                raise ValueError(f"Address must be {self.ADDRESS_LENGTH} bytes")
            self._bytes = addr
            
        if len(self._bytes) != self.ADDRESS_LENGTH:
            raise ValueError(f"Address must be {self.ADDRESS_LENGTH} bytes")
    
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

    @classmethod
    def from_bech32(cls, bech32_addr: str) -> 'RoochAddress':
        hrp, data = bech32_decode(bech32_addr)
        if hrp != cls.BECH32_HRP:
            raise ValueError(f"Invalid bech32 hrp: {hrp}")
        decoded = convertbits(data, 5, 8, False)
        if decoded is None or len(decoded) != cls.ADDRESS_LENGTH:
            raise ValueError(f"Invalid bech32 data length: {len(decoded) if decoded else 0}")
        return cls(bytes(decoded))

    def to_bech32(self) -> str:
        data = list(self._bytes)
        bech32_data = convertbits(data, 8, 5)
        return bech32_encode(self.BECH32_HRP, bech32_data)

    @staticmethod
    def is_valid_address(address: str) -> bool:
        """Check if a string is a valid Rooch address
        
        Args:
            address: Address string to validate
            
        Returns:
            True if the address is valid
        """
        if not isinstance(address, str):
            return False
        
        address = address.strip()
        if address.startswith("0x"):
            hex_part = address[2:]
            return is_hex_string(hex_part) and len(hex_part) <= 64
        elif address.startswith(RoochAddress.BECH32_HRP):
            try:
                RoochAddress.from_bech32(address)
                return True
            except Exception:
                return False
        return False
    
    @staticmethod
    def validate_address(address: str) -> bool:
        """Check if a string is a valid Rooch address (alias for is_valid_address)
        
        Args:
            address: Address string to validate
            
        Returns:
            True if the address is valid
        """
        return RoochAddress.is_valid_address(address)
    
    @staticmethod
    def normalize_address(address: str) -> str:
        """Normalize a Rooch address (lowercase with 0x prefix)
        
        Args:
            address: Address to normalize
            
        Returns:
            Normalized address
            
        Raises:
            ValueError: If the address is invalid
        """
        if address.startswith("0x"):
            hex_part = address[2:]
            padded_hex = hex_part.rjust(64, '0')
            return "0x" + padded_hex.lower()
        elif address.startswith(RoochAddress.BECH32_HRP):
            return RoochAddress.from_bech32(address).to_hex()
        else:
            raise ValueError(f"Invalid Rooch address: {address}")
    
    @staticmethod
    def from_hex(hex_str: str) -> 'RoochAddress':
        """Create from hex string
        
        Args:
            hex_str: Hex string with or without 0x prefix
            
        Returns:
            RoochAddress instance
        """
        # Convert to lowercase before creating address
        return RoochAddress(hex_str.lower())
    
    @staticmethod
    def from_hex_literal(literal: str) -> 'RoochAddress':
        """Create a RoochAddress from a hex literal (e.g., "0x1", "0xabc").
        Handles padding short addresses to the full 32-byte length.

        Args:
            literal: Hex literal string, must start with "0x"

        Returns:
            RoochAddress instance

        Raises:
            ValueError: If the literal is invalid or too long
        """
        if not isinstance(literal, str) or not literal.startswith("0x"):
            raise ValueError('Hex literal must start with "0x"')

        # Ensure no non-hex characters after 0x
        hex_part = literal[2:]
        if not is_hex_string(hex_part):
             raise ValueError(f"Invalid characters in hex literal: {literal}")
             
        hex_len = len(hex_part)
        expected_hex_len = 64 # 32 bytes * 2 hex chars/byte

        if hex_len == 0:
             raise ValueError("Hex literal cannot be empty (0x)")
             
        if hex_len > expected_hex_len:
            raise ValueError(
                f"Hex literal too long ({hex_len} chars > {expected_hex_len}): {literal}"
            )

        # Pad with leading zeros if shorter than expected length
        if hex_len < expected_hex_len:
            padded_hex = '0' * (expected_hex_len - hex_len) + hex_part
        else:
            padded_hex = hex_part

        # Call the existing from_hex method with the full-length hex string
        return RoochAddress.from_hex(f"0x{padded_hex}")
    
    def __str__(self) -> str:
        """Convert to 0x-prefixed hex string"""
        return self.to_hex()

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
        """Convert the address to hex string with 0x prefix
        
        Returns:
            Address as hex string with full length (64 characters + 0x prefix)
        """
        return "0x" + self._bytes.hex()
    
    def to_hex_full(self) -> str:
        """Convert the address to hex string with 0x prefix and full length
        
        Returns:
            Address as hex string with full length (64 characters + 0x prefix)
        """
        return "0x" + self._bytes.hex()
    
    def to_hex_literal(self) -> str:
        """Convert the address to short hex string with 0x prefix
        
        Returns:
            Address as short hex string (without leading zeros)
        """
        hex_str = self._bytes.hex().lstrip('0')
        if not hex_str:
            hex_str = '0'
        return "0x" + hex_str
    
    def to_hex_no_prefix(self) -> str:
        """Convert the address to hex string without 0x prefix
        
        Returns:
            Address as hex string without 0x prefix
        """
        return self._bytes.hex()

    def to_bech32_address(self) -> str:
        """Convert the address to bech32 format (rooch1...) with checksum
        
        Returns:
            Address as bech32 string
        """
        return self.to_bech32()


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
    """Convert a package address to a hex string
    
    Args:
        package_address: Package address as string or RoochAddress
        
    Returns:
        Package address as hex string without 0x prefix
    """
    if isinstance(package_address, RoochAddress):
        return package_address.to_hex_no_prefix()
    elif isinstance(package_address, str):
        if package_address.startswith("0x"):
            return package_address[2:]
        return package_address
    else:
        raise TypeError("Package address must be a string or RoochAddress")