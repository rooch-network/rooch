#!/usr/bin/env python3

import pytest
import asyncio
from rooch.client.client import RoochClient
from rooch.crypto.keypair import KeyPair
from rooch.crypto.signer import RoochSigner
from rooch.address.rooch import RoochAddress
from rooch.utils.hex import to_hex


@pytest.mark.asyncio
async def test_debug_account_status(rooch_client):
    """Debug account status and sequence number"""
    test_kp = KeyPair.from_seed("test_seed_for_integration")
    signer = RoochSigner(test_kp)
    
    address = signer.get_address()
    print(f"\nTesting address: {address}")
    
    # Check account existence first
    try:
        account = await rooch_client.account.get_account(address)
        print(f"Account info: {account}")
    except Exception as e:
        print(f"Error getting account: {e}")
    
    # Check sequence number
    try:
        rooch_address = RoochAddress.from_hex(address)
        address_arg_bytes = rooch_address.to_bytes()
        address_arg_hex = to_hex(address_arg_bytes)
        
        print(f"Address bytes for view function: {address_arg_hex}")
        
        result = await rooch_client.transaction.execute_view_function(
            function_id="0x3::account::sequence_number",
            type_args=[],
            args=[address_arg_hex]
        )
        
        print(f"Sequence number result: {result}")
        
    except Exception as e:
        print(f"Error getting sequence number: {e}")
        import traceback
        traceback.print_exc()
