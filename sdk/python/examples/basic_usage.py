#!/usr/bin/env python3
# Copyright (c) Rooch Network
# SPDX-License-Identifier: Apache-2.0

"""
Basic usage examples for the Rooch Python SDK.
This demonstrates how to connect to a Rooch node and perform basic operations.
"""

import asyncio
import json
from typing import Dict, Any

from rooch.client.client import RoochClient
from rooch.transport import RoochEnvironment
from rooch.crypto.keypair import KeyPair
from rooch.crypto.signer import Signer


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


async def transfer_coins(
    client: RoochClient, 
    signer: Signer,
    recipient_address: str,
    amount: int,
    coin_type: str = "0x1::coin::ROOCH"
) -> Dict[str, Any]:
    """Transfer coins to a recipient
    
    Args:
        client: Rooch client
        signer: Transaction signer
        recipient_address: Recipient's address
        amount: Amount to transfer
        coin_type: Coin type
        
    Returns:
        Transaction result
    """
    print(f"\n=== Transferring {amount} {coin_type} to {recipient_address} ===")
    
    try:
        # Get sender address
        sender_address = signer.get_address()
        print(f"Sender address: {sender_address}")
        
        # Execute the transfer
        result = await client.execute_move_call(
            signer=signer,
            function_id="0x1::coin::transfer",
            type_args=[coin_type],
            args=[[recipient_address, str(amount)]],
            max_gas_amount=10_000_000
        )
        
        print(f"Transfer transaction result: {json.dumps(result, indent=2)}")
        return result
    except Exception as e:
        print(f"Error transferring coins: {e}")
        return {}


async def main() -> None:
    """Main function"""
    try:
        # Connect to Rooch local node by default
        # You can also connect to testnet using RoochEnvironment.TESTNET
        async with RoochClient(RoochEnvironment.LOCAL) as client:
            print("=== Connected to Rooch node ===")
            
            # Generate a new key pair for demonstration
            print("\n=== Generating a new key pair ===")
            keypair = KeyPair.generate()
            signer = Signer(keypair)
            
            # Get the address
            address = signer.get_address()
            print(f"Generated address: {address}")
            
            # Get chain ID
            chain_id = await client.get_chain_id()
            print(f"\n=== Chain ID: {chain_id} ===")
            
            # Get account info
            await get_account_info(client, address)
            
            # Get account balance
            await get_account_balance(client, address)
            
            # Execute a view function (get total supply of ROOCH coin)
            await execute_view_function(
                client,
                "0x1::coin::total_supply",
                ["0x1::coin::ROOCH"],
                []
            )
            
            # Optional: Transfer coins (requires a funded account)
            # For this example, we'll generate a random recipient address
            recipient_keypair = KeyPair.generate()
            recipient_signer = Signer(recipient_keypair)
            recipient_address = recipient_signer.get_address()
            
            print("\n=== Note: The following transfer will likely fail since the demo account isn't funded ===")
            print("=== This is just to demonstrate the API usage pattern ===")
            
            await transfer_coins(
                client,
                signer,
                recipient_address,
                100,  # Amount to transfer
                "0x1::coin::ROOCH"  # Coin type
            )
            
    except Exception as e:
        print(f"Error: {e}")


if __name__ == "__main__":
    asyncio.run(main())