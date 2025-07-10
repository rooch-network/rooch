#!/usr/bin/env python3
# Copyright (c) Rooch Network
# SPDX-License-Identifier: Apache-2.0

"""
Move function call builders.

This module provides clean APIs for building Move function calls
with type-safe argument encoding.
"""

from typing import List, Union, Any, Optional
from .args import Args
from ..transactions.move.move_types import FunctionArgument
from ..transactions.move.module_id import ModuleId, FunctionId
from ..transactions.tags.type_tags import TypeTag


class MoveFunctionBuilder:
    """
    Builder for Move function calls with type-safe arguments.
    
    Usage:
        builder = MoveFunctionBuilder("0x3::transfer::transfer_coin")
        builder.add_arg(Args.address("0x123..."))
        builder.add_arg(Args.u256(1000))
        function_call = builder.build()
    """
    
    def __init__(self, function_id: str):
        """Initialize with function ID string like '0x3::transfer::transfer_coin'."""
        self.function_id = function_id
        self.args: List[Args] = []
        self.type_args: List[TypeTag] = []
    
    def add_arg(self, arg: Args) -> 'MoveFunctionBuilder':
        """Add a typed argument. Returns self for chaining."""
        self.args.append(arg)
        return self
    
    def add_type_arg(self, type_arg: TypeTag) -> 'MoveFunctionBuilder':
        """Add a type argument. Returns self for chaining."""
        self.type_args.append(type_arg)
        return self
    
    def build(self) -> FunctionArgument:
        """Build the final FunctionArgument object."""
        # Convert Args objects to raw bytes
        raw_args = [arg.encode() for arg in self.args]
        
        # Create FunctionArgument with raw bytes (no type information)
        return FunctionArgument(
            function_id=self.function_id,
            ty_args=self.type_args,
            args=raw_args  # Pass raw bytes directly
        )


# Convenience functions for common operations

def transfer_coin(
    to_address: str,
    amount: int,
    coin_type: Optional[str] = None
) -> FunctionArgument:
    """
    Create a transfer_coin function call.
    
    Args:
        to_address: Recipient address (0x...)
        amount: Amount to transfer (will be encoded as u256)
        coin_type: Optional coin type for type arguments
    
    Returns:
        FunctionArgument ready for transaction
    """
    builder = MoveFunctionBuilder("0x3::transfer::transfer_coin")
    builder.add_arg(Args.address(to_address))
    builder.add_arg(Args.u256(amount))
    
    if coin_type:
        # Parse coin type string into TypeTag if needed
        # For now, just use default implementation
        pass
    
    return builder.build()


def faucet_claim(amount: int) -> FunctionArgument:
    """
    Create a faucet claim function call.
    
    Args:
        amount: Amount to claim (will be encoded as u256)
    
    Returns:
        FunctionArgument ready for transaction
    """
    return (MoveFunctionBuilder("0x3::gas_coin::faucet_coin")
            .add_arg(Args.u256(amount))
            .build())


# Factory functions for common argument patterns

def create_u256_args(*values: int) -> List[Args]:
    """Create a list of u256 arguments."""
    return [Args.u256(v) for v in values]


def create_address_args(*addresses: str) -> List[Args]:
    """Create a list of address arguments."""
    return [Args.address(addr) for addr in addresses]


def create_mixed_args(*values: Any) -> List[Args]:
    """
    Create arguments with automatic type inference.
    
    Warning: This uses type inference which may not always be correct.
    For precise control, use specific Args.* methods.
    """
    from .args import infer_and_encode
    return [infer_and_encode(v) for v in values]


# Example usage patterns as documentation

def example_usage():
    """Examples of how to use the new argument system."""
    
    # Example 1: Manual builder pattern (most precise)
    transfer = (MoveFunctionBuilder("0x3::transfer::transfer_coin")
                .add_arg(Args.address("0x123"))
                .add_arg(Args.u256(1000))
                .build())
    
    # Example 2: Convenience function
    transfer2 = transfer_coin("0x123", 1000)
    
    # Example 3: Complex types
    complex_call = (MoveFunctionBuilder("0x1::some_module::complex_function")
                    .add_arg(Args.vec_u64([1, 2, 3, 4]))
                    .add_arg(Args.vec_address(["0x123", "0x456"]))
                    .add_arg(Args.bool(True))
                    .build())
    
    # Example 4: Type inference (use with caution)
    auto_args = create_mixed_args("0x123", 1000, True, [1, 2, 3])
    auto_call = (MoveFunctionBuilder("0x1::auto::function")
                 .add_arg(auto_args[0])
                 .add_arg(auto_args[1])
                 .add_arg(auto_args[2])
                 .add_arg(auto_args[3])
                 .build())
    
    return [transfer, transfer2, complex_call, auto_call]
