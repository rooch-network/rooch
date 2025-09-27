#!/usr/bin/env python3
# Copyright (c) Rooch Network
# SPDX-License-Identifier: Apache-2.0

"""
Migration adapter for converting between old TransactionArgument system 
and new Args system.

This module provides utilities to ease the migration from the old
TransactionArgument-based parameter system to the new Args system.
"""

from typing import Any, List, Union, Optional
from ...bcs import Args
from .move_types import RawBytesArgument, TransactionArgument
from ..tags.type_tags import TypeTagCode


class ArgsAdapter:
    """Adapter for migrating from TransactionArgument to Args system."""
    
    @staticmethod
    def convert_args_to_raw_bytes(args: List[Any]) -> List[RawBytesArgument]:
        """
        Convert a mixed list of arguments to RawBytesArgument objects.
        
        This function handles:
        - New Args system objects (with encode() method)
        - Old TransactionArgument objects
        - Raw bytes
        - Python primitives that need type inference
        
        Args:
            args: List of mixed argument types
            
        Returns:
            List of RawBytesArgument objects
        """
        result = []
        
        for arg in args:
            # Check if it's from the new Args system
            if hasattr(arg, 'encode') and callable(getattr(arg, 'encode')):
                # New Args system - get raw bytes
                raw_bytes = arg.encode()
                result.append(RawBytesArgument(raw_bytes))
            elif isinstance(arg, RawBytesArgument):
                # Already a RawBytesArgument
                result.append(arg)
            elif isinstance(arg, TransactionArgument):
                # Old TransactionArgument - extract the value and re-encode without type tag
                raw_bytes = ArgsAdapter._serialize_value_without_type_tag(arg)
                result.append(RawBytesArgument(raw_bytes))
            elif isinstance(arg, bytes):
                # Raw bytes
                result.append(RawBytesArgument(arg))
            else:
                # Python primitive - infer type and convert to Args
                args_obj = ArgsAdapter._infer_args_type(arg)
                raw_bytes = args_obj.encode()
                result.append(RawBytesArgument(raw_bytes))
        
        return result
    
    @staticmethod
    def _serialize_value_without_type_tag(transaction_arg: TransactionArgument) -> bytes:
        """Serialize TransactionArgument value without the type tag."""
        from ...bcs.serializer import BcsSerializer
        
        serializer = BcsSerializer()
        
        # Serialize based on type tag
        if transaction_arg.type_tag == TypeTagCode.U8:
            serializer.u8(transaction_arg.value)
        elif transaction_arg.type_tag == TypeTagCode.U16:
            serializer.u16(transaction_arg.value)
        elif transaction_arg.type_tag == TypeTagCode.U32:
            serializer.u32(transaction_arg.value)
        elif transaction_arg.type_tag == TypeTagCode.U64:
            serializer.u64(transaction_arg.value)
        elif transaction_arg.type_tag == TypeTagCode.U128:
            serializer.u128(transaction_arg.value)
        elif transaction_arg.type_tag == TypeTagCode.U256:
            serializer.u256(transaction_arg.value)
        elif transaction_arg.type_tag == TypeTagCode.BOOL:
            serializer.bool(transaction_arg.value)
        elif transaction_arg.type_tag == TypeTagCode.ADDRESS:
            from ...address.rooch import RoochAddress
            if isinstance(transaction_arg.value, str):
                addr = RoochAddress.from_hex(transaction_arg.value)
                serializer.fixed_bytes(addr.to_bytes())
            else:
                serializer.fixed_bytes(transaction_arg.value)
        elif transaction_arg.type_tag == TypeTagCode.VECTOR:
            # Handle vector serialization
            if isinstance(transaction_arg.value, (list, tuple)):
                values_list = list(transaction_arg.value)
                # Use a direct serialization approach for vectors
                serializer.u32(len(values_list))  # Length prefix
                for item in values_list:
                    serializer.u8(item)  # Assuming u8 vector for simplicity
            else:
                raise ValueError(f"Invalid vector value: {transaction_arg.value}")
        else:
            raise ValueError(f"Unsupported type tag: {transaction_arg.type_tag}")
        
        return serializer.output()
    
    @staticmethod
    def _infer_args_type(value: Any):
        """Infer the Args type from a Python value."""
        if isinstance(value, bool):
            return Args.bool(value)
        elif isinstance(value, int):
            # Default to u256 for backward compatibility, but warn about precision loss
            if value < 0:
                raise ValueError(f"Negative integers not supported: {value}")
            elif value <= 255:
                # Could be u8, but default to u64 for safety
                return Args.u64(value)
            elif value <= 2**64 - 1:
                return Args.u64(value)
            else:
                return Args.u256(value)
        elif isinstance(value, str):
            # Check if it's an address (starts with 0x and correct length)
            if value.startswith('0x') and len(value) == 66:  # 0x + 64 hex chars
                return Args.address(value)
            else:
                return Args.string(value)
        elif isinstance(value, (list, tuple)):
            # Try to infer vector type from first element
            if not value:
                return Args.vec_u8([])  # Empty vector defaults to u8
            
            first_elem = value[0]
            if isinstance(first_elem, bool):
                return Args.vec_bool(list(value))
            elif isinstance(first_elem, int):
                # Check if all values fit in u8
                if all(isinstance(x, int) and 0 <= x <= 255 for x in value):
                    return Args.vec_u8(list(value))
                else:
                    return Args.vec_u64(list(value))
            elif isinstance(first_elem, str) and first_elem.startswith('0x'):
                return Args.vec_address(list(value))
            else:
                raise ValueError(f"Cannot infer vector type from: {value}")
        else:
            raise ValueError(f"Cannot infer Args type for value: {value} (type: {type(value)})")


def migrate_function_args(function_id: str, ty_args: Optional[List[str]] = None, args: Optional[List[Any]] = None) -> dict:
    """
    Helper function to migrate function calls to the new Args system.
    
    This is a convenience function that shows how to convert old-style
    function calls to use the new Args system.
    
    Args:
        function_id: Function ID string
        ty_args: Type arguments (unchanged)
        args: Mixed list of arguments to convert
        
    Returns:
        Dictionary with converted arguments ready for FunctionArgument
    """
    if ty_args is None:
        ty_args = []
    if args is None:
        args = []
    
    # Convert all arguments to RawBytesArgument objects
    converted_args = ArgsAdapter.convert_args_to_raw_bytes(args)
    
    return {
        'function_id': function_id,
        'ty_args': ty_args,
        'args': converted_args
    }


# Convenience functions for common migration patterns
def create_transfer_args(recipient: str, amount: int, use_u64: bool = False):
    """Create transfer function arguments using new Args system."""
    return [
        Args.address(recipient),
        Args.u64(amount) if use_u64 else Args.u256(amount)
    ]


def create_faucet_args(amount: int):
    """Create faucet function arguments using new Args system."""
    return [Args.u256(amount)]


def create_swap_args(token_in: str, token_out: str, amount_in: int, min_amount_out: int, deadline: int):
    """Create DEX swap function arguments using new Args system."""
    return [
        Args.address(token_in),
        Args.address(token_out), 
        Args.u256(amount_in),
        Args.u256(min_amount_out),
        Args.u64(deadline)  # Deadline is typically u64, not u256
    ]
