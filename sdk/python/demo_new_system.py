#!/usr/bin/env python3
# Copyright (c) Rooch Network
# SPDX-License-Identifier: Apache-2.0

"""
Complete example using the new argument encoding system.

This demonstrates how to use the refactored parameter system
for real Move function calls.
"""

import sys
import os
sys.path.insert(0, os.path.join(os.path.dirname(__file__), '..', '..'))

from rooch.bcs import Args, MoveFunctionBuilder, transfer_coin
from rooch.transactions.move.move_types import FunctionArgument, RawBytesArgument


def create_legacy_compatible_function_argument(function_id: str, raw_args: list) -> dict:
    """
    Create a function argument that's compatible with the existing transaction system.
    This bridges the new Args system with the current FunctionArgument class.
    """
    
    # Convert Args objects to raw bytes if needed
    raw_bytes_list = []
    for arg in raw_args:
        if hasattr(arg, 'encode'):
            raw_bytes_list.append(arg.encode())
        elif isinstance(arg, bytes):
            raw_bytes_list.append(arg)
        else:
            raise ValueError(f"Unsupported argument type: {type(arg)}")
    
    # Return dictionary representation that can be used by existing systems
    return {
        "function_id": function_id,
        "ty_args": [],
        "args": [f"0x{arg.hex()}" for arg in raw_bytes_list]
    }


def demo_transfer_transaction():
    """Demonstrate transfer transaction with new argument system."""
    print("=== Transfer Transaction Demo ===")
    
    # Recipient address and amount
    recipient = "0xe787f41c2fc947febe4fcfd414cfc379137f01427116e9c62c841551a0ef6c4f"
    amount = 1000
    
    # Method 1: Using MoveFunctionBuilder (most flexible)
    transfer_call = (MoveFunctionBuilder("0x3::transfer::transfer_coin")
                     .add_arg(Args.address(recipient))
                     .add_arg(Args.u256(amount))
                     .build())
    
    print(f"Transfer function call created with builder pattern")
    
    # Method 2: Using convenience function (simplest)
    transfer_call2 = transfer_coin(recipient, amount)
    print(f"Transfer function call created with convenience function")
    
    # Method 3: Raw args for integration with existing system
    raw_args = [
        Args.address(recipient),
        Args.u256(amount)
    ]
    
    legacy_format = create_legacy_compatible_function_argument(
        "0x3::transfer::transfer_coin",
        raw_args
    )
    
    print(f"Legacy compatible format: {legacy_format}")
    
    # Show the actual encoded bytes
    addr_bytes = Args.address(recipient).encode()
    amount_bytes = Args.u256(amount).encode()
    
    print(f"Address bytes (32 bytes): 0x{addr_bytes.hex()}")
    print(f"Amount bytes (32 bytes): 0x{amount_bytes.hex()}")
    print(f"Total argument size: {len(addr_bytes) + len(amount_bytes)} bytes")


def demo_faucet_transaction():
    """Demonstrate faucet transaction."""
    print("\n=== Faucet Transaction Demo ===")
    
    # Request 10000 gas coins
    amount = 10000
    
    faucet_call = (MoveFunctionBuilder("0x3::gas_coin::faucet_coin")
                   .add_arg(Args.u256(amount))
                   .build())
    
    print(f"Faucet call created for {amount} gas coins")
    
    # Show encoded argument
    amount_bytes = Args.u256(amount).encode()
    print(f"Amount bytes: 0x{amount_bytes.hex()}")


def demo_complex_function():
    """Demonstrate a complex function with multiple argument types."""
    print("\n=== Complex Function Demo ===")
    
    # Imagine a function that takes various types
    function_call = (MoveFunctionBuilder("0x1::example::complex_function")
                     .add_arg(Args.address("0x1234567890123456789012345678901234567890123456789012345678901234"))
                     .add_arg(Args.u64(1000))           # Different from u256
                     .add_arg(Args.bool(True))
                     .add_arg(Args.string("metadata"))
                     .add_arg(Args.vec_u8([1, 2, 3, 4, 5]))
                     .add_arg(Args.vec_address([
                         "0x1111111111111111111111111111111111111111111111111111111111111111",
                         "0x2222222222222222222222222222222222222222222222222222222222222222"
                     ]))
                     .build())
    
    print("Complex function call created with:")
    print("- address parameter")
    print("- u64 parameter (8 bytes, not 32)")
    print("- bool parameter")
    print("- string parameter")
    print("- vector<u8> parameter")
    print("- vector<address> parameter")
    
    # Show argument sizes
    args = [
        Args.address("0x1234567890123456789012345678901234567890123456789012345678901234"),
        Args.u64(1000),
        Args.bool(True),
        Args.string("metadata"),
        Args.vec_u8([1, 2, 3, 4, 5]),
        Args.vec_address([
            "0x1111111111111111111111111111111111111111111111111111111111111111",
            "0x2222222222222222222222222222222222222222222222222222222222222222"
        ])
    ]
    
    total_size = 0
    for i, arg in enumerate(args):
        size = len(arg.encode())
        total_size += size
        print(f"Arg {i}: {size} bytes")
    
    print(f"Total argument size: {total_size} bytes")


def demo_type_precision():
    """Demonstrate the importance of type precision."""
    print("\n=== Type Precision Demo ===")
    
    # Different integer types produce different output
    value = 1000
    
    u8_bytes = Args.u8(value % 256).encode()        # 1 byte
    u16_bytes = Args.u16(value % 65536).encode()    # 2 bytes
    u32_bytes = Args.u32(value).encode()            # 4 bytes
    u64_bytes = Args.u64(value).encode()            # 8 bytes
    u128_bytes = Args.u128(value).encode()          # 16 bytes
    u256_bytes = Args.u256(value).encode()          # 32 bytes
    
    print(f"Same value {value} encoded as different types:")
    print(f"u8:   0x{u8_bytes.hex()} ({len(u8_bytes)} bytes)")
    print(f"u16:  0x{u16_bytes.hex()} ({len(u16_bytes)} bytes)")
    print(f"u32:  0x{u32_bytes.hex()} ({len(u32_bytes)} bytes)")
    print(f"u64:  0x{u64_bytes.hex()} ({len(u64_bytes)} bytes)")
    print(f"u128: 0x{u128_bytes.hex()} ({len(u128_bytes)} bytes)")
    print(f"u256: 0x{u256_bytes.hex()} ({len(u256_bytes)} bytes)")
    
    print("\nThis precision is crucial for Move function signatures!")


def demo_vector_encoding():
    """Demonstrate vector encoding efficiency."""
    print("\n=== Vector Encoding Demo ===")
    
    # Compare vector encodings
    small_vec = Args.vec_u8([1, 2, 3]).encode()
    medium_vec = Args.vec_u64([1000, 2000, 3000]).encode()
    large_vec = Args.vec_u256([1000, 2000, 3000]).encode()
    
    print(f"vec_u8([1,2,3]):        0x{small_vec.hex()} ({len(small_vec)} bytes)")
    print(f"vec_u64([1000,2000,3000]): 0x{medium_vec.hex()[:20]}... ({len(medium_vec)} bytes)")
    print(f"vec_u256([1000,2000,3000]): 0x{large_vec.hex()[:20]}... ({len(large_vec)} bytes)")
    
    # Break down the vector structure
    print("\nVector encoding structure:")
    print("- Length prefix (ULEB128)")
    print("- Elements in sequence")
    print("- No type information (determined by Move function signature)")


def compare_old_vs_new():
    """Compare old and new parameter encoding."""
    print("\n=== Old vs New System Comparison ===")
    
    # Simulate old system output (with type tags)
    print("Old system (with type tags):")
    print("Address: 0x04 + 32 bytes = 33 bytes total")
    print("U256:    0x0a + 32 bytes = 33 bytes total")
    print("Total:   66 bytes")
    
    # New system output (raw bytes only)
    addr_arg = Args.address("0x1234567890123456789012345678901234567890123456789012345678901234")
    u256_arg = Args.u256(1000)
    
    addr_size = len(addr_arg.encode())
    u256_size = len(u256_arg.encode())
    
    print(f"\nNew system (raw bytes only):")
    print(f"Address: {addr_size} bytes")
    print(f"U256:    {u256_size} bytes")
    print(f"Total:   {addr_size + u256_size} bytes")
    print(f"Savings: {66 - (addr_size + u256_size)} bytes per function call")


def main():
    """Run all demonstrations."""
    print("Rooch Python SDK - New Parameter System Demo")
    print("=" * 50)
    
    demo_transfer_transaction()
    demo_faucet_transaction()
    demo_complex_function()
    demo_type_precision()
    demo_vector_encoding()
    compare_old_vs_new()
    
    print("\n" + "=" * 50)
    print("Summary:")
    print("✓ Type-safe argument encoding")
    print("✓ Precise type control (u8, u16, u32, u64, u128, u256)")
    print("✓ Efficient vector encoding")
    print("✓ No type tag overhead")
    print("✓ Compatible with Rust/TypeScript SDKs")
    print("✓ Builder pattern for complex calls")
    print("✓ Convenience functions for common operations")
    
    print("\nNext steps:")
    print("1. Update existing code to use new Args system")
    print("2. Replace TransactionArgument with Args.* methods")
    print("3. Use MoveFunctionBuilder for complex function calls")
    print("4. Leverage type safety to prevent runtime errors")


if __name__ == "__main__":
    main()
