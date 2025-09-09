#!/usr/bin/env python3

import os
import asyncio
from rooch.client.client import RoochClient
from rooch.crypto.keypair import KeyPair
from rooch.crypto.signer import RoochSigner

async def debug_sequence_number():
    # Use external URL
    external_url = os.getenv("ROOCH_EXTERNAL_URL", "http://localhost:6767")
    print(f"Using Rooch node at: {external_url}")
    
    # Create client and signer (same as in tests)
    client = RoochClient(external_url)
    test_kp = KeyPair.from_seed("test_seed_for_integration")
    signer = RoochSigner(test_kp)
    
    address = signer.get_address()
    print(f"Testing address: {address}")
    
    try:
        # Try to get sequence number directly
        seq_num = await client.account.get_account_sequence_number(address)
        print(f"Account sequence number: {seq_num}")
        
        # Also check if account exists
        try:
            account = await client.account.get_account(address)
            print(f"Account exists: {account is not None}")
            if account:
                print(f"Account data: {account}")
        except Exception as e:
            print(f"Account doesn't exist or error: {e}")
            
    except Exception as e:
        print(f"Error getting sequence number: {e}")
        import traceback
        traceback.print_exc()
    
    await client.close()

if __name__ == "__main__":
    asyncio.run(debug_sequence_number())
