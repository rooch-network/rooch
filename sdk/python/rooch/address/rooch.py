#!/usr/bin/env python3
# Copyright (c) Rooch Network
# SPDX-License-Identifier: Apache-2.0

import re
from typing import Optional, Union

from ..utils.hex import ensure_hex_prefix, is_hex_string, strip_hex_prefix, to_hex
from ..utils.bytes import to_bytes
# Import BCS classes and protocols
from ..bcs.serializer import BcsSerializer, Serializable, BcsDeserializer, Deserializable


class RoochAddress(Serializable, Deserializable):
    """Class for handling Rooch addresses, implementing BCS protocols."""
    
    # Rooch address regex pattern
    ADDRESS_REGEX = re.compile(r"^(0x)?[a-fA-F0-9]{64}$")
    ADDRESS_LENGTH = 32 # Address length in bytes
    
    def __init__(self, addr: Union[str, bytes]):
        """
        Args:
            addr: Address in hex string (with or without 0x prefix) or bytes
            
        Raises:
            ValueError: If the address is invalid
        """
        if isinstance(addr, str):
            # Remove 0x prefix if present
            addr = addr.removeprefix("0x")
            # Check length before converting to bytes
            if len(addr) != self.ADDRESS_LENGTH * 2:
                raise ValueError(f"Address must be {self.ADDRESS_LENGTH * 2} hex characters")
            try:
                self._bytes = bytes.fromhex(addr)
            except ValueError as e:
                raise ValueError(f"Invalid hex characters in address: {e}")
        else:
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
        
        # Check address format - must be 0x followed by exactly 64 hex chars
        normalized_address = ensure_hex_prefix(address.lower())
        return bool(RoochAddress.ADDRESS_REGEX.match(normalized_address))
    
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
        if not RoochAddress.is_valid_address(address):
            raise ValueError(f"Invalid Rooch address: {address}")
        
        return ensure_hex_prefix(address.lower())
    
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
        return "0x" + self._bytes.hex()
    
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
        return self.to_hex_full()
    
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
        # Remove leading zeros and add 0x prefix
        hex_str = self._bytes.hex()
        hex_str = hex_str.lstrip('0')
        if not hex_str:
            hex_str = '0'
        return "0x" + hex_str
    
    def to_hex_no_prefix(self) -> str:
        """Convert the address to hex string without 0x prefix
        
        Returns:
            Address as hex string without 0x prefix
        """
        return self._bytes.hex()


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