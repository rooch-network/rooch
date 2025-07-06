#!/usr/bin/env python3

import asyncio
import os
from rooch.client.client import RoochClient
from rooch.crypto.keypair import KeyPair
from rooch.crypto.signer import RoochSigner
from rooch.address.rooch import RoochAddress
from rooch.utils.hex import to_hex

async def main():
    # Use external URL
    external_url = os.getenv("ROOCH_EXTERNAL_URL", "http://localhost:6767")
    print(f"Using Rooch node at: {external_url}")
    
    # Create client and signer
    client = RoochClient(external_url)
    test_kp = KeyPair.from_seed("test_seed_for_integration")
    signer = RoochSigner(test_kp)
    
    address = signer.get_address()
    print(f"Testing address: {address}")
    
    # Check account existence first
    try:
        account = await client.account.get_account(address)
        print(f"Account info: {account}")
    except Exception as e:
        print(f"Error getting account: {e}")
    
    # Check sequence number
    try:
        rooch_address = RoochAddress.from_hex(address)
        address_arg_bytes = rooch_address.to_bytes()
        address_arg_hex = to_hex(address_arg_bytes)
        
        print(f"Address bytes: {address_arg_hex}")
        
        result = await client.transaction.execute_view_function(
            function_id="0x3::account::sequence_number",
            type_args=[],
            args=[address_arg_hex]
        )
        
        print(f"Sequence number result: {result}")
        
    except Exception as e:
        print(f"Error getting sequence number: {e}")
    
    await client.close()

if __name__ == "__main__":
    asyncio.run(main())
