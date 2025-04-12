#!/usr/bin/env python3
# Copyright (c) Rooch Network
# SPDX-License-Identifier: Apache-2.0

"""
Example of publishing a Move module and interacting with it using the Rooch Python SDK.
For this example, we'll use a simple counter module:

module examples::counter {
    use std::signer;
    
    struct Counter has key {
        value: u64,
    }
    
    public fun init(account: &signer) {
        move_to(account, Counter { value: 0 });
    }
    
    public fun increment(account: &signer) acquires Counter {
        let counter = borrow_global_mut<Counter>(signer::address_of(account));
        counter.value = counter.value + 1;
    }
    
    public fun get_value(addr: address): u64 acquires Counter {
        if (exists<Counter>(addr)) {
            borrow_global<Counter>(addr).value
        } else {
            0
        }
    }
}

Compile this module using the Rooch CLI before running this example:
  rooch move build
"""

import asyncio
import json
import os
import sys
from typing import Dict, Any, Optional

from rooch.client.client import RoochClient
from rooch.transport import RoochEnvironment
from rooch.crypto.keypair import KeyPair
from rooch.crypto.signer import Signer
from rooch.bcs.serializer import Args


async def publish_counter_module(client: RoochClient, signer: Signer, module_path: str) -> Dict[str, Any]:
    """Publish the counter module
    
    Args:
        client: Rooch client
        signer: Transaction signer
        module_path: Path to compiled module bytecode
        
    Returns:
        Transaction result
    """
    print(f"\n=== Publishing module from {module_path} ===")
    
    try:
        # Read module bytecode
        with open(module_path, "rb") as f:
            module_bytes = f.read()
        
        # Publish the module
        result = await client.publish_module(
            signer=signer,
            module_bytes=module_bytes,
            max_gas_amount=10_000_000
        )
        
        print(f"Module published: {json.dumps(result, indent=2)}")
        return result
    except Exception as e:
        print(f"Error publishing module: {e}")
        return {}


async def initialize_counter(client: RoochClient, signer: Signer, module_address: str) -> Dict[str, Any]:
    """Initialize counter for the signer
    
    Args:
        client: Rooch client
        signer: Transaction signer
        module_address: Address where the module is published
        
    Returns:
        Transaction result
    """
    print("\n=== Initializing counter ===")
    
    try:
        # Execute the init function
        result = await client.execute_move_call(
            signer=signer,
            function_id=f"{module_address}::counter::init",
            type_args=[],
            args=[]
        )
        
        print(f"Counter initialized: {json.dumps(result, indent=2)}")
        return result
    except Exception as e:
        print(f"Error initializing counter: {e}")
        return {}


async def increment_counter(client: RoochClient, signer: Signer, module_address: str) -> Dict[str, Any]:
    """Increment the counter
    
    Args:
        client: Rooch client
        signer: Transaction signer
        module_address: Address where the module is published
        
    Returns:
        Transaction result
    """
    print("\n=== Incrementing counter ===")
    
    try:
        # Execute the increment function
        result = await client.execute_move_call(
            signer=signer,
            function_id=f"{module_address}::counter::increment",
            type_args=[],
            args=[]
        )
        
        print(f"Counter incremented: {json.dumps(result, indent=2)}")
        return result
    except Exception as e:
        print(f"Error incrementing counter: {e}")
        return {}


async def get_counter_value(client: RoochClient, address: str, module_address: str) -> Optional[int]:
    """Get the counter value
    
    Args:
        client: Rooch client
        address: Address to query counter value for
        module_address: Address where the module is published
        
    Returns:
        Counter value
    """
    print(f"\n=== Getting counter value for {address} ===")
    
    try:
        # Execute the view function
        result = await client.transaction.execute_view_function(
            function_id=f"{module_address}::counter::get_value",
            type_args=[],
            args=[[address]]  # Address as argument
        )
        
        print(f"Counter value: {result}")
        return result
    except Exception as e:
        print(f"Error getting counter value: {e}")
        return None


async def main() -> None:
    """Main function"""
    if len(sys.argv) < 2:
        print("Usage: python publish_module.py <path_to_compiled_module>")
        print("Example: python publish_module.py build/Example/bytecode_modules/counter.mv")
        return
    
    module_path = sys.argv[1]
    
    if not os.path.exists(module_path):
        print(f"Error: Module file not found: {module_path}")
        return
    
    try:
        # Connect to Rooch local node
        async with RoochClient(RoochEnvironment.LOCAL) as client:
            print("=== Connected to Rooch node ===")
            
            # Generate a key pair
            # In a real application, you would use a private key from a secure source
            keypair = KeyPair.generate()
            signer = Signer(keypair)
            
            address = signer.get_address()
            print(f"Using address: {address}")
            
            # Publish the counter module
            result = await publish_counter_module(client, signer, module_path)
            
            # If the publication was successful, interact with the module
            if result:
                # Initialize counter
                await initialize_counter(client, signer, address)
                
                # Get initial counter value
                await get_counter_value(client, address, address)
                
                # Increment counter
                await increment_counter(client, signer, address)
                
                # Get updated counter value
                await get_counter_value(client, address, address)
    
    except Exception as e:
        print(f"Error: {e}")


if __name__ == "__main__":
    asyncio.run(main())