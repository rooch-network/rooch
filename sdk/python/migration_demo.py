#!/usr/bin/env python3
# Copyright (c) Rooch Network
# SPDX-License-Identifier: Apache-2.0

"""
Demonstration of migrating from old parameter system to new Args system.

This file shows practical examples of how to migrate existing code
from the old TransactionArgument system to the new Args system.
"""

import asyncio
from rooch.bcs import Args, MoveFunctionBuilder
from rooch.transactions.move.args_adapter import ArgsAdapter, create_transfer_args, create_faucet_args


def demo_migration_patterns():
    """Demonstrate various migration patterns."""
    
    print("=== Migration Patterns Demo ===\n")
    
    # Pattern 1: Simple type migration
    print("1. Simple type migration:")
    print("   Old: amount = 1000  # Would be inferred as u256")
    print("   New: amount = Args.u64(1000)  # Explicit type control")
    
    old_amount = 1000
    new_amount = Args.u64(1000)
    print(f"   Old encoding would include type tag")
    print(f"   New encoding: 0x{new_amount.encode().hex()} ({len(new_amount.encode())} bytes)")
    
    # Pattern 2: Address migration  
    print("\n2. Address migration:")
    recipient = "0xe787f41c2fc947febe4fcfd414cfc379137f01427116e9c62c841551a0ef6c4f"
    print(f"   Old: recipient = '{recipient}'  # Type inferred")
    print(f"   New: recipient = Args.address('{recipient}')")
    
    old_addr = recipient
    new_addr = Args.address(recipient)
    print(f"   Old encoding would include type tag")
    print(f"   New encoding: 0x{new_addr.encode().hex()} ({len(new_addr.encode())} bytes)")
    
    # Pattern 3: Function call migration
    print("\n3. Function call migration:")
    print("   Old approach:")
    print("   ```python")
    print("   args = [")
    print("       TransactionArgument(TypeTagCode.ADDRESS, recipient),")
    print("       TransactionArgument(TypeTagCode.U256, amount)")
    print("   ]")
    print("   func_arg = FunctionArgument('0x3::transfer::transfer_coin', [], args)")
    print("   ```")
    
    print("\n   New approach:")
    print("   ```python")
    print("   args = [")
    print("       Args.address(recipient),")
    print("       Args.u64(amount)  # or Args.u256(amount)")
    print("   ]")
    print("   func_arg = FunctionArgument('0x3::transfer::transfer_coin', [], args)")
    print("   ```")


def demo_convenience_functions():
    """Demonstrate convenience functions for common patterns."""
    
    print("\n=== Convenience Functions Demo ===\n")
    
    # Transfer function
    recipient = "0xe787f41c2fc947febe4fcfd414cfc379137f01427116e9c62c841551a0ef6c4f"
    amount = 1000
    
    print("1. Transfer function arguments:")
    transfer_args = create_transfer_args(recipient, amount, use_u64=True)
    print(f"   Function: 0x3::transfer::transfer_coin")
    print(f"   Args: {len(transfer_args)} arguments")
    for i, arg in enumerate(transfer_args):
        encoded = arg.encode()
        print(f"     Arg {i}: 0x{encoded.hex()[:20]}... ({len(encoded)} bytes)")
    
    # Faucet function
    faucet_amount = 10000
    print(f"\n2. Faucet function arguments:")
    faucet_args = create_faucet_args(faucet_amount)
    print(f"   Function: 0x3::gas_coin::faucet_coin")
    print(f"   Args: {len(faucet_args)} arguments")
    for i, arg in enumerate(faucet_args):
        encoded = arg.encode()
        print(f"     Arg {i}: 0x{encoded.hex()[:20]}... ({len(encoded)} bytes)")


def demo_builder_pattern():
    """Demonstrate the builder pattern for complex functions."""
    
    print("\n=== Builder Pattern Demo ===\n")
    
    # Complex function with multiple argument types
    print("Complex DEX swap function:")
    
    builder = MoveFunctionBuilder("0x1::dex::swap_exact_input")
    builder.add_arg(Args.address("0x1111111111111111111111111111111111111111111111111111111111111111"))  # token_in
    builder.add_arg(Args.address("0x2222222222222222222222222222222222222222222222222222222222222222"))  # token_out  
    builder.add_arg(Args.u256(1000))        # amount_in
    builder.add_arg(Args.u256(950))         # min_amount_out
    builder.add_arg(Args.u64(1700000000))   # deadline (u64, not u256!)
    builder.add_arg(Args.bool(True))        # exact_input flag
    
    print(f"   Function: {builder.function_id}")
    print(f"   Arguments: {len(builder.args)}")
    
    total_size = 0
    for i, arg in enumerate(builder.args):
        encoded = arg.encode()
        size = len(encoded)
        total_size += size
        print(f"     Arg {i}: {size} bytes")
    
    print(f"   Total argument size: {total_size} bytes")
    print("   ✅ Type precision prevents errors (deadline is u64, not u256)")


def demo_vector_arguments():
    """Demonstrate vector argument handling."""
    
    print("\n=== Vector Arguments Demo ===\n")
    
    # Different vector types
    vectors = [
        ("Batch addresses", Args.vec_address([
            "0x1111111111111111111111111111111111111111111111111111111111111111",
            "0x2222222222222222222222222222222222222222222222222222222222222222",
            "0x3333333333333333333333333333333333333333333333333333333333333333"
        ])),
        ("Amounts list", Args.vec_u64([100, 200, 300, 400, 500])),
        ("Flags array", Args.vec_bool([True, False, True, False])),
        ("Bytes data", Args.vec_u8([0x01, 0x02, 0x03, 0x04, 0xFF]))
    ]
    
    for name, vec_arg in vectors:
        encoded = vec_arg.encode()
        print(f"   {name}: {len(encoded)} bytes")
        print(f"     Encoded: 0x{encoded.hex()[:40]}{'...' if len(encoded) > 20 else ''}")


def demo_error_prevention():
    """Demonstrate how the new system prevents common errors."""
    
    print("\n=== Error Prevention Demo ===\n")
    
    # Type overflow prevention
    print("1. Type overflow prevention:")
    try:
        # This would fail in the old system at runtime
        Args.u8(256)  # Overflow
    except ValueError as e:
        print(f"   ✅ Caught u8 overflow: {e}")
    
    try:
        Args.u64(-1)  # Negative value
    except ValueError as e:
        print(f"   ✅ Caught negative u64: {e}")
    
    # Address format validation
    print("\n2. Address format validation:")
    try:
        Args.address("invalid_address")
    except ValueError as e:
        print(f"   ✅ Caught invalid address: {e}")
    
    # Type precision demonstration
    print("\n3. Type precision benefits:")
    value = 1000
    types = [
        ("u8", lambda: Args.u8(value % 256)),
        ("u16", lambda: Args.u16(value)),
        ("u32", lambda: Args.u32(value)),
        ("u64", lambda: Args.u64(value)),
        ("u128", lambda: Args.u128(value)),
        ("u256", lambda: Args.u256(value))
    ]
    
    print(f"   Value {value} in different types:")
    for type_name, constructor in types:
        arg = constructor()
        encoded = arg.encode()
        print(f"     {type_name:4}: {len(encoded)} bytes")
    
    print("   ✅ Move functions receive exactly the right type!")


async def main():
    """Run all migration demonstrations."""
    print("Python SDK Parameter Migration Demonstration")
    print("=" * 60)
    
    demo_migration_patterns()
    demo_convenience_functions() 
    demo_builder_pattern()
    demo_vector_arguments()
    demo_error_prevention()
    
    print("\n" + "=" * 60)
    print("Migration Summary:")
    print("✅ Type safety prevents runtime errors")
    print("✅ Precise type control (u8, u16, u32, u64, u128, u256)")
    print("✅ Efficient serialization without type tags") 
    print("✅ Clean builder pattern for complex calls")
    print("✅ Comprehensive error handling")
    print("✅ Compatible with Rust/TypeScript SDKs")
    
    print("\nNext Steps:")
    print("1. Replace TransactionArgument usage with Args.* methods")
    print("2. Use specific types (Args.u64) instead of generic inference")
    print("3. Leverage builder pattern for complex functions")
    print("4. Utilize convenience functions for common patterns")
    print("5. Test thoroughly with the new type-safe system")
    
    print("\n🎉 Migration to new Args system complete!")


if __name__ == "__main__":
    asyncio.run(main())
