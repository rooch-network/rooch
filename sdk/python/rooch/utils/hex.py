#!/usr/bin/env python3
# Copyright (c) Rooch Network
# SPDX-License-Identifier: Apache-2.0

from typing import Union


def to_hex(value: Union[bytes, bytearray]) -> str:
    """Convert bytes to hex string with 0x prefix
    
    Args:
        value: Bytes to convert
        
    Returns:
        Hex string with 0x prefix
    """
    return "0x" + value.hex()


def from_hex(hex_str: str) -> bytes:
    """Convert hex string to bytes
    
    Args:
        hex_str: Hex string (with or without 0x prefix)
        
    Returns:
        Bytes
        
    Raises:
        ValueError: If the input contains non-hex characters
    """
    # Remove 0x prefix if present
    if hex_str.startswith("0x") or hex_str.startswith("0X"):
        hex_str = hex_str[2:]
        
    # Add leading zero if hex_str has odd length
    if len(hex_str) % 2 != 0:
        hex_str = "0" + hex_str
    
    try:
        return bytes.fromhex(hex_str)
    except ValueError:
        raise ValueError(f"Invalid hex string: {hex_str}")


def ensure_hex_prefix(hex_str: str) -> str:
    """Ensure a hex string has 0x prefix
    
    Args:
        hex_str: Hex string (with or without 0x prefix)
        
    Returns:
        Hex string with 0x prefix
    """
    if not hex_str.startswith("0x") and not hex_str.startswith("0X"):
        return "0x" + hex_str
    return hex_str


def strip_hex_prefix(hex_str: str) -> str:
    """Remove 0x prefix from a hex string if present
    
    Args:
        hex_str: Hex string (with or without 0x prefix)
        
    Returns:
        Hex string without 0x prefix
    """
    if hex_str.startswith("0x") or hex_str.startswith("0X"):
        return hex_str[2:]
    return hex_str


def is_hex_string(value: str) -> bool:
    """Check if a string is a valid hex string
    
    Args:
        value: String to check
        
    Returns:
        True if the string is a valid hex string (with or without 0x prefix)
    """
    if value.startswith("0x") or value.startswith("0X"):
        value = value[2:]
    
    try:
        int(value, 16)
        return True
    except ValueError:
        return False