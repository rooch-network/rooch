#!/usr/bin/env python3
# Copyright (c) Rooch Network
# SPDX-License-Identifier: Apache-2.0

"""
Basic usage examples for the Rooch Python SDK.
This demonstrates how to connect to a Rooch node and perform basic operations
using the new Args system for type-safe parameter encoding.
"""

import asyncio
import json
from typing import Dict, Any

from rooch.client.client import RoochClient
from rooch.transport import RoochEnvironment
from rooch.crypto.keypair import KeyPair
from rooch.crypto.signer import Signer

# Import the new Args system for type-safe parameter encoding
from rooch.bcs import Args, MoveFunctionBuilder


async def get_account_info(client: RoochClient, address: str) -> Dict[str, Any]:
    """Get account information
    
    Args:
        client: Rooch client
        address: Account address
        
    Returns:
        Account information
    """
    print(f"\n=== Getting account info for {address} ===")
    
    try:
        account_info = await client.account.get_account(address)
        print(f"Account info: {json.dumps(account_info, indent=2)}")
        return account_info
    except Exception as e:
        print(f"Error getting account info: {e}")
        return {}


async def get_account_balance(client: RoochClient, address: str) -> Dict[str, Any]:
    """Get account balance
    
    Args:
        client: Rooch client
        address: Account address
        
    Returns:
        Balance information
    """
    print(f"\n=== Getting balance for {address} ===")
    
    try:
        balance = await client.account.get_balances(address)
        print(f"Balances: {json.dumps(balance, indent=2)}")
        return balance
    except Exception as e:
        print(f"Error getting balance: {e}")
        return {}


async def execute_view_function(client: RoochClient, function_id: str, type_args=None, args=None) -> Any:
    """Execute a view function
    
    Args:
        client: Rooch client
        function_id: Function ID (module::function)
        type_args: Optional type arguments
        args: Optional function arguments
        
    Returns:
        Function result
    """
    print(f"\n=== Executing view function {function_id} ===")
    
    try:
        result = await client.transaction.execute_view_function(
            function_id=function_id,
            type_args=type_args if type_args else [],
            args=args if args else []
        )
        print(f"Result: {json.dumps(result, indent=2)}")
        return result
    except Exception as e:
        print(f"Error executing view function: {e}")
        return None


async def transfer_coins_new_args(
    client: RoochClient, 
    signer,
    recipient_address: str,
    amount: int,
    coin_type: str = "0x3::gas_coin::RGas"
) -> Dict[str, Any]:
    """
    Transfer coins using the new Args system for type-safe parameters.
    
    This demonstrates the new parameter encoding system that provides:
    - Type safety with explicit type control (u8, u16, u32, u64, u128, u256)
    - Efficient serialization without type tags
    - Better error handling and validation
    
    Args:
        client: Rooch client
        signer: Transaction signer
        recipient_address: Recipient's address
        amount: Amount to transfer
        coin_type: Coin type
        
    Returns:
        Transaction result
    """
    print(f"\n=== Transferring {amount} {coin_type} using NEW Args system ===")
    
    try:
        # Get sender address
        sender_address = signer.get_address()
        print(f"Sender address: {sender_address}")
        
        # Create arguments using the new Args system
        # This provides precise type control and prevents common errors
        transfer_args = [
            Args.address(recipient_address),  # Type-safe address encoding
            Args.u64(amount)                  # Use u64 instead of u256 for efficiency
        ]
        
        print(f"✅ Created type-safe arguments:")
        print(f"   Recipient: {recipient_address}")
        print(f"   Amount: {amount} (as u64)")
        
        # Show the encoded bytes (no type tags included)
        for i, arg in enumerate(transfer_args):
            encoded = arg.encode()
            print(f"   Arg {i}: 0x{encoded.hex()[:20]}... ({len(encoded)} bytes)")
        
        # Execute the transfer with new args
        result = await client.execute_move_call(
            signer=signer,
            function_id="0x1::coin::transfer",
            type_args=[coin_type],
            args=transfer_args,  # Use the new Args objects directly
            max_gas_amount=10_000_000
        )
        
        print(f"Transfer transaction result: {json.dumps(result, indent=2)}")
        return result
    except Exception as e:
        print(f"Error transferring coins: {e}")
        return {}


async def demonstrate_args_system_benefits():
    """Demonstrate the benefits of the new Args system."""
    
    print("\n=== New Args System Benefits Demo ===")
    
    # 1. Type precision
    print("\n1. Type Precision:")
    amount = 1000
    
    print(f"   Value: {amount}")
    print("   Old system: Would default to u256 (32 bytes)")
    print("   New system: Can choose optimal type")
    
    types_demo = [
        ("u8", Args.u8(amount % 256)),
        ("u16", Args.u16(amount)),
        ("u32", Args.u32(amount)),
        ("u64", Args.u64(amount)),
        ("u256", Args.u256(amount))
    ]
    
    for type_name, arg in types_demo:
        encoded = arg.encode()
        print(f"     {type_name:4}: {len(encoded)} bytes")
    
    # 2. Error prevention
    print("\n2. Error Prevention:")
    try:
        Args.u8(256)  # This will fail with clear error
    except ValueError as e:
        print(f"   ✅ Prevented u8 overflow: {e}")
    
    try:
        Args.address("invalid")  # This will fail with clear error
    except ValueError as e:
        print(f"   ✅ Prevented invalid address: {e}")
    
    # 3. Vector support
    print("\n3. Vector Support:")
    recipients = [
        "0x1111111111111111111111111111111111111111111111111111111111111111",
        "0x2222222222222222222222222222222222222222222222222222222222222222"
    ]
    amounts = [100, 200, 300]
    
    vec_addrs = Args.vec_address(recipients)
    vec_amounts = Args.vec_u64(amounts)
    
    print(f"   Address vector: {len(vec_addrs.encode())} bytes")
    print(f"   Amount vector: {len(vec_amounts.encode())} bytes")
    
    # 4. Builder pattern
    print("\n4. Builder Pattern for Complex Functions:")
    builder = MoveFunctionBuilder("0x1::dex::swap")
    builder.add_arg(Args.address(recipients[0]))    # token_in
    builder.add_arg(Args.address(recipients[1]))    # token_out
    builder.add_arg(Args.u256(1000))               # amount_in
    builder.add_arg(Args.u256(950))                # min_amount_out
    builder.add_arg(Args.u64(1700000000))          # deadline (u64!)
    builder.add_arg(Args.bool(True))               # exact_input
    
    print(f"   Function: {builder.function_id}")
    print(f"   Arguments: {len(builder.args)} with precise types")
    
    total_size = sum(len(arg.encode()) for arg in builder.args)
    print(f"   Total size: {total_size} bytes (no type tag overhead)")


async def faucet_claim_new_args(client: RoochClient, signer, amount: int = 10000):
    """Claim from faucet using new Args system."""
    
    print(f"\n=== Claiming {amount} from faucet using NEW Args system ===")
    
    try:
        # Create faucet arguments using new Args system
        faucet_args = [Args.u256(amount)]  # Faucet typically uses u256
        
        print(f"✅ Created faucet argument:")
        print(f"   Amount: {amount} (as u256)")
        
        encoded = faucet_args[0].encode()
        print(f"   Encoded: 0x{encoded.hex()[:20]}... ({len(encoded)} bytes)")
        
        # Execute faucet claim
        result = await client.execute_move_call(
            signer=signer,
            function_id="0x3::gas_coin::faucet_coin",
            type_args=[],
            args=faucet_args,
            max_gas_amount=10_000_000
        )
        
        print(f"Faucet claim result: {json.dumps(result, indent=2)}")
        return result
    except Exception as e:
        print(f"Error claiming from faucet: {e}")
        return {}


async def main() -> None:
    """Main function demonstrating the new Args system"""
    try:
        # Connect to Rooch local node by default
        # You can also connect to testnet using RoochEnvironment.TESTNET
        async with RoochClient(RoochEnvironment.LOCAL) as client:
            print("=== Connected to Rooch node ===")
            
            # Generate a new key pair for demonstration
            print("\n=== Generating a new key pair ===")
            keypair = KeyPair.generate()
            # Note: In real usage, you'd use a proper signer implementation
            # This is just for demonstration of the Args system
            
            # Get chain ID
            chain_id = await client.get_chain_id()
            print(f"\n=== Chain ID: {chain_id} ===")
            
            # Demonstrate the new Args system benefits
            await demonstrate_args_system_benefits()
            
            print("\n=== Args System Migration Complete ===")
            print("Key benefits demonstrated:")
            print("✅ Type safety prevents runtime errors")
            print("✅ Precise type control (u8, u16, u32, u64, u128, u256)")
            print("✅ Efficient serialization without type tags")
            print("✅ Clean builder pattern for complex calls")
            print("✅ Comprehensive error handling")
            print("✅ Vector support for batch operations")
            
            print("\nNext steps for migration:")
            print("1. Replace TransactionArgument with Args.* methods")
            print("2. Use specific types (Args.u64) instead of generic inference")  
            print("3. Leverage builder pattern for complex functions")
            print("4. Test with the new type-safe parameter system")
            
    except Exception as e:
        print(f"Error: {e}")


if __name__ == "__main__":
    asyncio.run(main())