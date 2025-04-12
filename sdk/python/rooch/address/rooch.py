#!/usr/bin/env python3
# Copyright (c) Rooch Network
# SPDX-License-Identifier: Apache-2.0

import re
from typing import Optional, Union

from ..utils.hex import ensure_hex_prefix, is_hex_string, strip_hex_prefix, to_hex
from ..utils.bytes import to_bytes


class RoochAddress:
    """Class for handling Rooch addresses"""
    
    # Rooch address regex pattern
    ADDRESS_REGEX = re.compile(r"^(0x)?[a-fA-F0-9]{64}$")
    
    def __init__(self, address: Union[str, bytes]):
        """Initialize a Rooch address
        
        Args:
            address: Address as a string or bytes
            
        Raises:
            ValueError: If the address is invalid
        """
        if isinstance(address, bytes):
            self._bytes = address
            self._str = to_hex(address)
        else:
            normalized_address = ensure_hex_prefix(address.lower())
            # For direct string construction, just check format
            if not bool(RoochAddress.ADDRESS_REGEX.match(normalized_address)):
                raise ValueError(f"Invalid Rooch address: {address}")
            
            self._str = normalized_address
            # Convert to bytes without the 0x prefix
            self._bytes = to_bytes(strip_hex_prefix(self._str))
    
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
        """Create a RoochAddress from a hex string
        
        Args:
            hex_str: Hex string with or without 0x prefix
            
        Returns:
            RoochAddress instance
            
        Raises:
            ValueError: If the hex string is invalid
        """
        # Remove 0x prefix if present
        clean_hex = hex_str
        if clean_hex.startswith("0x") or clean_hex.startswith("0X"):
            clean_hex = clean_hex[2:]
            
        # Check length - should be exactly 64 hex chars (32 bytes) for a valid address
        if len(clean_hex) != 64:
            if len(clean_hex) > 64:
                raise ValueError(f"Hex string too long: {hex_str}")
            else:
                raise ValueError(f"Hex string too short: {hex_str}")
        
        # Ensure the hex string is valid
        if not is_hex_string(clean_hex):
            raise ValueError(f"Invalid hex string: {hex_str}")
            
        # Add 0x prefix if it was not present
        normalized_hex = f"0x{clean_hex}"
        return RoochAddress(normalized_hex)
    
    def __str__(self) -> str:
        """Return the address as a string with 0x prefix"""
        return self._str
    
    def __repr__(self) -> str:
        """Return the string representation of the address"""
        return f"RoochAddress('{self._str}')"
    
    def __eq__(self, other: object) -> bool:
        """Check if two addresses are equal"""
        if not isinstance(other, RoochAddress):
            return False
        return self.to_hex() == other.to_hex()
    
    def __hash__(self) -> int:
        """Return hash value for the address"""
        return hash(self._str)
    
    def to_bytes(self) -> bytes:
        """Convert the address to bytes
        
        Returns:
            Address as bytes
        """
        return self._bytes
    
    def to_hex(self) -> str:
        """Convert the address to hex string with 0x prefix
        
        Returns:
            Address as hex string
        """
        return self._str
    
    def to_hex_no_prefix(self) -> str:
        """Convert the address to hex string without 0x prefix
        
        Returns:
            Address as hex string without 0x prefix
        """
        return strip_hex_prefix(self._str)


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