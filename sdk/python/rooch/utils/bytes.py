#!/usr/bin/env python3
# Copyright (c) Rooch Network
# SPDX-License-Identifier: Apache-2.0

import base64
from typing import List, Union, cast


def bytes_to_list(b: bytes) -> List[int]:
    """Convert bytes to a list of integers
    
    Args:
        b: Bytes to convert
        
    Returns:
        List of integers representing bytes
    """
    return list(b)


def list_to_bytes(lst: List[int]) -> bytes:
    """Convert a list of integers to bytes
    
    Args:
        lst: List of integers to convert
        
    Returns:
        Bytes
        
    Raises:
        ValueError: If any value is outside the valid byte range (0-255)
    """
    for i, value in enumerate(lst):
        if not (0 <= value <= 255):
            raise ValueError(f"Invalid byte value at index {i}: {value}")
    
    return bytes(lst)


def to_base64(data: Union[bytes, bytearray, str]) -> str:
    """Convert data to base64 string
    
    Args:
        data: Data to convert (bytes, bytearray, or str)
        
    Returns:
        Base64 string
    """
    if isinstance(data, str):
        data = data.encode("utf-8")
    
    return base64.b64encode(cast(bytes, data)).decode("utf-8")


def from_base64(b64_str: str) -> bytes:
    """Convert base64 string to bytes
    
    Args:
        b64_str: Base64 string to convert
        
    Returns:
        Bytes
        
    Raises:
        ValueError: If the input is not a valid base64 string
    """
    try:
        return base64.b64decode(b64_str)
    except Exception as e:
        raise ValueError(f"Invalid base64 string: {str(e)}")


def to_bytes(value: Union[bytes, bytearray, str, int, List[int]]) -> bytes:
    """Convert various types to bytes
    
    Args:
        value: Value to convert (bytes, bytearray, str, int, or list of integers)
        
    Returns:
        Bytes
        
    Raises:
        TypeError: If the input type is not supported
        ValueError: If the input contains invalid values
    """
    if isinstance(value, (bytes, bytearray)):
        return bytes(value)
    elif isinstance(value, str):
        try:
            # Try to interpret as hex string if it starts with 0x
            if value.startswith("0x") or value.startswith("0X"):
                from .hex import from_hex
                return from_hex(value)
            else:
                # Otherwise interpret as UTF-8
                return value.encode("utf-8")
        except Exception as e:
            raise ValueError(f"Invalid string: {str(e)}")
    elif isinstance(value, int):
        if value < 0:
            raise ValueError("Cannot convert negative integer to bytes")
        # Convert to minimal bytes representation
        return value.to_bytes((value.bit_length() + 7) // 8 or 1, byteorder="big")
    elif isinstance(value, list):
        return list_to_bytes(value)
    else:
        raise TypeError(f"Cannot convert {type(value)} to bytes")


def bytes_to_str(data: bytes, encoding: str = "utf-8") -> str:
    """Convert bytes to string using the specified encoding
    
    Args:
        data: Bytes to convert
        encoding: Encoding to use (default: utf-8)
        
    Returns:
        String
        
    Raises:
        UnicodeDecodeError: If the bytes cannot be decoded using the specified encoding
    """
    return data.decode(encoding)