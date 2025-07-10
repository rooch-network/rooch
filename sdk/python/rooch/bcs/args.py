#!/usr/bin/env python3
# Copyright (c) Rooch Network
# SPDX-License-Identifier: Apache-2.0

"""
BCS Argument encoding utilities for Move function calls.

This module provides type-safe argument encoding for Move functions,
inspired by TypeScript SDK's Args class and Rust's parameter serialization.
"""

from typing import Union, List, Any
from enum import Enum

from ..bcs.serializer import BcsSerializer
from ..address.rooch import RoochAddress
from ..utils.hex import from_hex


class ArgType(Enum):
    """Move type tags for function arguments"""
    U8 = "u8"
    U16 = "u16"
    U32 = "u32"
    U64 = "u64"
    U128 = "u128"
    U256 = "u256"
    BOOL = "bool"
    ADDRESS = "address"
    STRING = "string"
    VECTOR = "vector"
    OBJECT_ID = "object_id"


class Args:
    """
    Type-safe argument encoder for Move function calls.
    
    This class provides static methods for creating properly encoded
    arguments that can be used in Move function calls. Each method
    returns raw bytes without type tags, matching the Rust 
    FunctionCall.args: Vec<Vec<u8>> format.
    """
    
    def __init__(self, value: bytes):
        """Initialize with raw bytes."""
        self._value = value
    
    def encode(self) -> bytes:
        """Return the encoded bytes."""
        return self._value
    
    def encode_hex(self) -> str:
        """Return hex-encoded string."""
        return f"0x{self._value.hex()}"
    
    # Primitive types
    
    @staticmethod
    def u8(value: int) -> 'Args':
        """Encode u8 value."""
        if not (0 <= value <= 255):
            raise ValueError(f"u8 value must be in range 0-255, got {value}")
        serializer = BcsSerializer()
        serializer.u8(value)
        return Args(serializer.output())
    
    @staticmethod
    def u16(value: int) -> 'Args':
        """Encode u16 value."""
        if not (0 <= value <= 65535):
            raise ValueError(f"u16 value must be in range 0-65535, got {value}")
        serializer = BcsSerializer()
        serializer.u16(value)
        return Args(serializer.output())
    
    @staticmethod
    def u32(value: int) -> 'Args':
        """Encode u32 value."""
        if not (0 <= value <= 4294967295):
            raise ValueError(f"u32 value must be in range 0-4294967295, got {value}")
        serializer = BcsSerializer()
        serializer.u32(value)
        return Args(serializer.output())
    
    @staticmethod
    def u64(value: int) -> 'Args':
        """Encode u64 value."""
        if not (0 <= value <= 18446744073709551615):
            raise ValueError(f"u64 value must be in range 0-18446744073709551615, got {value}")
        serializer = BcsSerializer()
        serializer.u64(value)
        return Args(serializer.output())
    
    @staticmethod
    def u128(value: int) -> 'Args':
        """Encode u128 value."""
        if not (0 <= value < 2**128):
            raise ValueError(f"u128 value must be in range 0-2^128-1, got {value}")
        serializer = BcsSerializer()
        serializer.u128(value)
        return Args(serializer.output())
    
    @staticmethod
    def u256(value: int) -> 'Args':
        """Encode u256 value."""
        if not (0 <= value < 2**256):
            raise ValueError(f"u256 value must be in range 0-2^256-1, got {value}")
        serializer = BcsSerializer()
        serializer.u256(value)
        return Args(serializer.output())
    
    @staticmethod
    def bool(value: bool) -> 'Args':
        """Encode boolean value."""
        serializer = BcsSerializer()
        serializer.bool(value)
        return Args(serializer.output())
    
    @staticmethod
    def address(value: Union[str, RoochAddress]) -> 'Args':
        """Encode address value."""
        if isinstance(value, str):
            if value.startswith("0x"):
                addr = RoochAddress.from_hex(value)
            else:
                raise ValueError(f"Address string must start with '0x', got {value}")
        else:
            addr = value
        
        serializer = BcsSerializer()
        serializer.fixed_bytes(addr.to_bytes())
        return Args(serializer.output())
    
    @staticmethod
    def string(value: str) -> 'Args':
        """Encode string value."""
        serializer = BcsSerializer()
        serializer.str(value)
        return Args(serializer.output())
    
    @staticmethod
    def object_id(value: str) -> 'Args':
        """Encode ObjectID value (32-byte hex string)."""
        if isinstance(value, str):
            if value.startswith("0x"):
                hex_str = value[2:]
            else:
                hex_str = value
            
            if len(hex_str) != 64:  # 32 bytes = 64 hex chars
                raise ValueError(f"ObjectID must be 32 bytes (64 hex chars), got {len(hex_str)}")
            
            object_bytes = from_hex(value)
        else:
            raise ValueError("ObjectID must be a hex string")
        
        serializer = BcsSerializer()
        serializer.fixed_bytes(object_bytes)
        return Args(serializer.output())
    
    # Vector types
    
    @staticmethod
    def vec_u8(values: List[int]) -> 'Args':
        """Encode vector<u8>."""
        for v in values:
            if not (0 <= v <= 255):
                raise ValueError(f"All u8 values must be in range 0-255, got {v}")
        
        serializer = BcsSerializer()
        serializer.sequence(values, lambda s, v: s.u8(v))
        return Args(serializer.output())
    
    @staticmethod
    def vec_u16(values: List[int]) -> 'Args':
        """Encode vector<u16>."""
        for v in values:
            if not (0 <= v <= 65535):
                raise ValueError(f"All u16 values must be in range 0-65535, got {v}")
        
        serializer = BcsSerializer()
        serializer.sequence(values, lambda s, v: s.u16(v))
        return Args(serializer.output())
    
    @staticmethod
    def vec_u32(values: List[int]) -> 'Args':
        """Encode vector<u32>."""
        for v in values:
            if not (0 <= v <= 4294967295):
                raise ValueError(f"All u32 values must be in range 0-4294967295, got {v}")
        
        serializer = BcsSerializer()
        serializer.sequence(values, lambda s, v: s.u32(v))
        return Args(serializer.output())
    
    @staticmethod
    def vec_u64(values: List[int]) -> 'Args':
        """Encode vector<u64>."""
        for v in values:
            if not (0 <= v <= 18446744073709551615):
                raise ValueError(f"All u64 values must be in range 0-18446744073709551615, got {v}")
        
        serializer = BcsSerializer()
        serializer.sequence(values, lambda s, v: s.u64(v))
        return Args(serializer.output())
    
    @staticmethod
    def vec_u128(values: List[int]) -> 'Args':
        """Encode vector<u128>."""
        for v in values:
            if not (0 <= v < 2**128):
                raise ValueError(f"All u128 values must be in range 0-2^128-1, got {v}")
        
        serializer = BcsSerializer()
        serializer.sequence(values, lambda s, v: s.u128(v))
        return Args(serializer.output())
    
    @staticmethod
    def vec_u256(values: List[int]) -> 'Args':
        """Encode vector<u256>."""
        for v in values:
            if not (0 <= v < 2**256):
                raise ValueError(f"All u256 values must be in range 0-2^256-1, got {v}")
        
        serializer = BcsSerializer()
        serializer.sequence(values, lambda s, v: s.u256(v))
        return Args(serializer.output())
    
    @staticmethod
    def vec_bool(values: List[bool]) -> 'Args':
        """Encode vector<bool>."""
        serializer = BcsSerializer()
        serializer.sequence(values, lambda s, v: s.bool(v))
        return Args(serializer.output())
    
    @staticmethod
    def vec_address(values: List[Union[str, RoochAddress]]) -> 'Args':
        """Encode vector<address>."""
        addresses = []
        for v in values:
            if isinstance(v, str):
                if v.startswith("0x"):
                    addresses.append(RoochAddress.from_hex(v))
                else:
                    raise ValueError(f"Address string must start with '0x', got {v}")
            else:
                addresses.append(v)
        
        serializer = BcsSerializer()
        serializer.sequence(addresses, lambda s, addr: s.fixed_bytes(addr.to_bytes()))
        return Args(serializer.output())
    
    @staticmethod
    def vec_string(values: List[str]) -> 'Args':
        """Encode vector<string>."""
        serializer = BcsSerializer()
        serializer.sequence(values, lambda s, v: s.str(v))
        return Args(serializer.output())
    
    @staticmethod
    def raw_bytes(value: bytes) -> 'Args':
        """Create Args from raw bytes (for advanced usage)."""
        return Args(value)
    
    @staticmethod
    def from_hex(value: str) -> 'Args':
        """Create Args from hex string (for advanced usage)."""
        return Args(from_hex(value))


def infer_and_encode(value: Any) -> Args:
    """
    Convenience function to infer type and encode value.
    
    Note: This function makes assumptions about types and should be used
    carefully. For precise control, use the specific Args.* methods.
    
    Type inference rules:
    - bool -> Args.bool()
    - int -> Args.u256() (default for Rooch)
    - str starting with "0x" and 64 chars -> Args.object_id()
    - str starting with "0x" -> Args.address()
    - str -> Args.string()
    - List[int] -> Args.vec_u256()
    - List[str] -> Args.vec_string()
    - List[bool] -> Args.vec_bool()
    """
    if isinstance(value, bool):
        return Args.bool(value)
    elif isinstance(value, int):
        # Default to u256 for integers in Rooch context
        return Args.u256(value)
    elif isinstance(value, str):
        if value.startswith("0x"):
            hex_part = value[2:]
            if len(hex_part) == 64:  # 32 bytes = ObjectID
                return Args.object_id(value)
            else:  # Address
                return Args.address(value)
        else:
            return Args.string(value)
    elif isinstance(value, list):
        if not value:
            raise ValueError("Cannot infer type of empty list")
        
        first_type = type(value[0])
        if not all(isinstance(v, first_type) for v in value):
            raise ValueError("All elements in list must be the same type")
        
        if isinstance(value[0], int):
            return Args.vec_u256(value)
        elif isinstance(value[0], str):
            return Args.vec_string(value)
        elif isinstance(value[0], bool):
            return Args.vec_bool(value)
        else:
            raise ValueError(f"Unsupported list element type: {first_type}")
    else:
        raise ValueError(f"Cannot infer encoding for type: {type(value)}")
